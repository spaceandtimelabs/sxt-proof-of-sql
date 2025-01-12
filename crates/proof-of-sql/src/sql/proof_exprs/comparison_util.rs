use crate::{
    base::{
        database::{
            literal_value::{ExprExt, ToScalar},
            Column, ColumnError, ColumnarValue,
        },
        math::decimal::{DecimalError, Precision},
        scalar::{Scalar, ScalarExt},
        slice_ops,
    },
    sql::parse::{type_check_binary_operation, ConversionError, ConversionResult},
};
use alloc::{format, string::ToString, vec};
use bumpalo::Bump;
use core::cmp::{max, Ordering};
use sqlparser::ast::{BinaryOperator, DataType, Expr as SqlExpr, ObjectName};

/// Scale LHS and RHS to the same scale if at least one of them is decimal
/// and take the difference. This function is used for comparisons.
///
/// # Panics
/// This function will panic if `lhs` and `rhs` have [`ColumnType`]s that are not comparable
/// or if we have precision overflow issues.
#[allow(clippy::cast_sign_loss)]
pub fn scale_and_subtract_literal<S: Scalar>(
    lhs: &SqlExpr,
    rhs: &SqlExpr,
    lhs_scale: i8,
    rhs_scale: i8,
    is_equal: bool,
) -> ConversionResult<S> {
    let lhs_type = lhs.column_type();
    let rhs_type = rhs.column_type();
    let operator = if is_equal {
        BinaryOperator::Eq
    } else {
        BinaryOperator::LtEq
    };
    if !type_check_binary_operation(lhs_type, rhs_type, &operator) {
        return Err(ConversionError::DataTypeMismatch {
            left_type: lhs_type.to_string(),
            right_type: rhs_type.to_string(),
        });
    }
    let max_scale = max(lhs_scale, rhs_scale);
    let lhs_upscale = max_scale - lhs_scale;
    let rhs_upscale = max_scale - rhs_scale;
    // Only check precision overflow issues if at least one side is decimal
    if max_scale != 0 {
        let lhs_precision_value = lhs_type
            .precision_value()
            .expect("If scale is set, precision must be set");
        let rhs_precision_value = rhs_type
            .precision_value()
            .expect("If scale is set, precision must be set");
        let max_precision_value = max(
            lhs_precision_value + (max_scale - lhs_scale) as u8,
            rhs_precision_value + (max_scale - rhs_scale) as u8,
        );
        // Check if the precision is valid
        let _max_precision = Precision::new(max_precision_value).map_err(|_| {
            ConversionError::DecimalConversionError {
                source: DecimalError::InvalidPrecision {
                    error: max_precision_value.to_string(),
                },
            }
        })?;
    }
    match lhs_scale.cmp(&rhs_scale) {
        Ordering::Less => {
            let upscale_factor = S::pow10(rhs_upscale as u8);
            Ok(lhs.to_scalar::<S>() * upscale_factor - rhs.to_scalar())
        }
        Ordering::Equal => Ok(lhs.to_scalar::<S>() - rhs.to_scalar()),
        Ordering::Greater => {
            let upscale_factor = S::pow10(lhs_upscale as u8);
            Ok(lhs.to_scalar::<S>() - rhs.to_scalar::<S>() * upscale_factor)
        }
    }
}

#[allow(
    clippy::missing_panics_doc,
    reason = "precision and scale are validated prior to calling this function, ensuring no panic occurs"
)]
/// Scale LHS and RHS to the same scale if at least one of them is decimal
/// and take the difference. This function is used for comparisons.
#[allow(clippy::cast_sign_loss)]
pub(crate) fn scale_and_subtract<'a, S: Scalar>(
    alloc: &'a Bump,
    lhs: Column<'a, S>,
    rhs: Column<'a, S>,
    lhs_scale: i8,
    rhs_scale: i8,
    is_equal: bool,
) -> ConversionResult<&'a [S]> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    if lhs_len != rhs_len {
        return Err(ConversionError::DifferentColumnLength {
            len_a: lhs_len,
            len_b: rhs_len,
        });
    }
    let lhs_type = lhs.column_type();
    let rhs_type = rhs.column_type();
    let operator = if is_equal {
        BinaryOperator::Eq
    } else {
        BinaryOperator::LtEq
    };
    if !type_check_binary_operation(lhs_type, rhs_type, &operator) {
        return Err(ConversionError::DataTypeMismatch {
            left_type: lhs_type.to_string(),
            right_type: rhs_type.to_string(),
        });
    }
    let max_scale = max(lhs_scale, rhs_scale);
    let lhs_upscale = max_scale - lhs_scale;
    let rhs_upscale = max_scale - rhs_scale;
    // Only check precision overflow issues if at least one side is decimal
    if max_scale != 0 {
        let lhs_precision_value = lhs_type
            .precision_value()
            .expect("If scale is set, precision must be set");
        let rhs_precision_value = rhs_type
            .precision_value()
            .expect("If scale is set, precision must be set");
        let max_precision_value = max(
            lhs_precision_value + (max_scale - lhs_scale) as u8,
            rhs_precision_value + (max_scale - rhs_scale) as u8,
        );
        // Check if the precision is valid
        let _max_precision = Precision::new(max_precision_value).map_err(|_| {
            ConversionError::DecimalConversionError {
                source: DecimalError::InvalidPrecision {
                    error: max_precision_value.to_string(),
                },
            }
        })?;
    }
    let result = alloc.alloc_slice_fill_default(lhs_len);
    slice_ops::sub(
        result,
        &lhs.to_scalar_with_scaling(lhs_upscale),
        &rhs.to_scalar_with_scaling(rhs_upscale),
    );
    Ok(result)
}

#[allow(clippy::cast_sign_loss)]
#[allow(dead_code)]
/// Scale LHS and RHS to the same scale if at least one of them is decimal
/// and take the difference. This function is used for comparisons.
pub(crate) fn scale_and_subtract_columnar_value<'a, S: Scalar>(
    alloc: &'a Bump,
    lhs: ColumnarValue<'a, S>,
    rhs: ColumnarValue<'a, S>,
    lhs_scale: i8,
    rhs_scale: i8,
    is_equal: bool,
) -> Result<ColumnarValue<'a, S>, ColumnError> {
    match (lhs, rhs) {
        (ColumnarValue::Column(lhs), ColumnarValue::Column(rhs)) => {
            Ok(ColumnarValue::Column(Column::Scalar(
                scale_and_subtract(alloc, lhs, rhs, lhs_scale, rhs_scale, is_equal)
                    .map_err(|err| ColumnError::ConversionError { source: err })?,
            )))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Column(rhs)) => {
            Ok(ColumnarValue::Column(Column::Scalar(
                scale_and_subtract(
                    alloc,
                    Column::from_literal_with_length(&lhs, rhs.len(), alloc)?,
                    rhs,
                    lhs_scale,
                    rhs_scale,
                    is_equal,
                )
                .map_err(|err| ColumnError::ConversionError { source: err })?,
            )))
        }
        (ColumnarValue::Column(lhs), ColumnarValue::Literal(rhs)) => {
            Ok(ColumnarValue::Column(Column::Scalar(
                scale_and_subtract(
                    alloc,
                    lhs,
                    Column::from_literal_with_length(&rhs, lhs.len(), alloc)?,
                    lhs_scale,
                    rhs_scale,
                    is_equal,
                )
                .map_err(|err| ColumnError::ConversionError { source: err })?,
            )))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Literal(rhs)) => {
            let result_scalar =
                scale_and_subtract_literal::<S>(&lhs, &rhs, lhs_scale, rhs_scale, is_equal)
                    .map_err(|err| ColumnError::ConversionError { source: err })?;
            Ok(ColumnarValue::Literal(SqlExpr::TypedString {
                data_type: DataType::Custom(ObjectName(vec![]), vec![]),
                value: format!("scalar:{result_scalar}"),
            }))
        }
    }
}
