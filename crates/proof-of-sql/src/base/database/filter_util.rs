use crate::base::{database::Column, scalar::Scalar};
use alloc::vec::Vec;
use bumpalo::Bump;

/// This function takes a selection vector and a set of columns and returns a
/// new set of columns that only contains the selected rows. The function
/// panics if the selection vector is a different length than the columns.
///
/// The function returns a tuple of the filtered columns and the number of
/// rows in the filtered columns.
/// # Panics
/// This function requires that `columns` and `selection` have the same length.
pub fn filter_columns<'a, S: Scalar>(
    alloc: &'a Bump,
    columns: &[Column<'a, S>],
    selection: &[bool],
) -> (Vec<Column<'a, S>>, usize) {
    for col in columns {
        assert_eq!(col.len(), selection.len());
    }
    let indexes: Vec<_> = selection
        .iter()
        .enumerate()
        .filter(|(_, &b)| b)
        .map(|(i, _)| i)
        .collect();
    let result_length = indexes.len();
    let filtered_result: Vec<_> = columns
        .iter()
        .map(|column| filter_column_by_index(alloc, column, &indexes))
        .collect();
    (filtered_result, result_length)
}
/// This function takes an index vector and a `Column` and returns a
/// new set of columns that only contains the selected indexes. It is assumed that
/// the indexes are valid.
pub fn filter_column_by_index<'a, S: Scalar>(
    alloc: &'a Bump,
    column: &Column<'a, S>,
    indexes: &[usize],
) -> Column<'a, S> {
    match column {
        Column::Boolean(col) => {
            Column::Boolean(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Uint8(col) => {
            Column::Uint8(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::TinyInt(col) => {
            Column::TinyInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::SmallInt(col) => {
            Column::SmallInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Int(col) => {
            Column::Int(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::BigInt(col) => {
            Column::BigInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Int128(col) => {
            Column::Int128(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::VarChar((col, scals)) => Column::VarChar((
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| scals[i])),
        )),
        Column::VarBinary((col, scals)) => Column::VarBinary((
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| scals[i])),
        )),
        Column::Scalar(col) => {
            Column::Scalar(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Decimal75(precision, scale, col) => Column::Decimal75(
            *precision,
            *scale,
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
        ),
        Column::TimestampTZ(tu, tz, col) => Column::TimestampTZ(
            *tu,
            *tz,
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
        ),
        Column::FixedSizeBinary(byte_width, col) => {
            let bw = byte_width.width_as_usize();
            let allocated_bytes = alloc.alloc_slice_fill_with(indexes.len() * bw, |i| {
                let row_idx = indexes[i / bw];
                col[row_idx * bw + (i % bw)]
            });
            Column::FixedSizeBinary(*byte_width, allocated_bytes)
        }
    }
}
