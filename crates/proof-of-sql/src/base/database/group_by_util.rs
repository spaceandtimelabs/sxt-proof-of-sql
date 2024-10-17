//! Contains the utility functions for the `GroupByExec` node.

use crate::base::{
    database::{filter_util::filter_column_by_index, Column, OwnedColumn},
    if_rayon,
    scalar::Scalar,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::cmp::Ordering;
use itertools::Itertools;
#[cfg(feature = "rayon")]
use rayon::prelude::ParallelSliceMut;
use snafu::Snafu;

/// The output of the `aggregate_columns` function.
#[derive(Debug)]
pub struct AggregatedColumns<'a, S: Scalar> {
    /// The columns that are being grouped by. These are all unique and correspond to each group.
    /// This is effectively just the original `group_by` columns filtered by the selection.
    pub group_by_columns: Vec<Column<'a, S>>,
    /// Resulting sums of the groups for the columns in `sum_columns_in`.
    pub sum_columns: Vec<&'a [S]>,
    /// Resulting maxima of the groups for the columns in `max_columns_in`. Note that for empty groups
    /// the result will be `None`.
    pub max_columns: Vec<&'a [Option<S>]>,
    /// Resulting minima of the groups for the columns in `min_columns_in`. Note that for empty groups
    /// the result will be `None`.
    pub min_columns: Vec<&'a [Option<S>]>,
    /// The number of rows in each group.
    pub count_column: &'a [i64],
}
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum AggregateColumnsError {
    #[snafu(display("Column length mismatch"))]
    ColumnLengthMismatch,
}

/// This is a function that gives the result of a group by query similar to the following:
/// ```sql
///     SELECT <group_by[0]>, <group_by[1]>, ..., SUM(<sum_columns[0]>), SUM(<sum_columns[1]>), ...,
///      MAX(<max_columns[0]>), ..., MIN(<min_columns[0]>), ..., COUNT(*)
///         WHERE selection GROUP BY <group_by[0]>, <group_by[1]>, ...
/// ```
///
/// This function takes a selection vector and a set of `group_by` and sum columns and returns
/// the given columns aggregated by the `group_by` columns only for the selected rows.
pub fn aggregate_columns<'a, S: Scalar>(
    alloc: &'a Bump,
    group_by_columns_in: &[Column<'a, S>],
    sum_columns_in: &[Column<S>],
    max_columns_in: &[Column<S>],
    min_columns_in: &[Column<S>],
    selection_column_in: &[bool],
) -> Result<AggregatedColumns<'a, S>, AggregateColumnsError> {
    // Check that all the columns have the same length
    let len = selection_column_in.len();
    if group_by_columns_in
        .iter()
        .chain(sum_columns_in.iter())
        .chain(max_columns_in.iter())
        .chain(min_columns_in.iter())
        .any(|col| col.len() != len)
    {
        return Err(AggregateColumnsError::ColumnLengthMismatch);
    }

    // `filtered_indexes` is a vector of indexes of the rows that are selected. We sort this vector
    // so that all the rows in the same group are next to each other.
    let mut filtered_indexes: Vec<_> = selection_column_in
        .iter()
        .enumerate()
        .filter(|&(_, &b)| b)
        .map(|(i, _)| i)
        .collect();
    if_rayon!(
        filtered_indexes.par_sort_unstable_by(|&a, &b| compare_indexes_by_columns(
            group_by_columns_in,
            a,
            b
        )),
        filtered_indexes.sort_unstable_by(|&a, &b| compare_indexes_by_columns(
            group_by_columns_in,
            a,
            b
        ))
    );

    // `group_by_result_indexes` gives a single index for each group in `filtered_indexes`. It does
    // not matter which index is chosen for each group, so we choose the first one. This is only used
    // to extract the `group_by_columns_out`, which is the same for all elements in the group.
    let (counts, group_by_result_indexes): (Vec<_>, Vec<_>) = filtered_indexes
        .iter()
        .dedup_by_with_count(|&&a, &&b| {
            compare_indexes_by_columns(group_by_columns_in, a, b) == Ordering::Equal
        })
        .multiunzip();
    let group_by_columns_out: Vec<_> = group_by_columns_in
        .iter()
        .map(|column| filter_column_by_index(alloc, column, &group_by_result_indexes))
        .collect();

    // This calls the `sum_aggregate_column_by_index_counts` function on each column in `sum_columns`
    // and gives a vector of `S` slices
    let sum_columns_out: Vec<_> = sum_columns_in
        .iter()
        .map(|column| {
            sum_aggregate_column_by_index_counts(alloc, column, &counts, &filtered_indexes)
        })
        .collect();

    let max_columns_out: Vec<_> = max_columns_in
        .iter()
        .map(|column| {
            max_aggregate_column_by_index_counts(alloc, column, &counts, &filtered_indexes)
        })
        .collect();

    let min_columns_out: Vec<_> = min_columns_in
        .iter()
        .map(|column| {
            min_aggregate_column_by_index_counts(alloc, column, &counts, &filtered_indexes)
        })
        .collect();

    // Cast the counts to something compatible with BigInt.
    let count_column_out = alloc.alloc_slice_fill_iter(counts.into_iter().map(|c| c as i64));

    Ok(AggregatedColumns {
        group_by_columns: group_by_columns_out,
        sum_columns: sum_columns_out,
        max_columns: max_columns_out,
        min_columns: min_columns_out,
        count_column: count_column_out,
    })
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped sums of `column`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `column`.
///
/// See [`sum_aggregate_slice_by_index_counts`] for an example. This is a helper wrapper around that function.
pub(crate) fn sum_aggregate_column_by_index_counts<'a, S: Scalar>(
    alloc: &'a Bump,
    column: &Column<S>,
    counts: &[usize],
    indexes: &[usize],
) -> &'a [S] {
    match column {
        Column::TinyInt(_, col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::SmallInt(_, col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Int(_, col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::BigInt(_, col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Int128(_, col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Decimal75(_, _, _, col) => {
            sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes)
        }
        Column::Scalar(_, col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        // The following should never be reached because the `SUM` function can only be applied to numeric types.
        Column::VarChar(_, _) | Column::TimestampTZ(..) | Column::Boolean(..) => {
            unreachable!("SUM can not be applied to non-numeric types")
        }
    }
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped maxima of `column`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `column`.
///
/// See [`max_aggregate_slice_by_index_counts`] for an example. This is a helper wrapper around that function.
pub(crate) fn max_aggregate_column_by_index_counts<'a, S: Scalar>(
    alloc: &'a Bump,
    column: &Column<S>,
    counts: &[usize],
    indexes: &[usize],
) -> &'a [Option<S>] {
    match column {
        Column::Boolean(_, col) => max_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::TinyInt(_, col) => max_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::SmallInt(_, col) => max_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Int(_, col) => max_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::BigInt(_, col) => max_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Int128(_, col) => max_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Decimal75(_, _, _, col) => {
            max_aggregate_slice_by_index_counts(alloc, col, counts, indexes)
        }
        Column::TimestampTZ(_, _, _, col) => {
            max_aggregate_slice_by_index_counts(alloc, col, counts, indexes)
        }
        Column::Scalar(_, col) => max_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        // The following should never be reached because the `MAX` function can't be applied to varchar.
        Column::VarChar(_, _) => {
            unreachable!("MAX can not be applied to varchar")
        }
    }
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped minima of `column`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `column`.
///
/// See [`min_aggregate_slice_by_index_counts`] for an example. This is a helper wrapper around that function.
pub(crate) fn min_aggregate_column_by_index_counts<'a, S: Scalar>(
    alloc: &'a Bump,
    column: &Column<S>,
    counts: &[usize],
    indexes: &[usize],
) -> &'a [Option<S>] {
    match column {
        Column::Boolean(_, col) => min_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::TinyInt(_, col) => min_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::SmallInt(_, col) => min_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Int(_, col) => min_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::BigInt(_, col) => min_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Int128(_, col) => min_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Decimal75(_, _, _, col) => {
            min_aggregate_slice_by_index_counts(alloc, col, counts, indexes)
        }
        Column::TimestampTZ(_, _, _, col) => {
            min_aggregate_slice_by_index_counts(alloc, col, counts, indexes)
        }
        Column::Scalar(_, col) => min_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        // The following should never be reached because the `MIN` function can't be applied to varchar.
        Column::VarChar(_, _) => {
            unreachable!("MIN can not be applied to varchar")
        }
    }
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped sums of `slice`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `slice`.
///
/// For example:
/// ```ignore
/// let slice_a = &[
///     100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
/// ];
/// let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
/// let counts = &[3, 3, 4];
/// let expected = &[
///     Curve25519Scalar::from(112 + 111 + 101),
///     Curve25519Scalar::from(110 + 102 + 103),
///     Curve25519Scalar::from(106 + 114 + 113 + 109),
/// ];
/// let alloc = Bump::new();
/// let result = sum_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
/// assert_eq!(result, expected);
/// ```
pub(crate) fn sum_aggregate_slice_by_index_counts<'a, S, T>(
    alloc: &'a Bump,
    slice: &[T],
    counts: &[usize],
    indexes: &[usize],
) -> &'a [S]
where
    for<'b> S: From<&'b T> + Scalar,
{
    let mut index = 0;
    alloc.alloc_slice_fill_iter(counts.iter().map(|&count| {
        let start = index;
        index += count;
        indexes[start..index]
            .iter()
            .map(|i| S::from(&slice[*i]))
            .sum()
    }))
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped maxima of `slice`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `slice`. Note that for empty groups the result
/// will be `None`.
///
/// For example:
/// ```ignore
/// let slice_a = &[
///     100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
/// ];
/// let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
/// let counts = &[3, 3, 4];
/// let expected = &[
///     Some(Curve25519Scalar::from(max(112, 111, 101))),
///     Some(Curve25519Scalar::from(max(110, 102, 103))),
///     Some(Curve25519Scalar::from(max(106, 114, 113, 109))),
/// ];
/// let alloc = Bump::new();
/// let result = max_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
/// assert_eq!(result, expected);
/// ```
pub(crate) fn max_aggregate_slice_by_index_counts<'a, S, T>(
    alloc: &'a Bump,
    slice: &[T],
    counts: &[usize],
    indexes: &[usize],
) -> &'a [Option<S>]
where
    for<'b> S: From<&'b T> + Scalar,
{
    let mut index = 0;
    alloc.alloc_slice_fill_iter(counts.iter().map(|&count| {
        let start = index;
        index += count;
        // Note that currently we can't run this on empty slices
        // In the future we have to support NULL values
        indexes[start..index]
            .iter()
            .map(|i| S::from(&slice[*i]))
            .max_by(super::super::scalar::Scalar::signed_cmp)
    }))
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped minima of `slice`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `slice`. Note that for empty groups the result
/// will be `None`.
///
/// For example:
/// ```ignore
/// let slice_a = &[
///     100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
/// ];
/// let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
/// let counts = &[3, 3, 4];
/// let expected = &[
///     Some(Curve25519Scalar::from(min(112, 111, 101))),
///     Some(Curve25519Scalar::from(min(110, 102, 103))),
///     Some(Curve25519Scalar::from(min(106, 114, 113, 109))),
/// ];
/// let alloc = Bump::new();
/// let result = min_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
/// assert_eq!(result, expected);
/// ```
pub(crate) fn min_aggregate_slice_by_index_counts<'a, S, T>(
    alloc: &'a Bump,
    slice: &[T],
    counts: &[usize],
    indexes: &[usize],
) -> &'a [Option<S>]
where
    for<'b> S: From<&'b T> + Scalar,
{
    let mut index = 0;
    alloc.alloc_slice_fill_iter(counts.iter().map(|&count| {
        let start = index;
        index += count;
        indexes[start..index]
            .iter()
            .map(|i| S::from(&slice[*i]))
            .min_by(super::super::scalar::Scalar::signed_cmp)
    }))
}

/// Compares the tuples `(group_by[0][i], group_by[1][i], ...)` and
/// `(group_by[0][j], group_by[1][j], ...)` in lexicographic order.
pub(crate) fn compare_indexes_by_columns<S: Scalar>(
    group_by: &[Column<S>],
    i: usize,
    j: usize,
) -> Ordering {
    group_by
        .iter()
        .map(|col| match col {
            Column::Boolean(_, col) => col[i].cmp(&col[j]),
            Column::TinyInt(_, col) => col[i].cmp(&col[j]),
            Column::SmallInt(_, col) => col[i].cmp(&col[j]),
            Column::Int(_, col) => col[i].cmp(&col[j]),
            Column::BigInt(_, col) | Column::TimestampTZ(.., col) => col[i].cmp(&col[j]),
            Column::Int128(_, col) => col[i].cmp(&col[j]),
            Column::Decimal75(_, _, _, col) => col[i].signed_cmp(&col[j]),
            Column::Scalar(_, col) => col[i].cmp(&col[j]),
            Column::VarChar(_, (col, _)) => col[i].cmp(col[j]),
        })
        .find(|&ord| ord != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

/// Compares the tuples `(group_by[0][i], group_by[1][i], ...)` and
/// `(group_by[0][j], group_by[1][j], ...)` in lexicographic order.
///
/// Identical in functionality to [`compare_indexes_by_columns`]
pub(crate) fn compare_indexes_by_owned_columns<S: Scalar>(
    group_by: &[&OwnedColumn<S>],
    i: usize,
    j: usize,
) -> Ordering {
    group_by
        .iter()
        .map(|col| match col {
            OwnedColumn::Boolean(_, col) => col[i].cmp(&col[j]),
            OwnedColumn::TinyInt(_, col) => col[i].cmp(&col[j]),
            OwnedColumn::SmallInt(_, col) => col[i].cmp(&col[j]),
            OwnedColumn::Int(_, col) => col[i].cmp(&col[j]),
            OwnedColumn::BigInt(_, col) | OwnedColumn::TimestampTZ(.., col) => col[i].cmp(&col[j]),
            OwnedColumn::Int128(_, col) => col[i].cmp(&col[j]),
            OwnedColumn::Decimal75(.., col) => col[i].signed_cmp(&col[j]),
            OwnedColumn::Scalar(_, col) => col[i].cmp(&col[j]),
            OwnedColumn::VarChar(_, col) => col[i].cmp(&col[j]),
        })
        .find(|&ord| ord != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}
