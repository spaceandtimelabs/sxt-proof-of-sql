use super::{ColumnNullability, ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::{
        column_operation::{
            eq_decimal_columns, ge_decimal_columns, le_decimal_columns, slice_and, slice_eq,
            slice_eq_with_casting, slice_ge, slice_ge_with_casting, slice_le,
            slice_le_with_casting, slice_not, slice_or, try_add_decimal_columns, try_add_slices,
            try_add_slices_with_casting, try_divide_decimal_columns, try_divide_slices,
            try_divide_slices_left_upcast, try_divide_slices_right_upcast,
            try_multiply_decimal_columns, try_multiply_slices, try_multiply_slices_with_casting,
            try_subtract_decimal_columns, try_subtract_slices, try_subtract_slices_left_upcast,
            try_subtract_slices_right_upcast,
        },
        OwnedColumn,
    },
    scalar::Scalar,
};
use core::ops::{Add, Div, Mul, Sub};
use proof_of_sql_parser::intermediate_ast::{BinaryOperator, UnaryOperator};

impl<S: Scalar> OwnedColumn<S> {
    /// Element-wise NOT operation for a column
    pub fn element_wise_not(&self) -> ColumnOperationResult<Self> {
        match self {
            Self::Boolean(_, values) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_not(values),
            )),
            _ => Err(ColumnOperationError::UnaryOperationInvalidColumnType {
                operator: UnaryOperator::Not,
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
            (Self::Boolean(meta, lhs), Self::Boolean(_, rhs)) => {
                Ok(Self::Boolean(*meta, slice_and(lhs, rhs)))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::And,
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
            (Self::Boolean(_, lhs), Self::Boolean(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_or(lhs, rhs),
            )),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Or,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise equality check for two columns
    pub fn element_wise_eq(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::SmallInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::SmallInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Int(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::Int(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::Int(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::Int(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::Int(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::Int(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::BigInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::BigInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(lhs, rhs),
            )),
            (Self::BigInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Int128(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::Int128(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Decimal75(.., lhs_values), Self::TinyInt(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::SmallInt(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::Int(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::BigInt(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::Int128(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                eq_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::Decimal75(.., rhs_values)) => {
                Ok(Self::Boolean(
                    ColumnNullability::NotNullable,
                    eq_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    ),
                ))
            }
            (Self::Boolean(_, lhs), Self::Boolean(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::Scalar(_, lhs), Self::Scalar(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::VarChar(_, lhs), Self::VarChar(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_eq(lhs, rhs),
            )),
            (Self::TimestampTZ(_, _, _, _), Self::TimestampTZ(_, _, _, _)) => {
                todo!("Implement equality check for TimeStampTZ")
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Equal,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise <= check for two columns
    pub fn element_wise_le(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::SmallInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(rhs, lhs),
            )),
            (Self::SmallInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Int(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(rhs, lhs),
            )),
            (Self::Int(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(rhs, lhs),
            )),
            (Self::Int(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le(lhs, rhs),
            )),
            (Self::Int(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::Int(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::Int(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::BigInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le(lhs, rhs),
            )),
            (Self::BigInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(lhs, rhs),
            )),
            (Self::BigInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Int128(_, lhs), Self::TinyInt(ColumnNullability::NotNullable, rhs)) => {
                Ok(Self::Boolean(
                    ColumnNullability::NotNullable,
                    slice_ge_with_casting(rhs, lhs),
                ))
            }
            (Self::Int128(_, lhs), Self::SmallInt(ColumnNullability::NotNullable, rhs)) => {
                Ok(Self::Boolean(
                    ColumnNullability::NotNullable,
                    slice_ge_with_casting(rhs, lhs),
                ))
            }
            (Self::Int128(_, lhs), Self::Int(ColumnNullability::NotNullable, rhs)) => {
                Ok(Self::Boolean(
                    ColumnNullability::NotNullable,
                    slice_ge_with_casting(rhs, lhs),
                ))
            }
            (Self::Int128(_, lhs), Self::BigInt(ColumnNullability::NotNullable, rhs)) => {
                Ok(Self::Boolean(
                    ColumnNullability::NotNullable,
                    slice_ge_with_casting(rhs, lhs),
                ))
            }
            (Self::Int128(_, lhs), Self::Int128(ColumnNullability::NotNullable, rhs)) => Ok(
                Self::Boolean(ColumnNullability::NotNullable, slice_le(lhs, rhs)),
            ),
            (Self::Int128(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (
                Self::Decimal75(.., lhs_values),
                Self::TinyInt(ColumnNullability::NotNullable, rhs_values),
            ) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (
                Self::Decimal75(.., lhs_values),
                Self::SmallInt(ColumnNullability::NotNullable, rhs_values),
            ) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (
                Self::Decimal75(.., lhs_values),
                Self::Int(ColumnNullability::NotNullable, rhs_values),
            ) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (
                Self::Decimal75(.., lhs_values),
                Self::BigInt(ColumnNullability::NotNullable, rhs_values),
            ) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (
                Self::Decimal75(.., lhs_values),
                Self::Int128(ColumnNullability::NotNullable, rhs_values),
            ) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::Decimal75(.., rhs_values)) => {
                Ok(Self::Boolean(
                    ColumnNullability::NotNullable,
                    le_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    ),
                ))
            }
            (Self::Boolean(_, lhs), Self::Boolean(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le(lhs, rhs),
            )),
            (Self::Scalar(_, lhs), Self::Scalar(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le(lhs, rhs),
            )),
            (Self::TimestampTZ(_, _, _, _), Self::TimestampTZ(_, _, _, _)) => {
                todo!("Implement inequality check for TimeStampTZ")
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::LessThanOrEqual,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise >= check for two columns
    pub fn element_wise_ge(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::TinyInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::SmallInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::SmallInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::SmallInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Int(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::Int(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::Int(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge(lhs, rhs),
            )),
            (Self::Int(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::Int(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::Int(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::BigInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::BigInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge(lhs, rhs),
            )),
            (Self::BigInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge_with_casting(lhs, rhs),
            )),
            (Self::BigInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Int128(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::Int(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_le_with_casting(rhs, lhs),
            )),
            (Self::Int128(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge(lhs, rhs),
            )),
            (Self::Int128(_, lhs_values), Self::Decimal75(.., rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                ge_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                ),
            )),

            (Self::Decimal75(.., lhs_values), Self::TinyInt(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::SmallInt(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::Int(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::BigInt(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::Int128(_, rhs_values)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                le_decimal_columns(
                    rhs_values,
                    lhs_values,
                    rhs.column_type(),
                    self.column_type(),
                ),
            )),
            (Self::Decimal75(.., lhs_values), Self::Decimal75(.., rhs_values)) => {
                Ok(Self::Boolean(
                    ColumnNullability::NotNullable,
                    ge_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    ),
                ))
            }
            (Self::Boolean(_, lhs), Self::Boolean(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge(lhs, rhs),
            )),
            (Self::Scalar(_, lhs), Self::Scalar(_, rhs)) => Ok(Self::Boolean(
                ColumnNullability::NotNullable,
                slice_ge(lhs, rhs),
            )),
            (Self::TimestampTZ(_, _, _, _), Self::TimestampTZ(_, _, _, _)) => {
                todo!("Implement inequality check for TimeStampTZ")
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::GreaterThanOrEqual,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

impl<S: Scalar> Add for OwnedColumn<S> {
    type Output = ColumnOperationResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let meta = ColumnNullability::NotNullable;
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (&self, &rhs) {
            (Self::TinyInt(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::TinyInt(meta, try_add_slices(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::SmallInt(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::SmallInt(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::SmallInt(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::SmallInt(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::SmallInt(meta, try_add_slices(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::Int(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::Int(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_add_slices(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::Int(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::BigInt(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::BigInt(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::BigInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::BigInt(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::BigInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_add_slices(lhs, rhs)?))
            }
            (Self::BigInt(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::BigInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int128(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_add_slices(lhs, rhs)?))
            }
            (Self::Int128(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Decimal75(.., lhs_values), Self::TinyInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::SmallInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::BigInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int128(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Add,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

impl<S: Scalar> Sub for OwnedColumn<S> {
    type Output = ColumnOperationResult<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        let meta = ColumnNullability::NotNullable;
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (&self, &rhs) {
            (Self::TinyInt(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::TinyInt(meta, try_subtract_slices(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::SmallInt(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::TinyInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::TinyInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::TinyInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::SmallInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::SmallInt(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::SmallInt(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::SmallInt(meta, try_subtract_slices(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::SmallInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::SmallInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::Int(meta, try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::Int(meta, try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_subtract_slices(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::Int(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::Int(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::BigInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_subtract_slices(lhs, rhs)?))
            }
            (Self::BigInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int128(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::Int(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_subtract_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_subtract_slices(lhs, rhs)?))
            }
            (Self::Int128(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Decimal75(.., lhs_values), Self::TinyInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::SmallInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::BigInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int128(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Subtract,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

impl<S: Scalar> Mul for OwnedColumn<S> {
    type Output = ColumnOperationResult<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        let meta = ColumnNullability::NotNullable;
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (&self, &rhs) {
            (Self::TinyInt(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::TinyInt(meta, try_multiply_slices(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::SmallInt(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::TinyInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::TinyInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::TinyInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::SmallInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::SmallInt(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::SmallInt(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::SmallInt(meta, try_multiply_slices(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::SmallInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::SmallInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::Int(meta, try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::Int(meta, try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_multiply_slices(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::Int(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::Int(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::BigInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::BigInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::BigInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::BigInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_multiply_slices(lhs, rhs)?))
            }
            (Self::BigInt(_, lhs), Self::Int128(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int128(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::Int128(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::Int128(_, lhs), Self::Int(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::Int128(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_multiply_slices_with_casting(rhs, lhs)?,
            )),
            (Self::Int128(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_multiply_slices(lhs, rhs)?))
            }
            (Self::Int128(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Decimal75(.., lhs_values), Self::TinyInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::SmallInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::BigInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int128(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Multiply,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

impl<S: Scalar> Div for OwnedColumn<S> {
    type Output = ColumnOperationResult<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        let meta = ColumnNullability::NotNullable;
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (&self, &rhs) {
            (Self::TinyInt(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::TinyInt(meta, try_divide_slices(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::SmallInt(
                meta,
                try_divide_slices_left_upcast(lhs, rhs)?,
            )),
            (Self::TinyInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::TinyInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::SmallInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::SmallInt(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::SmallInt(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::SmallInt(meta, try_divide_slices(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::SmallInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int(_, lhs), Self::TinyInt(_, rhs)) => {
                Ok(Self::Int(meta, try_divide_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::SmallInt(_, rhs)) => {
                Ok(Self::Int(meta, try_divide_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::Int(_, rhs)) => {
                Ok(Self::Int(meta, try_divide_slices(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::Int(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::Int(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::BigInt(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs), Self::Int(_, rhs)) => Ok(Self::BigInt(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::BigInt(_, lhs), Self::BigInt(_, rhs)) => {
                Ok(Self::BigInt(meta, try_divide_slices(lhs, rhs)?))
            }
            (Self::BigInt(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_divide_slices_left_upcast(lhs, rhs)?))
            }
            (Self::BigInt(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Int128(_, lhs), Self::TinyInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::SmallInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::Int(_, rhs)) => Ok(Self::Int128(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::BigInt(_, rhs)) => Ok(Self::Int128(
                meta,
                try_divide_slices_right_upcast(lhs, rhs)?,
            )),
            (Self::Int128(_, lhs), Self::Int128(_, rhs)) => {
                Ok(Self::Int128(meta, try_divide_slices(lhs, rhs)?))
            }
            (Self::Int128(_, lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }

            (Self::Decimal75(.., lhs_values), Self::TinyInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::SmallInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::BigInt(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Int128(_, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            (Self::Decimal75(.., lhs_values), Self::Decimal75(.., rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_divide_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(meta, new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Division,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::Curve25519Scalar};

    #[test]
    fn we_cannot_do_binary_operation_on_columns_with_different_lengths() {
        let meta = ColumnNullability::NotNullable;
        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false, true]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false]);

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

        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2]);
        let result = lhs.clone() + rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 2]);
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
        let meta = ColumnNullability::NotNullable;
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
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

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 2, 3]);
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
        let meta = ColumnNullability::NotNullable;
        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false, true, false]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, true, false, false]);
        let result = lhs.element_wise_and(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, false, false]
            ))
        );

        let result = lhs.element_wise_or(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, true, false]
            ))
        );

        let result = lhs.element_wise_not();
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![false, true, false, true]
            ))
        );
    }

    #[test]
    fn we_can_do_eq_operation() {
        let meta = ColumnNullability::NotNullable;
        // Integers
        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 3, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, false]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 3, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, false]
            ))
        );

        // Strings
        let lhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            meta,
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            meta,
            ["Space", "and", "time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, false]
            ))
        );

        // Booleans
        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false, true]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, true, false]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, false]
            ))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, -3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            3,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, false]
            ))
        );

        // Decimals and integers
        let lhs_scalars = [10, 2, 30].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, -2, 3]);
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            1,
            lhs_scalars,
        );
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, true]
            ))
        );

        let lhs_scalars = [10, 2, 30].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, -2, 3]);
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            1,
            lhs_scalars,
        );
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, true]
            ))
        );
    }

    #[test]
    fn we_can_do_le_operation_on_numeric_and_boolean_columns() {
        let meta = ColumnNullability::NotNullable;
        // Booleans
        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false, true]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, true, false]);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, false]
            ))
        );

        // Integers
        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 3, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, true]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 3, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, true]
            ))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 24, -3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            3,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, false]
            ))
        );

        // Decimals and integers
        let lhs_scalars = [10, -2, -30].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, -20, 3]);
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            -1,
            lhs_scalars,
        );
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![false, true, true]
            ))
        );

        let lhs_scalars = [10, -2, -30].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, -20, 3]);
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            -1,
            lhs_scalars,
        );
        let result = lhs.element_wise_le(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![false, true, true]
            ))
        );
    }

    #[test]
    fn we_can_do_ge_operation_on_numeric_and_boolean_columns() {
        let meta = ColumnNullability::NotNullable;
        // Booleans
        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false, true]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, true, false]);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, true]
            ))
        );

        // Integers
        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 3, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, false]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 3, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, false]
            ))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 24, -3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            3,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, false, true]
            ))
        );

        // Decimals and integers
        let lhs_scalars = [10, -2, -30].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1_i8, -20, 3]);
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            -1,
            lhs_scalars,
        );
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, false]
            ))
        );

        let lhs_scalars = [10, -2, -30].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(meta, vec![1_i64, -20, 3]);
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            -1,
            lhs_scalars,
        );
        let result = lhs.element_wise_ge(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Boolean(
                meta,
                vec![true, true, false]
            ))
        );
    }

    #[test]
    fn we_cannot_do_comparison_on_columns_with_incompatible_types() {
        let meta = ColumnNullability::NotNullable;
        // Strings can't be compared with other types
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            meta,
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

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            meta,
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
        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false, true]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(meta, vec![true, false, true]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 2, 3]);
        let result = lhs.element_wise_le(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        // Strings can not be <= or >= to each other
        let lhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            meta,
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            meta,
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
        let meta = ColumnNullability::NotNullable;
        let lhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            meta,
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Scalar(
            meta,
            vec![
                Curve25519Scalar::from(1),
                Curve25519Scalar::from(2),
                Curve25519Scalar::from(3),
            ],
        );
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
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1_i8, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1_i8, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::TinyInt(
                meta,
                vec![2_i8, 4, 6]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1_i16, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![1_i16, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::SmallInt(
                meta,
                vec![2_i16, 4, 6]
            ))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1_i8, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1_i32, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int(
                meta,
                vec![2_i32, 4, 6]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int128(meta, vec![1_i128, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1_i32, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int128(
                meta,
                vec![2_i128, 4, 6]
            ))
        );
    }

    #[test]
    fn we_can_try_add_decimal_columns() {
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [2, 4, 6].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(6).unwrap(),
                2,
                expected_scalars
            )
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(51).unwrap(),
            3,
            rhs_scalars,
        );
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [11, 22, 33].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(52).unwrap(),
                3,
                expected_scalars
            )
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1, 2, 3]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [101, 202, 303].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(6).unwrap(),
                2,
                expected_scalars
            )
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1, 2, 3]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [101, 202, 303].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(13).unwrap(),
                2,
                expected_scalars
            )
        );
    }

    #[test]
    fn we_can_try_subtract_integer_columns() {
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![4_i8, 5, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1_i8, 2, 3]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::TinyInt(
                meta,
                vec![3_i8, 3, -1]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![4_i32, 5, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![1_i32, 2, 3]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int(
                meta,
                vec![3_i32, 3, -1]
            ))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![4_i8, 5, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(meta, vec![1_i64, 2, 5]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::BigInt(
                meta,
                vec![3_i64, 3, -3]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(meta, vec![1_i64, 2, 5]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::BigInt(
                meta,
                vec![2_i64, 0, -2]
            ))
        );
    }

    #[test]
    fn we_can_try_subtract_decimal_columns() {
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [4, 5, 2].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [3, 3, -1].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(6).unwrap(),
                2,
                expected_scalars
            )
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [4, 5, 2].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(25).unwrap(),
            2,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(51).unwrap(),
            3,
            rhs_scalars,
        );
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [39, 48, 17].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(52).unwrap(),
                3,
                expected_scalars
            )
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [399, 498, 197].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(6).unwrap(),
                2,
                expected_scalars
            )
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [399, 498, 197].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(13).unwrap(),
                2,
                expected_scalars
            )
        );
    }

    #[test]
    fn we_can_try_multiply_integer_columns() {
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![4_i8, 5, -2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1_i8, 2, 3]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::TinyInt(
                meta,
                vec![4_i8, 10, -6]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::BigInt(meta, vec![4_i64, 5, -2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(meta, vec![1_i64, 2, 3]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::BigInt(
                meta,
                vec![4_i64, 10, -6]
            ))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![3_i8, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int128(meta, vec![1_i128, 2, 5]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int128(
                meta,
                vec![3_i128, 4, 15]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int128(meta, vec![1_i128, 2, 5]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int128(
                meta,
                vec![3_i128, 4, 15]
            ))
        );
    }

    #[test]
    fn we_can_try_multiply_decimal_columns() {
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs are both decimals
        let lhs_scalars = [4, 5, 2].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            lhs_scalars,
        );
        let rhs_scalars = [-1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [-4, 10, 6].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(11).unwrap(),
                4,
                expected_scalars
            )
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [4, 10, 6].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(9).unwrap(),
                2,
                expected_scalars
            )
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [4, 10, 6].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(16).unwrap(),
                2,
                expected_scalars
            )
        );
    }

    #[test]
    fn we_can_try_divide_integer_columns() {
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![4_i8, 5, -2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![1_i8, 2, 3]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::TinyInt(
                meta,
                vec![4_i8, 2, 0]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::BigInt(meta, vec![4_i64, 5, -2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(meta, vec![1_i64, 2, 3]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::BigInt(
                meta,
                vec![4_i64, 2, 0]
            ))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![3_i8, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int128(meta, vec![1_i128, 2, 5]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int128(
                meta,
                vec![3_i128, 1, 0]
            ))
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::Int(meta, vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int128(meta, vec![1_i128, 2, 5]);
        let result = lhs / rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int128(
                meta,
                vec![3_i128, 1, 0]
            ))
        );
    }

    #[test]
    fn we_can_try_divide_decimal_columns() {
        let meta = ColumnNullability::NotNullable;
        // lhs and rhs are both decimals
        let lhs_scalars = [4, 5, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            lhs_scalars,
        );
        let rhs_scalars = [-1, 2, 4].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(5).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs / rhs).unwrap();
        let expected_scalars = [-400_000_000_i128, 250_000_000, 75_000_000]
            .iter()
            .map(Curve25519Scalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(13).unwrap(),
                8,
                expected_scalars
            )
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(meta, vec![4, 5, 3]);
        let rhs_scalars = [-1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(3).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs / rhs).unwrap();
        let expected_scalars = [-400_000_000, 250_000_000, 100_000_000]
            .iter()
            .map(Curve25519Scalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(11).unwrap(),
                6,
                expected_scalars
            )
        );

        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(meta, vec![4, 5, 3]);
        let rhs_scalars = [-1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs = OwnedColumn::<Curve25519Scalar>::Decimal75(
            meta,
            Precision::new(3).unwrap(),
            2,
            rhs_scalars,
        );
        let result = (lhs / rhs).unwrap();
        let expected_scalars = [-400_000_000, 250_000_000, 100_000_000]
            .iter()
            .map(Curve25519Scalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                meta,
                Precision::new(13).unwrap(),
                6,
                expected_scalars
            )
        );
    }
}
