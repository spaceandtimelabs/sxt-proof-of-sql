use crate::{
    base::{
        database::Column,
        math::decimal::{DecimalError, Precision},
        scalar::Scalar,
        slice_ops,
    },
    sql::{util::try_binary_operation_type, AnalyzeError, AnalyzeResult},
};
use alloc::string::ToString;
use bumpalo::Bump;
use core::cmp::max;
use sqlparser::ast::BinaryOperator;

#[expect(
    clippy::missing_panics_doc,
    reason = "precision and scale are validated prior to calling this function, ensuring no panic occurs"
)]
/// Scale LHS and RHS to the same scale if at least one of them is decimal
/// and take the difference. This function is used for comparisons.
#[expect(clippy::cast_sign_loss)]
pub(crate) fn scale_and_subtract<'a, S: Scalar>(
    alloc: &'a Bump,
    lhs: Column<'a, S>,
    rhs: Column<'a, S>,
    is_equal: bool,
) -> AnalyzeResult<&'a [S]> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    if lhs_len != rhs_len {
        return Err(AnalyzeError::DifferentColumnLength {
            len_a: lhs_len,
            len_b: rhs_len,
        });
    }
    let lhs_type = lhs.column_type();
    let rhs_type = rhs.column_type();
    let operator = if is_equal {
        BinaryOperator::Eq
    } else {
        BinaryOperator::Lt
    };
    if try_binary_operation_type(lhs_type, rhs_type, &operator).is_none() {
        return Err(AnalyzeError::DataTypeMismatch {
            left_type: lhs_type.to_string(),
            right_type: rhs_type.to_string(),
        });
    }
    let lhs_scale = lhs_type.scale().unwrap_or(0);
    let rhs_scale = rhs_type.scale().unwrap_or(0);
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
            AnalyzeError::DecimalConversionError {
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
