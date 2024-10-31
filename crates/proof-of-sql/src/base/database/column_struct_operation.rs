use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::{
        column_operation::{
            eq_decimal_columns, ge_decimal_columns, le_decimal_columns, slice_and, slice_eq,
            slice_eq_with_casting, slice_ge, slice_ge_with_casting, slice_le,
            slice_le_with_casting, slice_not, slice_or, try_add_decimal_columns, try_add_slices,
            try_add_slices_with_casting, try_multiply_decimal_columns, try_multiply_slices,
            try_multiply_slices_with_casting, try_subtract_decimal_columns, try_subtract_slices,
            try_subtract_slices_left_upcast, try_subtract_slices_right_upcast,
        },
        Column,
    },
    scalar::Scalar,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::cmp::Ordering;
use proof_of_sql_parser::{
    intermediate_ast::{BinaryOperator, UnaryOperator},
    posql_time::PoSQLTimeUnit,
};

/// Compare two [`PoSQLTimeStamp`]s using the provided comparison operator [`op`]
/// # Panics
/// Panics if the two data slices are not of the same length
#[allow(clippy::cast_sign_loss)]
fn cmp_timestamps(
    lhs_tu: PoSQLTimeUnit,
    lhs_data: &[i64],
    rhs_tu: PoSQLTimeUnit,
    rhs_data: &[i64],
    op: fn(i64, i64) -> bool,
) -> Vec<bool> {
    assert_eq!(lhs_data.len(), rhs_data.len());
    let lhs_scale: i8 = lhs_tu.into();
    let rhs_scale: i8 = rhs_tu.into();
    match lhs_scale.cmp(&rhs_scale) {
        Ordering::Less => {
            let scaling_factor = 10i64.pow((rhs_scale - lhs_scale) as u32);
            lhs_data
                .iter()
                .zip(rhs_data.iter())
                .map(|(lhs, rhs)| op(lhs * scaling_factor, *rhs))
                .collect()
        }
        Ordering::Greater => {
            let scaling_factor = 10i64.pow((lhs_scale - rhs_scale) as u32);
            lhs_data
                .iter()
                .zip(rhs_data.iter())
                .map(|(lhs, rhs)| op(*lhs, rhs * scaling_factor))
                .collect()
        }
        Ordering::Equal => lhs_data
            .iter()
            .zip(rhs_data.iter())
            .map(|(lhs, rhs)| op(*lhs, *rhs))
            .collect(),
    }
}

impl<'a, S: Scalar> Column<'a, S> {
    /// Element-wise NOT operation for a column
    pub fn element_wise_not(&self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        match self {
            Self::Boolean(values) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_not(values).as_ref()),
            )),
            _ => Err(ColumnOperationError::UnaryOperationInvalidColumnType {
                operator: UnaryOperator::Not,
                operand_type: self.column_type(),
            }),
        }
    }

    /// Element-wise AND for two columns
    pub fn element_wise_and(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_and(lhs, rhs).as_ref()),
            )),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::And,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise OR for two columns
    pub fn element_wise_or(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_or(lhs, rhs).as_ref()),
            )),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Or,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise equality check for two columns
    #[allow(clippy::too_many_lines)]
    pub fn element_wise_eq(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Int(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Int128(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    eq_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(
                    alloc.alloc_slice_copy(
                        eq_decimal_columns(
                            lhs_values,
                            rhs_values,
                            self.column_type(),
                            rhs.column_type(),
                        )
                        .as_ref(),
                    ),
                ))
            }
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::VarChar((lhs, _)), Self::VarChar((rhs, _))) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_eq(lhs, rhs).as_ref()),
            )),
            (Self::TimestampTZ(lhs_tu, _, lhs_raw), Self::TimestampTZ(rhs_tu, _, rhu_raw)) => {
                let result = cmp_timestamps(*lhs_tu, lhs_raw, *rhs_tu, rhu_raw, |a, b| a == b);
                Ok(Self::Boolean(alloc.alloc_slice_copy(result.as_ref())))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Equal,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise <= check for two columns
    #[allow(clippy::too_many_lines)]
    pub fn element_wise_le(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Int(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le(lhs, rhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Int128(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le(lhs, rhs).as_ref()),
            )),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(
                    alloc.alloc_slice_copy(
                        le_decimal_columns(
                            lhs_values,
                            rhs_values,
                            self.column_type(),
                            rhs.column_type(),
                        )
                        .as_ref(),
                    ),
                ))
            }
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le(lhs, rhs).as_ref()),
            )),
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le(lhs, rhs).as_ref()),
            )),
            (Self::TimestampTZ(lhs_tu, _, lhs_raw), Self::TimestampTZ(rhs_tu, _, rhu_raw)) => {
                let result = cmp_timestamps(*lhs_tu, lhs_raw, *rhs_tu, rhu_raw, |a, b| a <= b);
                Ok(Self::Boolean(alloc.alloc_slice_copy(result.as_ref())))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::LessThanOrEqual,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise >= check for two columns
    #[allow(clippy::too_many_lines)]
    pub fn element_wise_ge(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Int(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge(lhs, rhs).as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge_with_casting(lhs, rhs).as_ref()),
            )),
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Int128(lhs), Self::TinyInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::SmallInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::Int(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::BigInt(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_le_with_casting(rhs, lhs).as_ref()),
            )),
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge(lhs, rhs).as_ref()),
            )),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    ge_decimal_columns(
                        lhs_values,
                        rhs_values,
                        self.column_type(),
                        rhs.column_type(),
                    )
                    .as_ref(),
                ),
            )),

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(
                    le_decimal_columns(
                        rhs_values,
                        lhs_values,
                        rhs.column_type(),
                        self.column_type(),
                    )
                    .as_ref(),
                ),
            )),
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                Ok(Self::Boolean(
                    alloc.alloc_slice_copy(
                        ge_decimal_columns(
                            lhs_values,
                            rhs_values,
                            self.column_type(),
                            rhs.column_type(),
                        )
                        .as_ref(),
                    ),
                ))
            }
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge(lhs, rhs).as_ref()),
            )),
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Ok(Self::Boolean(
                alloc.alloc_slice_copy(slice_ge(lhs, rhs).as_ref()),
            )),
            (Self::TimestampTZ(lhs_tu, _, lhs_raw), Self::TimestampTZ(rhs_tu, _, rhu_raw)) => {
                let result = cmp_timestamps(*lhs_tu, lhs_raw, *rhs_tu, rhu_raw, |a, b| a >= b);
                Ok(Self::Boolean(alloc.alloc_slice_copy(result.as_ref())))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::GreaterThanOrEqual,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    #[allow(clippy::too_many_lines)]
    /// Element-wise + for two columns
    pub fn element_wise_add(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (&self, &rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::TinyInt(
                alloc.alloc_slice_copy(try_add_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_add_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Int(lhs), Self::TinyInt(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::SmallInt(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_add_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_add_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Int128(lhs), Self::TinyInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::SmallInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::Int(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::BigInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_add_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Add,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    #[allow(clippy::too_many_lines)]
    /// Element-wise - for two columns
    pub fn element_wise_sub(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (&self, &rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::TinyInt(
                alloc.alloc_slice_copy(try_subtract_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_subtract_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Int(lhs), Self::TinyInt(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::SmallInt(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_subtract_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_subtract_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_left_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Int128(lhs), Self::TinyInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::SmallInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::Int(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::BigInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices_right_upcast(lhs, rhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_subtract_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Subtract,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    #[allow(clippy::too_many_lines)]
    /// Element-wise * for two columns
    pub fn element_wise_mul(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (&self, &rhs) {
            (Self::TinyInt(lhs), Self::TinyInt(rhs)) => Ok(Self::TinyInt(
                alloc.alloc_slice_copy(try_multiply_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::SmallInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::TinyInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::SmallInt(lhs), Self::TinyInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => Ok(Self::SmallInt(
                alloc.alloc_slice_copy(try_multiply_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Int(lhs), Self::TinyInt(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::SmallInt(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Int(
                alloc.alloc_slice_copy(try_multiply_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::BigInt(lhs), Self::TinyInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(
                alloc.alloc_slice_copy(try_multiply_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(lhs, rhs)?.as_ref()),
            )),
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Int128(lhs), Self::TinyInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::SmallInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::Int(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::BigInt(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices_with_casting(rhs, lhs)?.as_ref()),
            )),
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Int128(
                alloc.alloc_slice_copy(try_multiply_slices(lhs, rhs)?.as_ref()),
            )),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }

            (Self::Decimal75(_, _, lhs_values), Self::TinyInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                let new_values = alloc.alloc_slice_copy(new_values.as_ref());
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Multiply,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};
    use proof_of_sql_parser::posql_time::PoSQLTimeZone;

    #[test]
    fn we_cannot_do_binary_operation_on_columns_with_different_lengths() {
        let alloc = Bump::new();
        let lhs = Column::<TestScalar>::Boolean(&[true, false, true]);
        let rhs = Column::<TestScalar>::Boolean(&[true, false]);

        let result = lhs.element_wise_and(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_le(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let lhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let rhs = Column::<TestScalar>::TinyInt(&[1, 2]);
        let result = lhs.element_wise_add(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let lhs = Column::<TestScalar>::SmallInt(&[1, 2, 3]);
        let rhs = Column::<TestScalar>::SmallInt(&[1, 2]);
        let result = lhs.element_wise_add(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_sub(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_mul(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
    }

    #[test]
    fn we_cannot_do_logical_operation_on_nonboolean_columns() {
        let alloc = Bump::new();

        let lhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let rhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let result = lhs.element_wise_and(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_or(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_not(&alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::UnaryOperationInvalidColumnType { .. })
        ));

        let lhs = Column::<TestScalar>::Int(&[1, 2, 3]);
        let rhs = Column::<TestScalar>::Int(&[1, 2, 3]);
        let result = lhs.element_wise_and(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_or(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_not(&alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::UnaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_do_logical_operation_on_boolean_columns() {
        let alloc = Bump::new();

        let lhs = Column::<TestScalar>::Boolean(&[true, false, true, false]);
        let rhs = Column::<TestScalar>::Boolean(&[true, true, false, false]);
        let result = lhs.element_wise_and(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, false, false]))
        );

        let result = lhs.element_wise_or(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, true, false]))
        );

        let result = lhs.element_wise_not(&alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[false, true, false, true]))
        );
    }

    #[test]
    fn we_can_do_eq_operation() {
        let alloc = Bump::new();

        // Integers
        let lhs = Column::<TestScalar>::SmallInt(&[1, 3, 2]);
        let rhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, false]))
        );

        let lhs = Column::<TestScalar>::Int(&[1, 3, 2]);
        let rhs = Column::<TestScalar>::SmallInt(&[1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, false]))
        );

        // Strings
        let data: Vec<&str> = vec!["Space", "and", "Time"];
        let scalars: Vec<TestScalar> = data.iter().map(TestScalar::from).collect();
        let alloc_data = alloc.alloc_slice_clone(data.as_ref());
        let alloc_scalars = alloc.alloc_slice_clone(scalars.as_ref());

        let bad_data: Vec<&str> = vec!["Space", "and", "Time2"];
        let bad_scalars: Vec<TestScalar> = bad_data.iter().map(TestScalar::from).collect();
        let alloc_bad_data = alloc.alloc_slice_clone(bad_data.as_ref());
        let alloc_bad_scalars = alloc.alloc_slice_clone(bad_scalars.as_ref());

        let lhs = Column::<TestScalar>::VarChar((alloc_data, alloc_scalars));
        let rhs = Column::<TestScalar>::VarChar((alloc_bad_data, alloc_bad_scalars));
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );

        // Booleans
        let lhs = Column::<TestScalar>::Boolean(&[true, false, true]);
        let rhs = Column::<TestScalar>::Boolean(&[true, true, false]);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, false]))
        );

        // Decimals
        let lhs_scalars: Vec<TestScalar> = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs_scalars: Vec<TestScalar> = [1, 2, -3].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let alloc_right_scalars = alloc.alloc_slice_copy(rhs_scalars.as_ref());
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, alloc_left_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, false]))
        );

        // Decimals and integers
        let lhs_scalars: Vec<TestScalar> = [10, 2, 30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs = Column::<TestScalar>::TinyInt(&[1, -2, 3]);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 1, alloc_left_scalars);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );

        let lhs_scalars: Vec<TestScalar> = [10, 2, 30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs = Column::<TestScalar>::Int(&[1, -2, 3]);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 1, alloc_left_scalars);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );

        // Timestamps
        // Note that timezone doesn't affect raw timestamp comparison since it is always stored in UTC
        // lhs and rhs have the same time unit
        let lhs_tz = PoSQLTimeZone::from_offset(0);
        let rhs_tz = PoSQLTimeZone::from_offset(0);
        let lhs_time_unit = PoSQLTimeUnit::Second;
        let rhs_time_unit = PoSQLTimeUnit::Second;
        let lhs_data: Vec<i64> = vec![1, 2, 3];
        let rhs_data: Vec<i64> = vec![-1, -2, 3];
        let alloc_left_data = alloc.alloc_slice_copy(lhs_data.as_ref());
        let alloc_right_data = alloc.alloc_slice_copy(rhs_data.as_ref());
        let lhs = Column::<TestScalar>::TimestampTZ(lhs_time_unit, lhs_tz, alloc_left_data);
        let rhs = Column::<TestScalar>::TimestampTZ(rhs_time_unit, rhs_tz, alloc_right_data);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[false, false, true]))
        );

        // lhs and rhs have different time units
        let lhs_tz = PoSQLTimeZone::from_offset(0);
        let rhs_tz = PoSQLTimeZone::from_offset(3600);
        let lhs_time_unit = PoSQLTimeUnit::Second;
        let rhs_time_unit = PoSQLTimeUnit::Millisecond;
        let lhs_data: Vec<i64> = vec![1, 2, 3];
        let rhs_data: Vec<i64> = vec![1000, 2000, 3002];
        let alloc_left_data = alloc.alloc_slice_copy(lhs_data.as_ref());
        let alloc_right_data = alloc.alloc_slice_copy(rhs_data.as_ref());
        let lhs = Column::<TestScalar>::TimestampTZ(lhs_time_unit, lhs_tz, alloc_left_data);
        let rhs = Column::<TestScalar>::TimestampTZ(rhs_time_unit, rhs_tz, alloc_right_data);
        let result = lhs.element_wise_eq(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );
    }

    #[test]
    fn we_can_do_le_operation_on_numeric_datetime_and_boolean_columns() {
        let alloc = Bump::new();

        // Booleans
        let lhs = Column::<TestScalar>::Boolean(&[true, false, true]);
        let rhs = Column::<TestScalar>::Boolean(&[true, true, false]);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );

        // Integers
        let lhs = Column::<TestScalar>::SmallInt(&[1, 3, 2]);
        let rhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );

        let lhs = Column::<TestScalar>::Int(&[1, 3, 2]);
        let rhs = Column::<TestScalar>::SmallInt(&[1, 2, 3]);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );

        // Decimals
        let lhs_scalars: Vec<TestScalar> = [10, 2, 30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs_scalars: Vec<TestScalar> = [1, 24, -3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(rhs_scalars.as_ref());
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, alloc_left_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );

        // Decimals and integers
        let lhs_scalars: Vec<TestScalar> = [10, -2, -30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs = Column::<TestScalar>::TinyInt(&[1, -20, 3]);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, alloc_left_scalars);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[false, true, true]))
        );

        let lhs_scalars: Vec<TestScalar> = [10, -2, -30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs = Column::<TestScalar>::Int(&[1, -20, 3]);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, alloc_left_scalars);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[false, true, true]))
        );

        // Timestamps
        // Note that timezone doesn't affect raw timestamp comparison since it is always stored in UTC
        // lhs and rhs have the same time unit
        let lhs_tz = PoSQLTimeZone::from_offset(0);
        let rhs_tz = PoSQLTimeZone::from_offset(0);
        let lhs_time_unit = PoSQLTimeUnit::Microsecond;
        let rhs_time_unit = PoSQLTimeUnit::Microsecond;
        let lhs_data: Vec<i64> = vec![1, 2, 3];
        let rhs_data: Vec<i64> = vec![-1, 4, 3];
        let alloc_left_data = alloc.alloc_slice_copy(lhs_data.as_ref());
        let alloc_right_data = alloc.alloc_slice_copy(rhs_data.as_ref());
        let lhs = Column::<TestScalar>::TimestampTZ(lhs_time_unit, lhs_tz, alloc_left_data);
        let rhs = Column::<TestScalar>::TimestampTZ(rhs_time_unit, rhs_tz, alloc_right_data);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[false, true, true]))
        );

        // lhs and rhs have different time units
        let lhs_tz = PoSQLTimeZone::from_offset(0);
        let rhs_tz = PoSQLTimeZone::from_offset(3600);
        let lhs_time_unit = PoSQLTimeUnit::Nanosecond;
        let rhs_time_unit = PoSQLTimeUnit::Millisecond;
        let lhs_data: Vec<i64> = vec![1_000_000, 2_900_000, 3_000_000];
        let rhs_data: Vec<i64> = vec![1, 2, 4];
        let alloc_left_data = alloc.alloc_slice_copy(lhs_data.as_ref());
        let alloc_right_data = alloc.alloc_slice_copy(rhs_data.as_ref());
        let lhs = Column::<TestScalar>::TimestampTZ(lhs_time_unit, lhs_tz, alloc_left_data);
        let rhs = Column::<TestScalar>::TimestampTZ(rhs_time_unit, rhs_tz, alloc_right_data);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );
    }

    #[test]
    fn we_can_do_ge_operation_on_numeric_datetime_and_boolean_columns() {
        let alloc = Bump::new();

        // Booleans
        let lhs = Column::<TestScalar>::Boolean(&[true, false, true]);
        let rhs = Column::<TestScalar>::Boolean(&[true, true, false]);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );

        // Integers
        let lhs = Column::<TestScalar>::SmallInt(&[1, 3, 2]);
        let rhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );

        let lhs = Column::<TestScalar>::Int(&[1, 3, 2]);
        let rhs = Column::<TestScalar>::SmallInt(&[1, 2, 3]);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );

        // Decimals
        let lhs_scalars: Vec<TestScalar> = [10, 2, 30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs_scalars: Vec<TestScalar> = [1, 24, -3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(rhs_scalars.as_ref());
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, alloc_left_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );

        // Decimals and integers
        let lhs_scalars: Vec<TestScalar> = [10, -2, -30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs = Column::<TestScalar>::TinyInt(&[1_i8, -20, 3]);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, alloc_left_scalars);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );

        let lhs_scalars: Vec<TestScalar> = [10, -2, -30].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs = Column::<TestScalar>::BigInt(&[1_i64, -20, 3]);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, alloc_left_scalars);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );

        // Timestamps
        // Note that timezone doesn't affect raw timestamp comparison since it is always stored in UTC
        // lhs and rhs have the same time unit
        let lhs_tz = PoSQLTimeZone::from_offset(0);
        let rhs_tz = PoSQLTimeZone::from_offset(0);
        let lhs_time_unit = PoSQLTimeUnit::Nanosecond;
        let rhs_time_unit = PoSQLTimeUnit::Nanosecond;
        let lhs_data: Vec<i64> = vec![1, 2, 3];
        let rhs_data: Vec<i64> = vec![-1, 4, 3];
        let alloc_left_data = alloc.alloc_slice_copy(lhs_data.as_ref());
        let alloc_right_data = alloc.alloc_slice_copy(rhs_data.as_ref());
        let lhs = Column::<TestScalar>::TimestampTZ(lhs_time_unit, lhs_tz, alloc_left_data);
        let rhs = Column::<TestScalar>::TimestampTZ(rhs_time_unit, rhs_tz, alloc_right_data);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, false, true]))
        );

        // lhs and rhs have different time units
        let lhs_tz = PoSQLTimeZone::from_offset(0);
        let rhs_tz = PoSQLTimeZone::from_offset(3600);
        let lhs_time_unit = PoSQLTimeUnit::Second;
        let rhs_time_unit = PoSQLTimeUnit::Microsecond;
        let lhs_data: Vec<i64> = vec![1, 2, 3];
        let rhs_data: Vec<i64> = vec![1_000_000, 1_200_000, 3_002_999];
        let alloc_left_data = alloc.alloc_slice_copy(lhs_data.as_ref());
        let alloc_right_data = alloc.alloc_slice_copy(rhs_data.as_ref());
        let lhs = Column::<TestScalar>::TimestampTZ(lhs_time_unit, lhs_tz, alloc_left_data);
        let rhs = Column::<TestScalar>::TimestampTZ(rhs_time_unit, rhs_tz, alloc_right_data);
        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert_eq!(
            result,
            Ok(Column::<TestScalar>::Boolean(&[true, true, false]))
        );
    }

    #[test]
    fn we_cannot_do_comparison_on_columns_with_incompatible_types() {
        let alloc = Bump::new();

        // Strings can't be compared with other types
        let data = vec!["Space", "and", "Time"];
        let scalars: Vec<TestScalar> = data.iter().map(TestScalar::from).collect();
        let alloc_data = alloc.alloc_slice_clone(data.as_ref());
        let alloc_scalars = alloc.alloc_slice_clone(scalars.as_ref());

        let lhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let rhs = Column::<TestScalar>::VarChar((alloc_data, alloc_scalars));
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = Column::<TestScalar>::Int(&[1, 2, 3]);
        let rhs = Column::<TestScalar>::VarChar((alloc_data, alloc_scalars));
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_le(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        // Booleans can't be compared with other types
        let lhs = Column::<TestScalar>::Boolean(&[true, false, true]);
        let rhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = Column::<TestScalar>::Boolean(&[true, false, true]);
        let rhs = Column::<TestScalar>::Int(&[1, 2, 3]);
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        // Strings can not be <= or >= to each other
        let lhs_data = vec!["Space", "and", "Time"];
        let alloc_left_data = alloc.alloc_slice_clone(lhs_data.as_ref());
        let lhs_scalars: Vec<TestScalar> = lhs_data.iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_clone(lhs_scalars.as_ref());

        let rhs_data = vec!["Space", "and", "time"];
        let alloc_right_data = alloc.alloc_slice_clone(rhs_data.as_ref());
        let rhs_scalars: Vec<TestScalar> = rhs_data.iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_clone(rhs_scalars.as_ref());

        let lhs = Column::<TestScalar>::VarChar((alloc_left_data, alloc_left_scalars));
        let rhs = Column::<TestScalar>::VarChar((alloc_right_data, alloc_right_scalars));
        let result = lhs.element_wise_le(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_ge(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_do_arithmetic_on_nonnumeric_columns() {
        let alloc = Bump::new();

        let lhs_data = vec!["Space", "and", "Time"];
        let alloc_left_data = alloc.alloc_slice_clone(lhs_data.as_ref());
        let lhs_scalars: Vec<TestScalar> = lhs_data.iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_clone(lhs_scalars.as_ref());
        let lhs = Column::<TestScalar>::VarChar((alloc_left_data, alloc_left_scalars));

        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = Column::<TestScalar>::Scalar(&rhs_scalars);
        let result = lhs.element_wise_add(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_sub(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_mul(&rhs, &alloc);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_add_integer_columns() {
        let alloc = Bump::new();
        // lhs and rhs have the same precision
        let lhs = Column::<TestScalar>::TinyInt(&[1_i8, 2, 3]);
        let rhs = Column::<TestScalar>::TinyInt(&[1_i8, 2, 3]);
        let result = lhs.element_wise_add(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::TinyInt(&[2_i8, 4, 6])));

        let lhs = Column::<TestScalar>::SmallInt(&[1_i16, 2, 3]);
        let rhs = Column::<TestScalar>::SmallInt(&[1_i16, 2, 3]);
        let result = lhs.element_wise_add(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::SmallInt(&[2_i16, 4, 6])));

        // lhs and rhs have different precisions
        let lhs = Column::<TestScalar>::TinyInt(&[1_i8, 2, 3]);
        let rhs = Column::<TestScalar>::Int(&[1_i32, 2, 3]);
        let result = lhs.element_wise_add(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::Int(&[2_i32, 4, 6])));

        let lhs = Column::<TestScalar>::Int128(&[1_i128, 2, 3]);
        let rhs = Column::<TestScalar>::Int(&[1_i32, 2, 3]);
        let result = lhs.element_wise_add(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::Int128(&[2_i128, 4, 6])));
    }

    #[test]
    fn we_can_try_add_decimal_columns() {
        let alloc = Bump::new();
        // lhs and rhs have the same precision and scale
        let lhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(rhs_scalars.as_ref());
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_left_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = (lhs.element_wise_add(&rhs, &alloc)).unwrap();
        let expected_scalars: Vec<TestScalar> = [2, 4, 6].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(expected_scalars.as_ref());
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, alloc_expected_scalars)
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(lhs_scalars.as_ref());
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(rhs_scalars.as_ref());
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_left_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(51).unwrap(), 3, alloc_right_scalars);
        let result = (lhs.element_wise_add(&rhs, &alloc)).unwrap();
        let expected_scalars: Vec<TestScalar> = [11, 22, 33].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(expected_scalars.as_ref());
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(52).unwrap(), 3, alloc_expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = Column::<TestScalar>::TinyInt(&[1, 2, 3]);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(rhs_scalars.as_ref());
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = (lhs.element_wise_add(&rhs, &alloc)).unwrap();
        let expected_scalars: Vec<TestScalar> =
            [101, 202, 303].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(expected_scalars.as_ref());
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, alloc_expected_scalars)
        );

        let lhs = Column::<TestScalar>::Int(&[1, 2, 3]);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(rhs_scalars.as_ref());
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = (lhs.element_wise_add(&rhs, &alloc)).unwrap();
        let expected_scalars: Vec<TestScalar> =
            [101, 202, 303].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(expected_scalars.as_ref());
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 2, alloc_expected_scalars)
        );
    }

    #[test]
    fn we_can_try_subtract_integer_columns() {
        let alloc = Bump::new();
        // lhs and rhs have the same precision
        let lhs = Column::<TestScalar>::TinyInt(&[4_i8, 5, 2]);
        let rhs = Column::<TestScalar>::TinyInt(&[1_i8, 2, 3]);
        let result = lhs.element_wise_sub(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::TinyInt(&[3_i8, 3, -1])));

        let lhs = Column::<TestScalar>::Int(&[4_i32, 5, 2]);
        let rhs = Column::<TestScalar>::Int(&[1_i32, 2, 3]);
        let result = lhs.element_wise_sub(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::Int(&[3_i32, 3, -1])));

        // lhs and rhs have different precisions
        let lhs = Column::<TestScalar>::TinyInt(&[4_i8, 5, 2]);
        let rhs = Column::<TestScalar>::BigInt(&[1_i64, 2, 5]);
        let result = lhs.element_wise_sub(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::BigInt(&[3_i64, 3, -3])));

        let lhs = Column::<TestScalar>::Int(&[3_i32, 2, 3]);
        let rhs = Column::<TestScalar>::BigInt(&[1_i64, 2, 5]);
        let result = lhs.element_wise_sub(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::BigInt(&[2_i64, 0, -2])));
    }

    #[test]
    fn we_can_try_subtract_decimal_columns() {
        let alloc = Bump::new();

        // lhs and rhs have the same precision and scale
        let lhs_scalars: Vec<TestScalar> = [4, 5, 2].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(&lhs_scalars);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(&rhs_scalars);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_left_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_sub(&rhs, &alloc).unwrap();
        let expected_scalars: Vec<TestScalar> = [3, 3, -1].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(&expected_scalars);
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, alloc_expected_scalars)
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars: Vec<TestScalar> = [4, 5, 2].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(&lhs_scalars);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(&rhs_scalars);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(25).unwrap(), 2, alloc_left_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(51).unwrap(), 3, alloc_right_scalars);
        let result = lhs.element_wise_sub(&rhs, &alloc).unwrap();
        let expected_scalars: Vec<TestScalar> = [39, 48, 17].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(&expected_scalars);
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(52).unwrap(), 3, alloc_expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs_scalars = &[4_i8, 5, 2];
        let lhs = Column::<TestScalar>::TinyInt(lhs_scalars);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(&rhs_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_sub(&rhs, &alloc).unwrap();
        let expected_scalars: Vec<TestScalar> =
            [399, 498, 197].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(&expected_scalars);
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, alloc_expected_scalars)
        );

        let lhs_scalars = &[4_i32, 5, 2];
        let lhs = Column::<TestScalar>::Int(lhs_scalars);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(&rhs_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_sub(&rhs, &alloc).unwrap();
        let expected_scalars: Vec<TestScalar> =
            [399, 498, 197].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(&expected_scalars);
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 2, alloc_expected_scalars)
        );
    }

    #[test]
    fn we_can_try_multiply_integer_columns() {
        let alloc = Bump::new();
        // lhs and rhs have the same precision
        let lhs = Column::<TestScalar>::TinyInt(&[4_i8, 5, -2]);
        let rhs = Column::<TestScalar>::TinyInt(&[1_i8, 2, 3]);
        let result = lhs.element_wise_mul(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::TinyInt(&[4_i8, 10, -6])));

        let lhs = Column::<TestScalar>::BigInt(&[4_i64, 5, -2]);
        let rhs = Column::<TestScalar>::BigInt(&[1_i64, 2, 3]);
        let result = lhs.element_wise_mul(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::BigInt(&[4_i64, 10, -6])));

        // lhs and rhs have different precisions
        let lhs = Column::<TestScalar>::TinyInt(&[3_i8, 2, 3]);
        let rhs = Column::<TestScalar>::Int128(&[1_i128, 2, 5]);
        let result = lhs.element_wise_mul(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::Int128(&[3_i128, 4, 15])));

        let lhs = Column::<TestScalar>::Int(&[3_i32, 2, 3]);
        let rhs = Column::<TestScalar>::Int128(&[1_i128, 2, 5]);
        let result = lhs.element_wise_mul(&rhs, &alloc);
        assert_eq!(result, Ok(Column::<TestScalar>::Int128(&[3_i128, 4, 15])));
    }

    #[test]
    fn we_can_try_multiply_decimal_columns() {
        let alloc = Bump::new();

        // lhs and rhs are both decimals
        let lhs_scalars: Vec<TestScalar> = [4, 5, 2].iter().map(TestScalar::from).collect();
        let alloc_left_scalars = alloc.alloc_slice_copy(&lhs_scalars);
        let lhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_left_scalars);
        let rhs_scalars: Vec<TestScalar> = [-1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(&rhs_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_mul(&rhs, &alloc).unwrap();
        let expected_scalars: Vec<TestScalar> = [-4, 10, 6].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(&expected_scalars);
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(11).unwrap(), 4, alloc_expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs_scalars = &[4_i8, 5, 2];
        let lhs = Column::<TestScalar>::TinyInt(lhs_scalars);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(&rhs_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_mul(&rhs, &alloc).unwrap();
        let expected_scalars: Vec<TestScalar> = [4, 10, 6].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(&expected_scalars);
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(9).unwrap(), 2, alloc_expected_scalars)
        );

        let lhs_scalars = &[4_i32, 5, 2];
        let lhs = Column::<TestScalar>::Int(lhs_scalars);
        let rhs_scalars: Vec<TestScalar> = [1, 2, 3].iter().map(TestScalar::from).collect();
        let alloc_right_scalars = alloc.alloc_slice_copy(&rhs_scalars);
        let rhs =
            Column::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, alloc_right_scalars);
        let result = lhs.element_wise_mul(&rhs, &alloc).unwrap();
        let expected_scalars: Vec<TestScalar> = [4, 10, 6].iter().map(TestScalar::from).collect();
        let alloc_expected_scalars = alloc.alloc_slice_copy(&expected_scalars);
        assert_eq!(
            result,
            Column::<TestScalar>::Decimal75(Precision::new(16).unwrap(), 2, alloc_expected_scalars)
        );
    }
}
