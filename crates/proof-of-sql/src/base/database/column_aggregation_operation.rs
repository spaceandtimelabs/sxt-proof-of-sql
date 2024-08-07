#![allow(dead_code)]
use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::ColumnType,
    math::decimal::{scale_scalar, DecimalError, Precision, MAX_SUPPORTED_PRECISION},
    scalar::Scalar,
};
use core::{cmp::Ordering, fmt::Debug, iter::Sum};
use num_bigint::BigInt;

use proof_of_sql_parser::intermediate_ast::AggregationOperator;

/// Count the number of rows in a column broken down into groups.
pub(crate) fn count_column<T>(
    last_indices_of_groups: &[usize],
) -> Vec<i64> {
    last_indices_of_groups
}

/// Sum the values in a column.
///
/// Assume that the column is sorted by the group columns.
pub(crate) fn sum_column<T>(
    column: &[T],
    group_sizes: &[usize],
) -> Vec<T>
where
    T: Sum,
{
    column.iter().sum()
}

/// Max the values in a column.