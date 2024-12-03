use super::{ColumnRepeatOp, ElementwiseRepeatOp, RepetitionOp, Table, TableOptions};
use crate::base::{if_rayon, scalar::Scalar};
use bumpalo::Bump;
use itertools::Itertools;

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
            .map(|(&ident, column)| {
                (
                    ident,
                    ColumnRepeatOp::column_op(column, alloc, right_num_rows),
                )
            })
            .chain(right.inner_table().iter().map(|(&ident, column)| {
                (
                    ident,
                    ElementwiseRepeatOp::column_op(column, alloc, left_num_rows),
                )
            })),
        TableOptions::new(Some(product_num_rows)),
    )
    .expect("Table creation should not fail")
}

/// Compute the JOIN of two tables using a sort-merge join.
///
/// Currently we only support INNER JOINs and only support joins on equalities.
/// # Panics
/// The function panics if we feed in incorrect data (e.g. Num of rows in `left` and some column of `left_on` being different).
pub fn sort_merge_join<'a, S: Scalar>(
    left: &Table<'a, S>,
    right: &Table<'a, S>,
    left_on: &[Column<'a, S>],
    right_on: &[Column<'a, S>],
    alloc: &'a Bump,
) -> Table<'a, S> {
    let left_num_rows = left.num_rows();
    let right_num_rows = right.num_rows();
    // Check that the number of rows is good
    for column in left_on.iter() {
        assert_eq!(column.len(), left_num_rows);
    }
    for column in right_on.iter() {
        assert_eq!(column.len(), right_num_rows);
    }
    // First of all sort the tables by the columns we are joining on
    let left_indexes = if_rayon!(
        (0..left.num_rows()).par_sort_unstable_by(|&a, &b| compare_indexes_by_columns(
            left_join_cols,
            a,
            b
        )),
        (0..left.num_rows())..sort_unstable_by(|&a, &b| compare_indexes_by_columns(
            left_join_cols
            a,
            b
        ))
    );
    let right_indexes = if_rayon!(
        (0..right.num_rows()).par_sort_unstable_by(|&a, &b| compare_indexes_by_columns(
            right_join_cols,
            a,
            b
        )),
        (0..right.num_rows())..sort_unstable_by(|&a, &b| compare_indexes_by_columns(
            right_join_cols
            a,
            b
        ))
    );
    // Collect the indexes of the rows that match
    let mut left_iter = left_indexes.into_iter().peekable();
    let mut right_iter = right_indexes.into_iter().peekable();
    let mut index_pairs = Vec::<(usize, usize)>::new();
    while let (Some(&left_index), Some(&right_index)) = (left_iter.peek(), right_iter.peek()) {
        match compare_indexes_of_tables_by_columns(
            left_join_cols,
            right_join_cols,
            left_index,
            right_index,
        ) {
            Ordering::Less => {
                left_iter.next();
            }
            Ordering::Greater => {
                right_iter.next();
            }
            Ordering::Equal => {
                // Collect all matching indexes from the left table
                let left_group: Vec<_> = left_iter
                    .by_ref()
                    .take_while(|item| {
                        compare_indexes_by_columns(left_join_cols, left_index, item)
                            == Ordering::Equal
                    })
                    .collect();
                // Collect all matching indexes from the right table
                let right_group: Vec<_> = right_iter
                    .by_ref()
                    .take_while(|item| {
                        compare_indexes_by_columns(right_join_cols, right_index, item)
                            == Ordering::Equal
                    })
                    .collect();
                // Collect indexes
                let matched_index_pairs = left_group.iter().cartesian_product(right_group.iter());
                index_pairs.extend(matched_index_pairs);
            }
        }
    }
    // Now we have the indexes of the rows that match, we can create the new table
    let (left_indexes, right_indexes) = index_pairs.iter().copied().unzip();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{database::Column, scalar::test_scalar::TestScalar};

    #[test]
    fn we_can_do_cross_joins() {
        let bump = Bump::new();
        let a = "a".parse().unwrap();
        let b = "b".parse().unwrap();
        let c = "c".parse().unwrap();
        let d = "d".parse().unwrap();
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a, Column::SmallInt(&[1_i16, 2, 3])),
                (b, Column::Int(&[4_i32, 5, 6])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c, Column::BigInt(&[7_i64, 8, 9])),
                (d, Column::Int128(&[10_i128, 11, 12])),
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
        let a = "a".parse().unwrap();
        let b = "b".parse().unwrap();
        let c = "c".parse().unwrap();
        let d = "d".parse().unwrap();

        // Right table has no rows
        let left = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (a, Column::SmallInt(&[1_i16, 2, 3])),
                (b, Column::Int(&[4_i32, 5, 6])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c, Column::BigInt(&[0_i64; 0])),
                (d, Column::Int128(&[0_i128; 0])),
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
                (a, Column::SmallInt(&[0_i16; 0])),
                (b, Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c, Column::BigInt(&[7_i64, 8, 9])),
                (d, Column::Int128(&[10_i128, 11, 12])),
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
                (a, Column::SmallInt(&[0_i16; 0])),
                (b, Column::Int(&[0_i32; 0])),
            ],
            TableOptions::default(),
        )
        .expect("Table creation should not fail");
        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c, Column::BigInt(&[0_i64; 0])),
                (d, Column::Int128(&[0_i128; 0])),
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
        let a = "a".parse().unwrap();
        let b = "b".parse().unwrap();
        let c = "c".parse().unwrap();
        let d = "d".parse().unwrap();
        let left =
            Table::<'_, TestScalar>::try_from_iter_with_options(vec![], TableOptions::new(Some(2)))
                .expect("Table creation should not fail");

        let right = Table::<'_, TestScalar>::try_from_iter_with_options(
            vec![
                (c, Column::BigInt(&[7_i64, 8])),
                (d, Column::Int128(&[10_i128, 11])),
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
                (a, Column::SmallInt(&[1_i16, 2])),
                (b, Column::Int(&[4_i32, 5])),
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
}
