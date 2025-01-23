use super::{
    apply_column_to_indexes,
    order_by_util::{compare_indexes_by_columns, compare_single_row_of_tables},
    union_util::column_union,
    Column, ColumnOperationResult, ColumnRepeatOp, ElementwiseRepeatOp, RepetitionOp, Table,
    TableOperationError, TableOperationResult, TableOptions,
};
use crate::base::{
    map::{IndexMap, IndexSet},
    scalar::Scalar,
};
use alloc::{vec, vec::Vec};
use bumpalo::Bump;
use core::cmp::Ordering;
use itertools::Itertools;
use sqlparser::ast::Ident;

/// Compute the set union of two slices of columns, deduplicate and sort the result.
///
/// Notes
/// 1. This is mostly used for joins.
/// 2. We do not check whether columns in the args have the same length, as we assume that the columns in an arg are already from the same table.
pub(crate) fn ordered_set_union<'a, S: Scalar>(
    left_on: &[Column<'a, S>],
    right_on: &[Column<'a, S>],
    alloc: &'a Bump,
) -> TableOperationResult<Vec<Column<'a, S>>> {
    //1. Union the columns
    if left_on.len() != right_on.len() {
        return Err(TableOperationError::JoinWithDifferentNumberOfColumns {
            left_num_columns: left_on.len(),
            right_num_columns: right_on.len(),
        });
    }
    if left_on.is_empty() {
        return Ok(Vec::new());
    }
    let raw_union = left_on
        .iter()
        .zip(right_on.iter())
        .map(|(left, right)| column_union(&[left, right], alloc, left.column_type()))
        .collect::<ColumnOperationResult<Vec<_>>>()?;
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
) -> Vec<u64> {
    // If unique is empty, the multiplicities vector is empty
    if unique.is_empty() {
        return Vec::new();
    }
    let num_unique_rows = unique[0].len();
    // If data is empty, all multiplicities are 0
    if data.is_empty() {
        return vec![0; num_unique_rows];
    }
    let num_rows = data[0].len();
    (0..num_unique_rows)
        .map(|unique_index| {
            (0..num_rows)
                .filter(|&data_index| {
                    compare_single_row_of_tables(data, unique, data_index, unique_index)
                        == Ok(Ordering::Equal)
                })
                .count() as u64
        })
        .collect::<Vec<_>>()
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

    #[test]
    fn we_can_do_ordered_set_union_fail_different_number_of_columns() {
        let alloc = Bump::new();

        // left has 2 columns, right has 1
        let left_on = vec![
            Column::<TestScalar>::Boolean(&[true, false]),
            Column::<TestScalar>::Int(&[1, 2]),
        ];
        let right_on = vec![Column::<TestScalar>::Boolean(&[true, false])];

        // We expect an error since they differ in number of columns
        let result = ordered_set_union(&left_on, &right_on, &alloc);
        assert!(matches!(
            result,
            Err(TableOperationError::JoinWithDifferentNumberOfColumns { .. })
        ));
    }

    /// Get Multiplicities
    #[test]
    fn we_can_get_multiplicities_empty_scenarios() {
        let empty_data: Vec<Column<TestScalar>> = vec![];
        let empty_unique: Vec<Column<TestScalar>> = vec![];

        // 1) Both 'data' and 'unique' empty
        let result = get_multiplicities(&empty_data, &empty_unique);
        assert!(
            result.is_empty(),
            "When both are empty, result should be empty"
        );

        // 2) 'unique' empty, 'data' non-empty
        let nonempty_data = vec![Column::<TestScalar>::Boolean(&[true, false])];
        let result = get_multiplicities(&nonempty_data, &empty_unique);
        assert!(
            result.is_empty(),
            "When 'unique' is empty, result must be empty"
        );

        // 3) 'unique' non-empty, 'data' empty => all zeros
        let nonempty_unique = vec![Column::<TestScalar>::Boolean(&[true, true, false])];
        let result = get_multiplicities(&empty_data, &nonempty_unique);
        assert_eq!(
            result,
            vec![0_u64; 3],
            "If data is empty, multiplicities should be zeros"
        );
    }

    #[test]
    fn we_can_get_multiplicities() {
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

        let result = get_multiplicities(&data, &unique);
        assert_eq!(result, vec![1, 0, 3, 1], "Expected multiplicities");
    }
}
