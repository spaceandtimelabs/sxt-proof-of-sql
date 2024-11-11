use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::{
        slice_decimal_operation::{eq_decimal_columns, ge_decimal_columns, le_decimal_columns},
        slice_operation::{
            slice_binary_op, slice_binary_op_left_upcast, slice_binary_op_right_upcast,
        },
        ColumnType, OwnedColumn,
    },
    scalar::Scalar,
};
use core::{cmp::Ord, fmt::Debug};
use num_traits::Zero;
use proof_of_sql_parser::intermediate_ast::BinaryOperator;

pub trait ComparisonOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + Ord;

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>;

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>;

    /// Return an error if op is not implemented for string
    fn string_op(lhs: &[String], rhs: &[String]) -> ColumnOperationResult<Vec<bool>>;

    #[allow(clippy::too_many_lines)]
    fn owned_column_element_wise_comparison<S: Scalar>(
        lhs: &OwnedColumn<S>,
        rhs: &OwnedColumn<S>,
    ) -> ColumnOperationResult<OwnedColumn<S>> {
        if lhs.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: lhs.len(),
                len_b: rhs.len(),
            });
        }
        let result = match (&lhs, &rhs) {
            (OwnedColumn::TinyInt(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::SmallInt(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::Int(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::BigInt(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::Int128(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::TinyInt(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::SmallInt(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::Int(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::BigInt(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::Int128(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (
                OwnedColumn::Decimal75(_, _, lhs_values),
                OwnedColumn::Decimal75(_, _, rhs_values),
            ) => Ok(Self::decimal_op_left_upcast(
                lhs_values,
                rhs_values,
                lhs.column_type(),
                rhs.column_type(),
            )),

            (OwnedColumn::VarChar(lhs), OwnedColumn::VarChar(rhs)) => Self::string_op(lhs, rhs),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Add,
                left_type: lhs.column_type(),
                right_type: rhs.column_type(),
            }),
        }?;
        Ok(OwnedColumn::Boolean(result))
    }
}

pub struct EqualOp {}
impl ComparisonOp for EqualOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + PartialEq,
    {
        l == r
    }

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + PartialEq + Zero + Into<S>,
    {
        eq_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + PartialEq + Zero + Into<S>,
    {
        eq_decimal_columns(rhs, lhs, right_column_type, left_column_type)
    }

    fn string_op(lhs: &[String], rhs: &[String]) -> ColumnOperationResult<Vec<bool>> {
        Ok(lhs.iter().zip(rhs.iter()).map(|(l, r)| l == r).collect())
    }
}

pub struct GreaterThanOrEqualOp {}
impl ComparisonOp for GreaterThanOrEqualOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + Ord,
    {
        l >= r
    }

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        ge_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        le_decimal_columns(rhs, lhs, right_column_type, left_column_type)
    }

    fn string_op(_lhs: &[String], _rhs: &[String]) -> ColumnOperationResult<Vec<bool>> {
        Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: BinaryOperator::Add,
            left_type: ColumnType::VarChar,
            right_type: ColumnType::VarChar,
        })
    }
}

pub struct LessThanOrEqualOp {}
impl ComparisonOp for LessThanOrEqualOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + Ord,
    {
        l <= r
    }

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        le_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        ge_decimal_columns(rhs, lhs, right_column_type, left_column_type)
    }

    fn string_op(_lhs: &[String], _rhs: &[String]) -> ColumnOperationResult<Vec<bool>> {
        Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: BinaryOperator::Add,
            left_type: ColumnType::VarChar,
            right_type: ColumnType::VarChar,
        })
    }
}
