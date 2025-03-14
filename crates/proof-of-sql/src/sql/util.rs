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
/// * `true` if the operation is valid, `false` otherwise.
pub(crate) fn type_check_binary_operation(
    left_dtype: ColumnType,
    right_dtype: ColumnType,
    binary_operator: &BinaryOperator,
) -> bool {
    match binary_operator {
        BinaryOperator::And | BinaryOperator::Or => {
            matches!(
                (left_dtype, right_dtype),
                (ColumnType::Boolean, ColumnType::Boolean)
            )
        }
        BinaryOperator::Eq => {
            matches!(
                (left_dtype, right_dtype),
                (ColumnType::VarChar, ColumnType::VarChar)
                    | (ColumnType::VarBinary, ColumnType::VarBinary)
                    | (
                        ColumnType::FixedSizeBinary(_),
                        ColumnType::FixedSizeBinary(_)
                    )
                    | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                    | (ColumnType::Boolean, ColumnType::Boolean)
                    | (_, ColumnType::Scalar)
                    | (ColumnType::Scalar, _)
            ) || (left_dtype.is_numeric() && right_dtype.is_numeric())
        }
        BinaryOperator::Gt | BinaryOperator::Lt => {
            if left_dtype == ColumnType::VarChar || right_dtype == ColumnType::VarChar {
                return false;
            }
            // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
            if let ColumnType::Decimal75(precision, _) = left_dtype {
                if precision.value() > 38 {
                    return false;
                }
            }
            if let ColumnType::Decimal75(precision, _) = right_dtype {
                if precision.value() > 38 {
                    return false;
                }
            }
            left_dtype.is_numeric() && right_dtype.is_numeric()
                || matches!(
                    (left_dtype, right_dtype),
                    (ColumnType::Boolean, ColumnType::Boolean)
                        | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                )
        }
        BinaryOperator::Plus | BinaryOperator::Minus => {
            try_add_subtract_column_types(left_dtype, right_dtype).is_ok()
        }
        BinaryOperator::Multiply => try_multiply_column_types(left_dtype, right_dtype).is_ok(),
        BinaryOperator::Divide => left_dtype.is_numeric() && right_dtype.is_numeric(),
        _ => {
            // Handle unsupported binary operations
            false
        }
    }
}

pub(crate) fn check_dtypes(
    left_dtype: ColumnType,
    right_dtype: ColumnType,
    binary_operator: &BinaryOperator,
) -> AnalyzeResult<()> {
    if type_check_binary_operation(left_dtype, right_dtype, binary_operator) {
        Ok(())
    } else {
        Err(AnalyzeError::DataTypeMismatch {
            left_type: left_dtype.to_string(),
            right_type: right_dtype.to_string(),
        })
    }
}
