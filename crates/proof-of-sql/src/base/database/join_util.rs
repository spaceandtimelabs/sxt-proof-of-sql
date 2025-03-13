use super::{
    apply_column_to_indexes,
    order_by_util::{compare_indexes_by_columns, compare_single_row_of_tables},
    union_util::column_union,
    Column, ColumnOperationResult, ColumnRepeatOp, ElementwiseRepeatOp, RepetitionOp, Table,
    TableOperationError, TableOperationResult, TableOptions,
};
use crate::base::scalar::Scalar;
use alloc::vec::Vec;
use bumpalo::Bump;
use core::cmp::Ordering;
use itertools::Itertools;
use tracing::{span, Level};

/// Compute the set union of two slices of columns, deduplicate and sort the result.
///
/// Notes
/// 1. This is mostly used for joins.
/// 2. We do not check whether columns in the args have the same length, as we assume that the columns in an arg are already from the same table.
///
/// # Panics
/// The function panics if we feed in incorrect data (e.g. Num of rows in `left_on` and `right_on` being different).
#[tracing::instrument(name = "join_util::ordered_set_union", level = "debug", skip_all)]
pub(crate) fn ordered_set_union<'a, S: Scalar>(
    left_on: &[Column<'a, S>],
    right_on: &[Column<'a, S>],
    alloc: &'a Bump,
) -> TableOperationResult<Vec<Column<'a, S>>> {
    //1. Union the columns
    if left_on.is_empty() {
        return Ok(Vec::new());
    }
    let span = span!(Level::DEBUG, "ordered_set_union::raw_union").entered();
    let raw_union = left_on
        .iter()
        .zip_eq(right_on)
        .map(|(left, right)| column_union(&[left, right], alloc, left.column_type()))
        .collect::<ColumnOperationResult<Vec<_>>>()?;
    span.exit();
    //2. Sort and deduplicate the raw union by indexes
    // Allowed because we already checked that the columns aren't empty
    let indexes: Vec<usize> = (0..raw_union[0].len())
        .sorted_unstable_by(|&a, &b| compare_indexes_by_columns(&raw_union, a, b))
        .dedup_by(|&a, &b| compare_indexes_by_columns(&raw_union, a, b) == Ordering::Equal)
        .collect();
    //3. Apply the deduplicated indexes to the raw union
    let result = raw_union
        .into_iter()
        .map(|column| apply_column_to_indexes(&column, alloc, &indexes))
        .collect::<ColumnOperationResult<Vec<_>>>()?;
    Ok(result)
}

/// Get multiplicities of rows of `data` in `unique`.
///
/// `data` consists of rows possibly present in `unique` and `unique` has only
/// unique rows. We want to get the number of times each row in `unique` is present
/// in `data`.
///
/// Note that schema incompatibility is caught by `compare_single_row_of_tables`.
#[tracing::instrument(name = "join_util::get_multiplicities", level = "debug", skip_all)]
pub(crate) fn get_multiplicities<'a, S: Scalar>(
    data: &[Column<'a, S>],
    unique: &[Column<'a, S>],
    alloc: &'a Bump,
) -> &'a [i128] {
    // If unique is empty, the multiplicities vector is empty
    if unique.is_empty() {
        return alloc.alloc_slice_fill_copy(0, 0_i128);
    }
    let num_unique_rows = unique[0].len();
    // If data is empty, all multiplicities are 0
    if data.is_empty() {
        return alloc.alloc_slice_fill_copy(num_unique_rows, 0_i128);
    }
    let num_rows = data[0].len();
    let multiplicities = (0..num_unique_rows)
        .map(|unique_index| {
            (0..num_rows)
                .filter(|&data_index| {
                    compare_single_row_of_tables(data, unique, data_index, unique_index)
                        == Ok(Ordering::Equal)
                })
                .count() as i128
        })
        .collect::<Vec<_>>();
    alloc.alloc_slice_copy(multiplicities.as_slice())
}

/// Compute the CROSS JOIN / cartesian product of two tables.
///
/// # Panics
/// The function if written correctly can not actually panic.
pub fn cross_join<'a, S: Scalar>(
    left: &Table<'a, S>,
    right: &Table<'a, S>,
    alloc: &'a Bump,
) -> Table<'a, S> {
    let left_num_rows = left.num_rows();
    let right_num_rows = right.num_rows();
    let product_num_rows = left_num_rows * right_num_rows;
    Table::<'a, S>::try_from_iter_with_options(
        left.inner_table()
            .iter()
            .map(|(ident, column)| {
                (
                    ident.clone(),
                    ColumnRepeatOp::column_op(column, alloc, right_num_rows),
                )
            })
            .chain(right.inner_table().iter().map(|(ident, column)| {
                (
                    ident.clone(),
                    ElementwiseRepeatOp::column_op(column, alloc, left_num_rows),
                )
            })),
        TableOptions::new(Some(product_num_rows)),
    )
    .expect("Table creation should not fail")
}

/// Get columns from a table with given indexes.
///
/// The function returns an error if any of the indexes are out of bounds.
pub(crate) fn get_columns_of_table<'a, S: Scalar>(
    table: &Table<'a, S>,
    indexes: &[usize],
) -> TableOperationResult<Vec<Column<'a, S>>> {
    indexes
        .iter()
        .map(|&i| {
            table
                .column(i)
                .copied()
                .ok_or(TableOperationError::ColumnIndexOutOfBounds { column_index: i })
        })
        .collect::<TableOperationResult<Vec<_>>>()
}

/// Get sort merge join indexes
///
/// Get indexes of rows in `left` and `right` that match on the columns `left_on` and `right_on`.
/// The results are sorted by (`left_index`, `right_index`).
/// # Panics
/// The function panics if we feed in incorrect data (e.g. Num of rows in `left` and some column of `left_on` being different).
pub(crate) fn get_sort_merge_join_indexes<'a, S: Scalar>(
    left_on: &'a [Column<'a, S>],
    right_on: &'a [Column<'a, S>],
    left_num_rows: usize,
    right_num_rows: usize,
) -> Vec<(usize, usize)> {
    // Validate input sizes
    for column in left_on {
        assert_eq!(column.len(), left_num_rows);
    }
    for column in right_on {
        assert_eq!(column.len(), right_num_rows);
    }

    // Sort indexes by the join columns
    let left_indexes =
        (0..left_num_rows).sorted_unstable_by(|&a, &b| compare_indexes_by_columns(left_on, a, b));
    let right_indexes =
        (0..right_num_rows).sorted_unstable_by(|&a, &b| compare_indexes_by_columns(right_on, a, b));

    let mut left_iter = left_indexes.into_iter().peekable();
    let mut right_iter = right_indexes.into_iter().peekable();

    core::iter::from_fn(move || {
        let (&left_index, &right_index) = (left_iter.peek()?, right_iter.peek()?);

        match compare_single_row_of_tables(left_on, right_on, left_index, right_index).ok()? {
            Ordering::Less => {
                left_iter.next();
                Some(Vec::new())
            }
            Ordering::Greater => {
                right_iter.next();
                Some(Vec::new())
            }
            Ordering::Equal => {
                // Gather all rows from left_iter matching the current key
                let left_group: Vec<_> = left_iter
                    .clone()
                    .take_while(|&lidx| {
                        compare_indexes_by_columns(left_on, left_index, lidx) == Ordering::Equal
                    })
                    .collect();

                // Gather all rows from right_iter matching the current key
                let right_group: Vec<_> = right_iter
                    .clone()
                    .take_while(|&ridx| {
                        compare_indexes_by_columns(right_on, right_index, ridx) == Ordering::Equal
                    })
                    .collect();

                // Advance the iterators
                left_iter.nth(left_group.len() - 1);
                right_iter.nth(right_group.len() - 1);

                // Generate all pairs (Cartesian product)
                let pairs: Vec<_> = left_group
                    .iter()
                    .cartesian_product(right_group.iter())
                    .map(|(&lidx, &ridx)| (lidx, ridx))
                    .collect();

                Some(pairs)
            }
        }
    })
    .flatten()
    .sorted()
    .collect::<Vec<(usize, usize)>>()
}

/// Apply sort merge join indexes
///
/// Currently we only support INNER JOINs and only support joins on equalities.
/// In terms of ordering of columns we retain
/// 1. Join columns
/// 2. Other columns from the left table
/// 3. Other columns from the right table
/// # Panics
/// The function panics if we feed in incorrect data (e.g. Num of rows in `left` and some column of `left_on` being different).
pub fn apply_sort_merge_join_indexes<'a, S: Scalar>(
    left: &Table<'a, S>,
    right: &Table<'a, S>,
    left_join_column_indexes: &[usize],
    right_join_column_indexes: &[usize],
    left_row_indexes: &[usize],
    right_row_indexes: &[usize],
    alloc: &'a Bump,
) -> ColumnOperationResult<Vec<Column<'a, S>>> {
    let left_other_col_indexes = (0..left.num_columns())
        .filter(|i| !left_join_column_indexes.contains(i))
        .collect::<Vec<_>>();
    let right_other_col_indexes = (0..right.num_columns())
        .filter(|i| !right_join_column_indexes.contains(i))
        .collect::<Vec<_>>();
    left_join_column_indexes
        .iter()
        .map(|i| -> ColumnOperationResult<_> {
            apply_column_to_indexes(
                left.column(*i).expect(
                    "Column definitely exists due to how `left_join_column_indexes` is constructed",
                ),
                alloc,
                left_row_indexes,
            )
        })
        .chain(
            left_other_col_indexes
                .iter()
                .map(|i| -> ColumnOperationResult<_> {
                    apply_column_to_indexes(
                left.column(*i).expect(
                    "Column definitely exists due to how `left_other_col_indexes` is constructed",
                ),
                alloc,
                left_row_indexes,
            )
                }),
        )
        .chain(
            right_other_col_indexes
                .iter()
                .map(|i| -> ColumnOperationResult<_> {
                    apply_column_to_indexes(
                right.column(*i).expect(
                    "Column definitely exists due to how `right_other_col_indexes` is constructed",
                ),
                alloc,
                right_row_indexes,
            )
                }),
        )
        .collect::<ColumnOperationResult<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{database::Column, scalar::test_scalar::TestScalar};
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_do_cross_joins() {
        let bump = Bump::new();
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();
        let d: Ident = "d".into();
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[1_i16, 2, 3])),
                (b.clone(), Column::Int(&[4_i32, 5, 6])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[7_i64, 8, 9])),
                (d.clone(), Column::Int128(&[10_i128, 11, 12])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 9);
        assert_eq!(result.num_columns(), 4);
        assert_eq!(
            result.inner_table()[&a].as_smallint().unwrap(),
            &[1_i16, 2, 3, 1, 2, 3, 1, 2, 3]
        );
        assert_eq!(
            result.inner_table()[&b].as_int().unwrap(),
            &[4_i32, 5, 6, 4, 5, 6, 4, 5, 6]
        );
        assert_eq!(
            result.inner_table()[&c].as_bigint().unwrap(),
            &[7_i64, 7, 7, 8, 8, 8, 9, 9, 9]
        );
        assert_eq!(
            result.inner_table()[&d].as_int128().unwrap(),
            &[10_i128, 10, 10, 11, 11, 11, 12, 12, 12]
        );
    }

    #[test]
    fn we_can_do_cross_joins_if_one_table_has_no_rows() {
        let bump = Bump::new();
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();
        let d: Ident = "d".into();

        // Right table has no rows
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[1_i16, 2, 3])),
                (b.clone(), Column::Int(&[4_i32, 5, 6])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[0_i64; 0])),
                (d.clone(), Column::Int128(&[0_i128; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 4);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);
        assert_eq!(result.inner_table()[&c].as_bigint().unwrap(), &[0_i64; 0]);
        assert_eq!(result.inner_table()[&d].as_int128().unwrap(), &[0_i128; 0]);

        // Left table has no rows
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[0_i16; 0])),
                (b.clone(), Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[7_i64, 8, 9])),
                (d.clone(), Column::Int128(&[10_i128, 11, 12])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 4);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);
        assert_eq!(result.inner_table()[&c].as_bigint().unwrap(), &[0_i64; 0]);
        assert_eq!(result.inner_table()[&d].as_int128().unwrap(), &[0_i128; 0]);

        // Both tables have no rows
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[0_i16; 0])),
                (b.clone(), Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[0_i64; 0])),
                (d.clone(), Column::Int128(&[0_i128; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 4);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);
        assert_eq!(result.inner_table()[&c].as_bigint().unwrap(), &[0_i64; 0]);
        assert_eq!(result.inner_table()[&d].as_int128().unwrap(), &[0_i128; 0]);
    }

    #[test]
    fn we_can_do_cross_joins_if_one_table_has_no_columns() {
        // Left table has no columns
        let bump = Bump::new();
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();
        let d: Ident = "d".into();
        let left =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(2)))
                .expect("Table creation should not fail");

        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[7_i64, 8])),
                (d.clone(), Column::Int128(&[10_i128, 11])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");

        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 4);
        assert_eq!(result.num_columns(), 2);
        assert_eq!(
            result.inner_table()[&c].as_bigint().unwrap(),
            &[7_i64, 7, 8, 8]
        );
        assert_eq!(
            result.inner_table()[&d].as_int128().unwrap(),
            &[10_i128, 10, 11, 11]
        );

        // Right table has no columns
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[1_i16, 2])),
                (b.clone(), Column::Int(&[4_i32, 5])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(0)))
                .expect("Table creation should not fail");
        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 2);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);

        // Both tables have no columns
        let left =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(2)))
                .expect("Table creation should not fail");
        let right =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(7)))
                .expect("Table creation should not fail");
        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 14);
        assert_eq!(result.num_columns(), 0);

        // Both tables have no columns and no rows
        let left =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(0)))
                .expect("Table creation should not fail");
        let right =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(0)))
                .expect("Table creation should not fail");
        let result = cross_join(&left, &right, &bump);
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 0);
    }

    /// Ordered Set Union
    #[test]
    fn we_can_do_ordered_set_union_success_single_column() {
        let alloc = Bump::new();

        // Two single-column slices: left_on and right_on
        let left_on = vec![Column::<TestScalar>::Boolean(&[true, false, true])];
        let right_on = vec![Column::<TestScalar>::Boolean(&[false, true])];

        // Union the columns
        let result = ordered_set_union(&left_on, &right_on, &alloc);

        assert!(result.is_ok(), "Expected Ok result from ordered_set_union");
        let collection = result.unwrap();

        // We expect just one column in the final result
        assert_eq!(collection.len(), 1, "Should have exactly one column");
        assert_eq!(collection[0], Column::<TestScalar>::Boolean(&[false, true]));
    }

    #[test]
    fn we_can_do_ordered_set_union_success_multiple_columns() {
        let alloc = Bump::new();
        let left_on = vec![
            Column::<TestScalar>::Boolean(&[true, true, false, false]),
            Column::<TestScalar>::Int(&[1, 2, 3, 3]),
            Column::<TestScalar>::BigInt(&[7_i64, 8, 7, 7]),
        ];
        let right_on = vec![
            Column::<TestScalar>::Boolean(&[true, false]),
            Column::<TestScalar>::Int(&[2, 4]),
            Column::<TestScalar>::BigInt(&[9_i64, 9]),
        ];

        let result = ordered_set_union(&left_on, &right_on, &alloc);
        assert!(result.is_ok(), "Expected Ok result from ordered_set_union");
        let collection = result.unwrap();
        assert_eq!(
            collection.len(),
            3,
            "Should produce exactly three columns in the final result"
        );
        assert_eq!(
            collection[0],
            Column::<TestScalar>::Boolean(&[false, false, true, true, true])
        );
        assert_eq!(collection[1], Column::<TestScalar>::Int(&[3, 4, 1, 2, 2]));
        assert_eq!(
            collection[2],
            Column::<TestScalar>::BigInt(&[7_i64, 9, 7, 8, 9])
        );
    }

    #[test]
    fn we_can_do_ordered_set_union_empty_slices() {
        let alloc = Bump::new();
        // Both sides have zero columns
        let left_on: Vec<Column<TestScalar>> = vec![];
        let right_on: Vec<Column<TestScalar>> = vec![];

        let result = ordered_set_union(&left_on, &right_on, &alloc);
        assert!(result.is_ok(), "Empty slices should not fail");
        let collection = result.unwrap();
        assert_eq!(collection.len(), 0, "Empty slices => no columns in result");
    }

    /// Get Multiplicities
    #[test]
    fn we_can_get_multiplicities_empty_scenarios() {
        let alloc = Bump::new();
        let empty_data: Vec<Column<TestScalar>> = vec![];
        let empty_unique: Vec<Column<TestScalar>> = vec![];

        // 1) Both 'data' and 'unique' empty
        let result = get_multiplicities(&empty_data, &empty_unique, &alloc);
        assert!(
            result.is_empty(),
            "When both are empty, result should be empty"
        );

        // 2) 'unique' empty, 'data' non-empty
        let nonempty_data = vec![Column::<TestScalar>::Boolean(&[true, false])];
        let result = get_multiplicities(&nonempty_data, &empty_unique, &alloc);
        assert!(
            result.is_empty(),
            "When 'unique' is empty, result must be empty"
        );

        // 3) 'unique' non-empty, 'data' empty => all zeros
        let nonempty_unique = vec![Column::<TestScalar>::Boolean(&[true, true, false])];
        let result = get_multiplicities(&empty_data, &nonempty_unique, &alloc);
        assert_eq!(
            result,
            &[0_i128, 0, 0],
            "If data is empty, multiplicities should be zeros"
        );
    }

    #[test]
    fn we_can_get_multiplicities() {
        let alloc = Bump::new();
        let data = vec![
            Column::<TestScalar>::Boolean(&[true, false, true, true, true]),
            Column::<TestScalar>::Int(&[1, 2, 1, 1, 2]),
            Column::<TestScalar>::BigInt(&[1_i64, 2, 1, 1, 1]),
        ];
        let unique = vec![
            Column::<TestScalar>::Boolean(&[false, false, true, true]),
            Column::<TestScalar>::Int(&[2, 3, 1, 2]),
            Column::<TestScalar>::BigInt(&[2_i64, 4, 1, 1]),
        ];

        let result = get_multiplicities(&data, &unique, &alloc);
        assert_eq!(result, &[1, 0, 3, 1], "Expected multiplicities");
    }

    // Get Columns of Table
    #[test]
    fn we_can_get_columns_of_table() {
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();

        let tab = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[8_i16, 2, 5, 1, 3, 7, 4])),
                (b.clone(), Column::Int(&[3_i32, 15, 9, 14, 15, 7, 4])),
                (c.clone(), Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let indexes = vec![1, 1, 0, 2, 2];
        let result = get_columns_of_table(&tab, &indexes).unwrap();
        assert_eq!(result[0], Column::Int(&[3_i32, 15, 9, 14, 15, 7, 4]));
        assert_eq!(result[1], Column::Int(&[3_i32, 15, 9, 14, 15, 7, 4]));
        assert_eq!(result[2], Column::SmallInt(&[8_i16, 2, 5, 1, 3, 7, 4]));
        assert_eq!(result[3], Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2]));
        assert_eq!(result[4], Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2]));
    }

    #[test]
    fn we_can_get_columns_of_table_with_empty_indexes() {
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();

        let tab = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[8_i16, 2, 5, 1, 3, 7, 4])),
                (b.clone(), Column::Int(&[3_i32, 15, 9, 14, 15, 7, 4])),
                (c.clone(), Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let indexes: Vec<usize> = vec![];
        let result = get_columns_of_table(&tab, &indexes).unwrap();
        assert!(
            result.is_empty(),
            "Empty indexes should return empty columns"
        );
    }

    #[test]
    fn we_can_get_columns_of_table_with_no_rows() {
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();

        let tab = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[0_i16; 0])),
                (b.clone(), Column::Int(&[0_i32; 0])),
                (c.clone(), Column::BigInt(&[0_i64; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let indexes: Vec<usize> = vec![];
        let result = get_columns_of_table(&tab, &indexes).unwrap();
        assert!(result.is_empty(), "Empty table should return empty columns");
    }

    #[test]
    fn we_can_get_columns_of_table_with_no_columns() {
        // 0 * 0 table
        let tab =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(0)))
                .expect("Table creation should not fail");
        let indexes: Vec<usize> = vec![];
        let result = get_columns_of_table(&tab, &indexes).unwrap();
        assert!(result.is_empty(), "Empty table should return empty columns");

        // 0 * 5 table
        let tab =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(5)))
                .expect("Table creation should not fail");
        let indexes: Vec<usize> = vec![];
        let result = get_columns_of_table(&tab, &indexes).unwrap();
        assert!(result.is_empty(), "Empty table should return empty columns");
    }

    #[test]
    fn we_cannot_get_columns_of_table_if_some_index_is_out_of_bound() {
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();

        let tab = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[8_i16, 2, 5, 1, 3, 7, 4])),
                (b.clone(), Column::Int(&[3_i32, 15, 9, 14, 15, 7, 4])),
                (c.clone(), Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let indexes = vec![1, 1, 0, 2, 3];
        let result = get_columns_of_table(&tab, &indexes);
        assert!(matches!(
            result,
            Err(TableOperationError::ColumnIndexOutOfBounds { .. })
        ));
    }

    // get_sort_merge_join_indexes
    #[test]
    fn we_can_get_sort_merge_join_indexes_two_tables() {
        let left_on = vec![Column::<TestScalar>::Int(&[3_i32, 5, 9, 4, 5, 7])];
        let right_on = vec![Column::<TestScalar>::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let row_indexes = get_sort_merge_join_indexes(&left_on, &right_on, 6, 7);
        assert_eq!(row_indexes, vec![(1, 3), (1, 4), (3, 5), (4, 3), (4, 4)]);
    }

    #[test]
    fn we_can_get_sort_merge_join_indexes_two_tables_with_empty_results() {
        let left_on = vec![Column::<TestScalar>::Int(&[3_i32, 15, 9, 14, 15, 7])];
        let right_on = vec![Column::<TestScalar>::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let row_indexes = get_sort_merge_join_indexes(&left_on, &right_on, 6, 7);
        assert!(row_indexes.is_empty());
    }

    #[test]
    fn we_can_get_sort_merge_join_indexes_tables_with_no_rows() {
        // Right table has no rows
        let left_on = vec![Column::<TestScalar>::Int(&[3_i32, 15, 9, 14, 15, 7])];
        let right_on = vec![Column::<TestScalar>::Int(&[0_i32; 0])];
        let row_indexes = get_sort_merge_join_indexes(&left_on, &right_on, 6, 0);
        assert!(row_indexes.is_empty());

        // Left table has no rows
        let left_on = vec![Column::<TestScalar>::Int(&[0_i32; 0])];
        let right_on = vec![Column::<TestScalar>::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let row_indexes = get_sort_merge_join_indexes(&left_on, &right_on, 0, 7);
        assert!(row_indexes.is_empty());

        // Both tables have no rows
        let left_on = vec![Column::<TestScalar>::Int(&[0_i32; 0])];
        let right_on = vec![Column::<TestScalar>::Int(&[0_i32; 0])];
        let row_indexes = get_sort_merge_join_indexes(&left_on, &right_on, 0, 0);
        assert!(row_indexes.is_empty());
    }

    #[test]
    fn we_can_apply_sort_merge_join_indexes_two_tables() {
        let bump = Bump::new();
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();

        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[8_i16, 2, 5, 1, 3, 7])),
                (b.clone(), Column::Int(&[3_i32, 5, 9, 4, 5, 7])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2])),
                (b.clone(), Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");

        let left_row_indexes = vec![3, 1, 1, 4, 4];
        let right_row_indexes = vec![5, 3, 4, 3, 4];

        let result = apply_sort_merge_join_indexes(
            &left,
            &right,
            &[1],
            &[1],
            &left_row_indexes,
            &right_row_indexes,
            &bump,
        )
        .unwrap();

        assert_eq!(result[0], Column::Int(&[4_i32, 5, 5, 5, 5]));
        assert_eq!(result[1], Column::SmallInt(&[1_i16, 2, 2, 3, 3]));
        assert_eq!(result[2], Column::BigInt(&[7_i64, 8, 9, 8, 9]));
    }

    #[test]
    fn we_can_apply_sort_merge_join_indexes_two_tables_with_empty_results() {
        let bump = Bump::new();
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();

        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[8_i16, 2, 5, 1, 3, 7])),
                (b.clone(), Column::Int(&[3_i32, 15, 9, 14, 15, 7])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2])),
                (b.clone(), Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");

        let left_row_indexes: Vec<usize> = vec![];
        let right_row_indexes: Vec<usize> = vec![];

        let result = apply_sort_merge_join_indexes(
            &left,
            &right,
            &[1],
            &[1],
            &left_row_indexes,
            &right_row_indexes,
            &bump,
        )
        .unwrap();
        assert_eq!(result[0], Column::Int(&[0_i32; 0]));
        assert_eq!(result[1], Column::SmallInt(&[0_i16; 0]));
        assert_eq!(result[2], Column::BigInt(&[0_i64; 0]));
    }

    #[test]
    fn we_can_apply_sort_merge_join_indexes_tables_with_no_rows() {
        let bump = Bump::new();
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();

        // Right table has no rows
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[8_i16, 2, 5, 1, 3, 7])),
                (b.clone(), Column::Int(&[3_i32, 15, 9, 14, 15, 7])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[0_i64; 0])),
                (b.clone(), Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let left_row_indexes: Vec<usize> = vec![];
        let right_row_indexes: Vec<usize> = vec![];
        let result = apply_sort_merge_join_indexes(
            &left,
            &right,
            &[1],
            &[1],
            &left_row_indexes,
            &right_row_indexes,
            &bump,
        )
        .unwrap();
        assert_eq!(result[0], Column::Int(&[0_i32; 0]));
        assert_eq!(result[1], Column::SmallInt(&[0_i16; 0]));
        assert_eq!(result[2], Column::BigInt(&[0_i64; 0]));

        // Left table has no rows
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[0_i16; 0])),
                (b.clone(), Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[1_i64, 2, 7, 8, 9, 7, 2])),
                (b.clone(), Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let left_row_indexes: Vec<usize> = vec![];
        let right_row_indexes: Vec<usize> = vec![];
        let result = apply_sort_merge_join_indexes(
            &left,
            &right,
            &[1],
            &[1],
            &left_row_indexes,
            &right_row_indexes,
            &bump,
        )
        .unwrap();
        assert_eq!(result[0], Column::Int(&[0_i32; 0]));
        assert_eq!(result[1], Column::SmallInt(&[0_i16; 0]));
        assert_eq!(result[2], Column::BigInt(&[0_i64; 0]));

        // Both tables have no rows
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a.clone(), Column::SmallInt(&[0_i16; 0])),
                (b.clone(), Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c.clone(), Column::BigInt(&[0_i64; 0])),
                (b.clone(), Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let left_row_indexes: Vec<usize> = vec![];
        let right_row_indexes: Vec<usize> = vec![];
        let result = apply_sort_merge_join_indexes(
            &left,
            &right,
            &[1],
            &[1],
            &left_row_indexes,
            &right_row_indexes,
            &bump,
        )
        .unwrap();
        assert_eq!(result[0], Column::Int(&[0_i32; 0]));
        assert_eq!(result[1], Column::SmallInt(&[0_i16; 0]));
        assert_eq!(result[2], Column::BigInt(&[0_i64; 0]));
    }
}
