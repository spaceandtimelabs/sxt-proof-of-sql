use super::{
    AddOp, ArithmeticOp, ColumnOperationError, ColumnOperationResult, DivOp, MulOp, SubOp,
};
use crate::base::{
    database::{
        slice_decimal_operation::{eq_decimal_columns, ge_decimal_columns, le_decimal_columns},
        slice_operation::{
            slice_and, slice_eq, slice_eq_with_casting, slice_ge, slice_ge_with_casting, slice_le,
            slice_le_with_casting, slice_not, slice_or,
        },
        OwnedColumn,
    },
    scalar::Scalar,
};
use alloc::string::ToString;
use core::ops::{Add, Div, Mul, Sub};

impl<S: Scalar> OwnedColumn<S> {
    /// Element-wise NOT operation for a column
    pub fn element_wise_not(&self) -> ColumnOperationResult<Self> {
        match self {
            Self::Boolean(values) => Ok(Self::Boolean(slice_not(values))),
            _ => Err(ColumnOperationError::UnaryOperationInvalidColumnType {
                operator: "NOT".to_string(),
                operand_type: self.column_type(),
            }),
        }
    }

    /// Element-wise AND for two columns
    pub fn element_wise_and(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(slice_and(lhs, rhs))),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "AND".to_string(),
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise OR for two columns
    pub fn element_wise_or(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(slice_or(lhs, rhs))),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "OR".to_string(),
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise equality check for two columns
    #[allow(clippy::too_many_lines)]
    pub fn element_wise_eq(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::SmallInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Int(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::Int(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::Int(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::Int(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::BigInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(lhs, rhs)))
            }
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Int128(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_eq_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::VarChar(lhs), Self::VarChar(rhs)) => Ok(Self::Boolean(slice_eq(lhs, rhs))),
            (Self::TimestampTZ(_, _, _), Self::TimestampTZ(_, _, _)) => {
                todo!("Implement equality check for TimeStampTZ")
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "=".to_string(),
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise <= check for two columns
    #[allow(clippy::too_many_lines)]
    pub fn element_wise_le(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(slice_le(lhs, rhs))),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(slice_le(lhs, rhs))),
            (Self::SmallInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Int(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::Int(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Boolean(slice_le(lhs, rhs))),
            (Self::Int(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::Int(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(slice_le(lhs, rhs))),
            (Self::BigInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(lhs, rhs)))
            }
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Int128(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(slice_le(lhs, rhs))),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(slice_le(lhs, rhs))),
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Ok(Self::Boolean(slice_le(lhs, rhs))),
            (Self::TimestampTZ(_, _, _), Self::TimestampTZ(_, _, _)) => {
                todo!("Implement inequality check for TimeStampTZ")
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "<=".to_string(),
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise >= check for two columns
    #[allow(clippy::too_many_lines)]
    pub fn element_wise_ge(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(slice_ge(lhs, rhs))),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(slice_ge(lhs, rhs))),
            (Self::SmallInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Int(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::Int(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Boolean(slice_ge(lhs, rhs))),
            (Self::Int(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::Int(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(slice_ge(lhs, rhs))),
            (Self::BigInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Boolean(slice_ge_with_casting(lhs, rhs)))
            }
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Int128(lhs), Self::TinyInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::Int(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Boolean(slice_le_with_casting(rhs, lhs)))
            }
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(slice_ge(lhs, rhs))),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                Ok(Self::Boolean(le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                )))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )))
            }
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(slice_ge(lhs, rhs))),
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Ok(Self::Boolean(slice_ge(lhs, rhs))),
            (Self::TimestampTZ(_, _, _), Self::TimestampTZ(_, _, _)) => {
                todo!("Implement inequality check for TimeStampTZ")
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: ">=".to_string(),
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise addition for two columns
    pub fn element_wise_add(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        AddOp::owned_column_element_wise_arithmetic(self, rhs)
    }

    /// Element-wise subtraction for two columns
    pub fn element_wise_sub(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        SubOp::owned_column_element_wise_arithmetic(self, rhs)
    }

    /// Element-wise multiplication for two columns
    pub fn element_wise_mul(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        MulOp::owned_column_element_wise_arithmetic(self, rhs)
    }

    /// Element-wise division for two columns
    pub fn element_wise_div(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        DivOp::owned_column_element_wise_arithmetic(self, rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};

    #[test]
    fn we_cannot_do_binary_operation_on_columns_with_different_lengths() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false]);

        let result = lhs.element_wise_and(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_eq(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_ge(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]);
        let result = lhs.clone() + rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]);
        let result = lhs.clone() + rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.clone() - rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.clone() * rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs / rhs;
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
    }

    #[test]
    fn we_cannot_do_logical_operation_on_nonboolean_columns() {
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_and(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_or(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_not();
        assert!(matches!(
            result,
            Err(ColumnOperationError::UnaryOperationInvalidColumnType { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let result = lhs.element_wise_and(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_or(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_not();
        assert!(matches!(
            result,
            Err(ColumnOperationError::UnaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_do_logical_operation_on_boolean_columns() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false]);
        let result = lhs.element_wise_and(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![
                true, false, false, false
            ]))
        );

        let result = lhs.element_wise_or(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![
                true, true, true, false
            ]))
        );

        let result = lhs.element_wise_not();
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![
                false, true, false, true
            ]))
        );
    }

    #[test]
    fn we_can_do_eq_operation() {
        // Integers
        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        // Strings
        let lhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );

        // Booleans
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, -3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        // Decimals and integers
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, -2, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 1, lhs_scalars);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );

        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, -2, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 1, lhs_scalars);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );
    }

    #[test]
    fn we_can_do_le_operation_on_numeric_and_boolean_columns() {
        // Booleans
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );

        // Integers
        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 24, -3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );

        // Decimals and integers
        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, true, true]))
        );

        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, true, true]))
        );
    }

    #[test]
    fn we_can_do_ge_operation_on_numeric_and_boolean_columns() {
        // Booleans
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );

        // Integers
        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 24, -3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );

        // Decimals and integers
        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );

        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );
    }

    #[test]
    fn we_cannot_do_comparison_on_columns_with_incompatible_types() {
        // Strings can't be compared with other types
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_ge(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        // Booleans can't be compared with other types
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        // Strings can not be <= or >= to each other
        let lhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_ge(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_do_arithmetic_on_nonnumeric_columns() {
        let lhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<TestScalar>::Scalar(vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ]);
        let result = lhs.clone() + rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.clone() - rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.clone() * rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs / rhs;
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_add_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::TinyInt(vec![2_i8, 4, 6]))
        );

        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1_i16, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1_i16, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::SmallInt(vec![2_i16, 4, 6]))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Int(vec![2_i32, 4, 6]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Int128(vec![2_i128, 4, 6]))
        );
    }

    #[test]
    fn we_can_decimal_op() {
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [2, 4, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(51).unwrap(), 3, rhs_scalars);
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [11, 22, 33].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(52).unwrap(), 3, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [101, 202, 303].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [101, 202, 303].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 2, expected_scalars)
        );
    }

    #[test]
    fn we_can_try_subtract_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::TinyInt(vec![3_i8, 3, -1]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![4_i32, 5, 2]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Int(vec![3_i32, 3, -1]))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, 2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 5]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::BigInt(vec![3_i64, 3, -3]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 5]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::BigInt(vec![2_i64, 0, -2]))
        );
    }

    #[test]
    fn we_can_try_subtract_decimal_columns() {
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [4, 5, 2].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [3, 3, -1].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [4, 5, 2].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(25).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(51).unwrap(), 3, rhs_scalars);
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [39, 48, 17].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(52).unwrap(), 3, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [399, 498, 197].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [399, 498, 197].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 2, expected_scalars)
        );
    }

    #[test]
    fn we_can_try_multiply_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 10, -6]))
        );

        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 3]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 10, -6]))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![3_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Int128(vec![3_i128, 4, 15]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Int128(vec![3_i128, 4, 15]))
        );
    }

    #[test]
    fn we_can_try_multiply_decimal_columns() {
        // lhs and rhs are both decimals
        let lhs_scalars = [4, 5, 2].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs_scalars = [-1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [-4, 10, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(11).unwrap(), 4, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [4, 10, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(9).unwrap(), 2, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [4, 10, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(16).unwrap(), 2, expected_scalars)
        );
    }

    #[test]
    fn we_can_try_divide_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 2, 0]))
        );

        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 3]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 2, 0]))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![3_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Int128(vec![3_i128, 1, 0]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Int128(vec![3_i128, 1, 0]))
        );
    }

    #[test]
    fn we_can_try_divide_decimal_columns() {
        // lhs and rhs are both decimals
        let lhs_scalars = [4, 5, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs_scalars = [-1, 2, 4].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs / rhs).unwrap();
        let expected_scalars = [-400_000_000_i128, 250_000_000, 75_000_000]
            .iter()
            .map(TestScalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 8, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4, 5, 3]);
        let rhs_scalars = [-1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(3).unwrap(), 2, rhs_scalars);
        let result = (lhs / rhs).unwrap();
        let expected_scalars = [-400_000_000, 250_000_000, 100_000_000]
            .iter()
            .map(TestScalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(11).unwrap(), 6, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![4, 5, 3]);
        let rhs_scalars = [-1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(3).unwrap(), 2, rhs_scalars);
        let result = (lhs / rhs).unwrap();
        let expected_scalars = [-400_000_000, 250_000_000, 100_000_000]
            .iter()
            .map(TestScalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 6, expected_scalars)
        );
    }
}
