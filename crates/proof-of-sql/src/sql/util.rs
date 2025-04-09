use super::{AnalyzeError, AnalyzeResult};
use crate::base::database::{try_add_subtract_column_types, try_multiply_column_types, ColumnType};
use alloc::string::ToString;
use sqlparser::ast::BinaryOperator;

/// Checks if the binary operation between the left and right data types is valid.
///
/// # Arguments
///
/// * `left_dtype` - The data type of the left operand.
/// * `right_dtype` - The data type of the right operand.
/// * `binary_operator` - The binary operator to be applied.
///
/// # Returns
///
/// * `Some(result_type)` if the operation is valid, `None` otherwise.
pub(crate) fn try_binary_operation_type(
    left_dtype: ColumnType,
    right_dtype: ColumnType,
    binary_operator: &BinaryOperator,
) -> Option<ColumnType> {
    match binary_operator {
        BinaryOperator::And | BinaryOperator::Or => matches!(
            (left_dtype, right_dtype),
            (ColumnType::Boolean, ColumnType::Boolean)
        )
        .then_some(ColumnType::Boolean),
        BinaryOperator::Eq => (matches!(
            (left_dtype, right_dtype),
            (ColumnType::VarChar, ColumnType::VarChar)
                | (ColumnType::VarBinary, ColumnType::VarBinary)
                | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                | (ColumnType::Boolean, ColumnType::Boolean)
                | (_, ColumnType::Scalar)
                | (ColumnType::Scalar, _)
        ) || (left_dtype.is_numeric() && right_dtype.is_numeric()))
        .then_some(ColumnType::Boolean),
        BinaryOperator::Gt | BinaryOperator::Lt => {
            if left_dtype == ColumnType::VarChar || right_dtype == ColumnType::VarChar {
                return None;
            }
            // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
            if let ColumnType::Decimal75(precision, _) = left_dtype {
                if precision.value() > 38 {
                    return None;
                }
            }
            if let ColumnType::Decimal75(precision, _) = right_dtype {
                if precision.value() > 38 {
                    return None;
                }
            }
            (left_dtype.is_numeric() && right_dtype.is_numeric()
                || matches!(
                    (left_dtype, right_dtype),
                    (ColumnType::Boolean, ColumnType::Boolean)
                        | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                ))
            .then_some(ColumnType::Boolean)
        }
        BinaryOperator::Plus | BinaryOperator::Minus => {
            try_add_subtract_column_types(left_dtype, right_dtype).ok()
        }
        BinaryOperator::Multiply => try_multiply_column_types(left_dtype, right_dtype).ok(),
        BinaryOperator::Divide => {
            (left_dtype.is_numeric() && right_dtype.is_numeric()).then_some(left_dtype)
        }
        _ => {
            // Handle unsupported binary operations
            None
        }
    }
}

pub(crate) fn check_dtypes(
    left_dtype: ColumnType,
    right_dtype: ColumnType,
    binary_operator: &BinaryOperator,
) -> AnalyzeResult<ColumnType> {
    try_binary_operation_type(left_dtype, right_dtype, binary_operator).ok_or(
        AnalyzeError::DataTypeMismatch {
            left_type: left_dtype.to_string(),
            right_type: right_dtype.to_string(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::try_binary_operation_type;
    use crate::base::{database::ColumnType, math::decimal::Precision};
    use sqlparser::ast::BinaryOperator;

    #[test]
    fn we_can_return_none_if_decimal_has_too_high_of_precision() {
        assert!(try_binary_operation_type(
            ColumnType::Int,
            ColumnType::Decimal75(40_u8, 0),
            &BinaryOperator::Gt
        )
        .is_none());
    }

    #[test]
    fn we_can_return_none_for_unsupported_operation() {
        assert!(try_binary_operation_type(
            ColumnType::Int,
            ColumnType::Decimal75(40_u8, 0),
            &BinaryOperator::BitwiseXor
        )
        .is_none());
    }

    #[test]
    fn we_can_return_value_for_divide() {
        assert!(try_binary_operation_type(
            ColumnType::Int,
            ColumnType::Decimal75(40_u8, 0),
            &BinaryOperator::Divide
        )
        .is_some());
    }
}
