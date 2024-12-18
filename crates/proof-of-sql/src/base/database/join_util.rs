use super::{
    apply_column_to_indexes,
    order_by_util::{compare_indexes_by_columns, compare_single_row_of_tables},
    Column, ColumnRepeatOp, ElementwiseRepeatOp, RepetitionOp, Table, TableOperationError,
    TableOperationResult, TableOptions,
};
use crate::base::{
    map::{IndexMap, IndexSet},
    scalar::Scalar,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::cmp::Ordering;
use itertools::Itertools;
use sqlparser::ast::Ident;

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

/// This is the core of sort-merge joins.
///
/// # Panics
/// The function panics if we feed in incorrect data (e.g. Num of rows in `left` and some column of `left_on` being different).
fn get_sort_merge_join_indexes<'a, S: Scalar>(
    left_on: &'a [Column<'a, S>],
    right_on: &'a [Column<'a, S>],
    left_num_rows: usize,
    right_num_rows: usize,
) -> impl Iterator<Item = (usize, usize)> + 'a {
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
}

/// Compute the JOIN of two tables using a sort-merge join.
///
/// Currently we only support INNER JOINs and only support joins on equalities.
/// # Panics
/// The function panics if we feed in incorrect data (e.g. Num of rows in `left` and some column of `left_on` being different).
#[allow(clippy::needless_borrowed_reference)]
pub fn sort_merge_join<'a, S: Scalar>(
    left: &Table<'a, S>,
    right: &Table<'a, S>,
    left_on: &[Column<'a, S>],
    right_on: &[Column<'a, S>],
    left_selected_column_ident_aliases: &[(&Ident, &Ident)],
    right_selected_column_ident_aliases: &[(&Ident, &Ident)],
    alloc: &'a Bump,
) -> TableOperationResult<Table<'a, S>> {
    let left_num_rows = left.num_rows();
    let right_num_rows = right.num_rows();
    // Check that result aliases are unique
    let aliases = left_selected_column_ident_aliases
        .iter()
        .map(|(_, alias)| alias)
        .chain(
            right_selected_column_ident_aliases
                .iter()
                .map(|(_, alias)| alias),
        )
        .collect::<IndexSet<_>>();
    if aliases.len()
        != left_selected_column_ident_aliases.len() + right_selected_column_ident_aliases.len()
    {
        return Err(TableOperationError::DuplicateColumn);
    }
    // Find indexes of rows that match
    let index_pairs = get_sort_merge_join_indexes(left_on, right_on, left_num_rows, right_num_rows);
    // Now we have the indexes of the rows that match, we can create the new table
    let (left_indexes, right_indexes): (Vec<usize>, Vec<usize>) = index_pairs.into_iter().unzip();
    let num_rows = left_indexes.len();
    let result_columns = left_selected_column_ident_aliases
        .iter()
        .map(
            move |(&ref ident, &ref alias)| -> TableOperationResult<(Ident, Column<'a, S>)> {
                Ok((
                    alias.clone(),
                    apply_column_to_indexes(
                        left.inner_table().get(&ident.clone()).ok_or(
                            TableOperationError::ColumnDoesNotExist {
                                column_ident: ident.clone(),
                            },
                        )?,
                        alloc,
                        &left_indexes,
                    )?,
                ))
            },
        )
        .chain(right_selected_column_ident_aliases.iter().map(
            move |(&ref ident, &ref alias)| -> TableOperationResult<(Ident, Column<'a, S>)> {
                Ok((
                    alias.clone(),
                    apply_column_to_indexes(
                        right.inner_table().get(&ident.clone()).ok_or(
                            TableOperationError::ColumnDoesNotExist {
                                column_ident: ident.clone(),
                            },
                        )?,
                        alloc,
                        &right_indexes,
                    )?,
                ))
            },
        ))
        .collect::<TableOperationResult<IndexMap<_, _>>>()?;
    Ok(
        Table::<'a, S>::try_new_with_options(result_columns, TableOptions::new(Some(num_rows)))
            .expect("Table creation should not fail"),
    )
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

    #[test]
    fn we_can_do_sort_merge_join_on_two_tables() {
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
        let left_on = vec![Column::Int(&[3_i32, 5, 9, 4, 5, 7])];
        let right_on = vec![Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let left_selected_column_ident_aliases = vec![(&a, &a), (&b, &b)];
        let right_selected_column_ident_aliases = vec![(&c, &c)];
        let result = sort_merge_join(
            &left,
            &right,
            &left_on,
            &right_on,
            &left_selected_column_ident_aliases,
            &right_selected_column_ident_aliases,
            &bump,
        )
        .unwrap();
        assert_eq!(result.num_rows(), 5);
        assert_eq!(result.num_columns(), 3);
        assert_eq!(
            result.inner_table()[&a].as_smallint().unwrap(),
            &[1_i16, 2, 2, 3, 3]
        );
        assert_eq!(
            result.inner_table()[&b].as_int().unwrap(),
            &[4_i32, 5, 5, 5, 5]
        );
        assert_eq!(
            result.inner_table()[&c].as_bigint().unwrap(),
            &[7_i64, 8, 9, 8, 9]
        );
    }

    #[test]
    fn we_can_do_sort_merge_join_on_two_tables_with_empty_results() {
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
        let left_on = vec![Column::Int(&[3_i32, 15, 9, 14, 15, 7])];
        let right_on = vec![Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let left_selected_column_ident_aliases = vec![(&a, &a), (&b, &b)];
        let right_selected_column_ident_aliases = vec![(&c, &c)];
        let result = sort_merge_join(
            &left,
            &right,
            &left_on,
            &right_on,
            &left_selected_column_ident_aliases,
            &right_selected_column_ident_aliases,
            &bump,
        )
        .unwrap();
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 3);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);
        assert_eq!(result.inner_table()[&c].as_bigint().unwrap(), &[0_i64; 0]);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn we_can_do_sort_merge_join_on_tables_with_no_rows() {
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
        let left_on = vec![Column::Int(&[3_i32, 15, 9, 14, 15, 7])];
        let right_on = vec![Column::Int(&[0_i32; 0])];
        let left_selected_column_ident_aliases = vec![(&a, &a), (&b, &b)];
        let right_selected_column_ident_aliases = vec![(&c, &c)];
        let result = sort_merge_join(
            &left,
            &right,
            &left_on,
            &right_on,
            &left_selected_column_ident_aliases,
            &right_selected_column_ident_aliases,
            &bump,
        )
        .unwrap();
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 3);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);
        assert_eq!(result.inner_table()[&c].as_bigint().unwrap(), &[0_i64; 0]);

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
        let left_on = vec![Column::Int(&[0_i32; 0])];
        let right_on = vec![Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let left_selected_column_ident_aliases = vec![(&a, &a), (&b, &b)];
        let right_selected_column_ident_aliases = vec![(&c, &c)];
        let result = sort_merge_join(
            &left,
            &right,
            &left_on,
            &right_on,
            &left_selected_column_ident_aliases,
            &right_selected_column_ident_aliases,
            &bump,
        )
        .unwrap();
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 3);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);
        assert_eq!(result.inner_table()[&c].as_bigint().unwrap(), &[0_i64; 0]);

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
        let left_on = vec![Column::Int(&[0_i32; 0])];
        let right_on = vec![Column::Int(&[0_i32; 0])];
        let left_selected_column_ident_aliases = vec![(&a, &a), (&b, &b)];
        let right_selected_column_ident_aliases = vec![(&c, &c)];
        let result = sort_merge_join(
            &left,
            &right,
            &left_on,
            &right_on,
            &left_selected_column_ident_aliases,
            &right_selected_column_ident_aliases,
            &bump,
        )
        .unwrap();
        assert_eq!(result.num_rows(), 0);
        assert_eq!(result.num_columns(), 3);
        assert_eq!(result.inner_table()[&a].as_smallint().unwrap(), &[0_i16; 0]);
        assert_eq!(result.inner_table()[&b].as_int().unwrap(), &[0_i32; 0]);
        assert_eq!(result.inner_table()[&c].as_bigint().unwrap(), &[0_i64; 0]);
    }

    #[test]
    fn we_can_not_do_sort_merge_join_with_duplicate_aliases() {
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
        let left_on = vec![Column::Int(&[3_i32, 5, 9, 4, 5, 7])];
        let right_on = vec![Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let left_selected_column_ident_aliases = vec![(&a, &a), (&b, &b)];
        let right_selected_column_ident_aliases = vec![(&b, &b), (&c, &c)];
        let result = sort_merge_join(
            &left,
            &right,
            &left_on,
            &right_on,
            &left_selected_column_ident_aliases,
            &right_selected_column_ident_aliases,
            &bump,
        );
        assert_eq!(result, Err(TableOperationError::DuplicateColumn));
    }

    #[test]
    fn we_can_not_do_sort_merge_join_with_wrong_column_idents() {
        let bump = Bump::new();
        let a: Ident = "a".into();
        let b: Ident = "b".into();
        let c: Ident = "c".into();
        let not_a_column: Ident = "not_a_column".into();
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
        let left_on = vec![Column::Int(&[3_i32, 5, 9, 4, 5, 7])];
        let right_on = vec![Column::Int(&[10_i32, 11, 6, 5, 5, 4, 8])];
        let left_selected_column_ident_aliases = vec![(&a, &a), (&b, &b)];
        let right_selected_column_ident_aliases = vec![(&not_a_column, &c)];
        let result = sort_merge_join(
            &left,
            &right,
            &left_on,
            &right_on,
            &left_selected_column_ident_aliases,
            &right_selected_column_ident_aliases,
            &bump,
        );
        assert!(matches!(
            result,
            Err(TableOperationError::ColumnDoesNotExist { .. })
        ));
    }
}
