use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::ColumnType,
    math::decimal::{DecimalError, MAX_SUPPORTED_PRECISION},
};
use alloc::{format, string::ToString};
// For decimal type manipulation please refer to
// https://learn.microsoft.com/en-us/sql/t-sql/data-types/precision-scale-and-length-transact-sql?view=sql-server-ver16

/// Determine the output type of an add or subtract operation if it is possible
/// to add or subtract the two input types. If the types are not compatible, return
/// an error.
///
/// # Panics
///
/// - Panics if `lhs` or `rhs` does not have a precision or scale when they are expected to be numeric types.
/// - Panics if `lhs` or `rhs` is an integer, and `lhs.max_integer_type(&rhs)` returns `None`.
pub fn try_add_subtract_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "+/-".to_string(),
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs.is_integer() && rhs.is_integer() {
        // We can unwrap here because we know that both types are integers
        return Ok(lhs.max_integer_type(&rhs).unwrap());
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value =
            i16::from(lhs.precision_value().expect("Numeric types have precision"));
        let right_precision_value =
            i16::from(rhs.precision_value().expect("Numeric types have precision"));
        let left_scale = lhs.scale().expect("Numeric types have scale");
        let right_scale = rhs.scale().expect("Numeric types have scale");
        let scale = left_scale.max(right_scale);
        let precision_value: i16 = i16::from(scale)
            + (left_precision_value - i16::from(left_scale))
                .max(right_precision_value - i16::from(right_scale))
            + 1_i16;
        let precision = u8::try_from(precision_value).map_err(|_| {
            ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision {
                    error: precision_value.to_string(),
                },
            }
        })?;
        if precision > MAX_SUPPORTED_PRECISION {
            return Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision {
                    error: format!(
                        "Required precision {precision_value} is beyond what we can support"
                    ),
                },
            });
        }
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Determine the output type of a multiplication operation if it is possible
/// to multiply the two input types. If the types are not compatible, return
/// an error.
///
/// # Panics
///
/// - Panics if `lhs` or `rhs` does not have a precision or scale when they are expected to be numeric types.
/// - Panics if `lhs` or `rhs` is an integer, and `lhs.max_integer_type(&rhs)` returns `None`.
pub fn try_multiply_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "*".to_string(),
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs.is_integer() && rhs.is_integer() {
        // We can unwrap here because we know that both types are integers
        return Ok(lhs.max_integer_type(&rhs).unwrap());
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value = lhs.precision_value().expect("Numeric types have precision");
        let right_precision_value = rhs.precision_value().expect("Numeric types have precision");
        let precision_value = left_precision_value + right_precision_value + 1;
        let precision = u8::try_from(precision_value).map_err(|_| {
            ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision {
                    error: precision_value.to_string(),
                },
            }
        })?;
        if precision > MAX_SUPPORTED_PRECISION {
            return Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision {
                    error: format!(
                        "Required precision {precision_value} is beyond what we can support"
                    ),
                },
            });
        }
        let left_scale = lhs.scale().expect("Numeric types have scale");
        let right_scale = rhs.scale().expect("Numeric types have scale");
        let scale = left_scale.checked_add(right_scale).ok_or(
            ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale {
                    scale: (i16::from(left_scale) + i16::from(right_scale)).to_string(),
                },
            },
        )?;
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Determine the output type of a divide/modulo operation if it is possible
/// to divide and modulo the two input types. If the types are not compatible, return
/// an error.
#[cfg_attr(not(test), expect(dead_code))]
fn try_divide_modulo_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<(ColumnType, ColumnType)> {
    if lhs.is_integer() && lhs.is_signed() && rhs.is_integer() && rhs.is_signed() {
        Ok((lhs, lhs))
    } else {
        Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "/%".to_string(),
            left_type: lhs,
            right_type: rhs,
        })
    }
}

/// Determine the output type of a division operation if it is possible
/// to divide the two input types. If the types are not compatible, return
/// an error.
///
/// # Panics
///
/// - Panics if `lhs` or `rhs` does not have a precision or scale when they are expected to be numeric types.
/// - Panics if `lhs` or `rhs` is an integer, and `lhs.max_integer_type(&rhs)` returns `None`.
pub fn try_divide_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric()
        || !rhs.is_numeric()
        || lhs == ColumnType::Scalar
        || rhs == ColumnType::Scalar
    {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "/".to_string(),
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs.is_integer() && rhs.is_integer() {
        // We can unwrap here because we know that both types are integers
        return Ok(lhs.max_integer_type(&rhs).unwrap());
    }
    let left_precision_value =
        i16::from(lhs.precision_value().expect("Numeric types have precision"));
    let right_precision_value =
        i16::from(rhs.precision_value().expect("Numeric types have precision"));
    let left_scale = i16::from(lhs.scale().expect("Numeric types have scale"));
    let right_scale = i16::from(rhs.scale().expect("Numeric types have scale"));
    let raw_scale = (left_scale + right_precision_value + 1_i16).max(6_i16);
    let precision_value: i16 = left_precision_value - left_scale + right_scale + raw_scale;
    let scale =
        i8::try_from(raw_scale).map_err(|_| ColumnOperationError::DecimalConversionError {
            source: DecimalError::InvalidScale {
                scale: raw_scale.to_string(),
            },
        })?;
    let precision = u8::try_from(precision_value).map_err(|_| {
        ColumnOperationError::DecimalConversionError {
            source: DecimalError::InvalidPrecision {
                error: precision_value.to_string(),
            },
        }
    })?;
    if precision > MAX_SUPPORTED_PRECISION {
        return Err(ColumnOperationError::DecimalConversionError {
            source: DecimalError::InvalidPrecision {
                error: format!(
                    "Required precision {precision_value} is beyond what we can support"
                ),
            },
        });
    }
    Ok(ColumnType::Decimal75(precision, scale))
}

/// Verifies that `from` can be cast to `to`. For now, this supports a limited number of casts.
#[expect(clippy::missing_panics_doc)]
pub fn try_cast_types(from: ColumnType, to: ColumnType) -> ColumnOperationResult<()> {
    match (from, to) {
        (
            ColumnType::Boolean,
            ColumnType::TinyInt
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::Int128
            | ColumnType::BigInt,
        )
        | (ColumnType::TimestampTZ(_, _), ColumnType::BigInt)
        | (ColumnType::Uint8, ColumnType::Uint8)
        | (ColumnType::TinyInt, ColumnType::TinyInt) => true,
        (
            ColumnType::TinyInt
            | ColumnType::Uint8
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::Int128
            | ColumnType::BigInt,
            ColumnType::Decimal75(_, 0)
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::BigInt
            | ColumnType::Int128,
        )
        | (ColumnType::Decimal75(_, _), ColumnType::Decimal75(_, _)) => {
            to.precision_value().unwrap() >= from.precision_value().unwrap()
                && to.scale() == from.scale()
        }
        _ => false,
    }
    .then_some(())
    .ok_or(ColumnOperationError::CastingError {
        left_type: from,
        right_type: to,
    })
}

/// Verifies that `from` can be cast to `to`.
/// Casting can only be supported if the resulting data type is a superset of the input data type.
/// For example Deciaml(6,1) can be cast to Decimal(7,1), but not vice versa.
#[expect(clippy::missing_panics_doc)]
pub fn try_decimal_scale_cast_types(from: ColumnType, to: ColumnType) -> ColumnOperationResult<()> {
    match (from, to) {
        (
            ColumnType::TinyInt
            | ColumnType::Uint8
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::Int128
            | ColumnType::BigInt
            | ColumnType::Decimal75(_, _),
            ColumnType::Decimal75(precision, scale),
        ) => {
            let from_precision = i16::from(from.precision_value().unwrap());
            let from_scale = i16::from(from.scale().unwrap());
            let to_precision = i16::from(precision.value());
            let to_scale = i16::from(scale);
            to_scale >= from_scale && (to_precision - to_scale) >= (from_precision - from_scale)
        }
        _ => false,
    }
    .then_some(())
    .ok_or(ColumnOperationError::ScaleCastingError {
        left_type: from,
        right_type: to,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use itertools::iproduct;

    #[test]
    fn we_can_add_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::TinyInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Int;
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal with nonnegative scale and rhs is an integer
        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(11_u8, 2);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(11_u8, 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(20_u8, 3);
        let rhs = ColumnType::Decimal75(10_u8, 2);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(21_u8, 3);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(13_u8, 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(13_u8, 0);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(40_u8, -13);
        let rhs = ColumnType::Decimal75(15_u8, 5);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(59_u8, 5);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(74_u8, -13);
        let rhs = ColumnType::Decimal75(15_u8, -14);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(75_u8, -13);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_add_non_numeric_types() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_add_some_numeric_types_due_to_decimal_issues() {
        let lhs = ColumnType::Decimal75(75_u8, 4);
        let rhs = ColumnType::Decimal75(73_u8, 4);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(75_u8, 10);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));
    }

    #[test]
    fn we_can_subtract_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::TinyInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Int;
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal and rhs is an integer
        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(11_u8, 2);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(11_u8, 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(20_u8, 3);
        let rhs = ColumnType::Decimal75(10_u8, 2);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(21_u8, 3);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(13_u8, 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(13_u8, 0);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(40_u8, -13);
        let rhs = ColumnType::Decimal75(15_u8, 5);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(59_u8, 5);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(61_u8, -13);
        let rhs = ColumnType::Decimal75(73_u8, -14);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(75_u8, -13);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_subtract_non_numeric_types() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_subtract_some_numeric_types_due_to_decimal_issues() {
        let lhs = ColumnType::Decimal75(75_u8, 0);
        let rhs = ColumnType::Decimal75(73_u8, 1);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));

        let lhs = ColumnType::Int128;
        let rhs = ColumnType::Decimal75(75_u8, 12);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));
    }

    #[test]
    fn we_can_multiply_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::TinyInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Int;
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Scalar;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal and rhs is an integer
        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(14_u8, 2);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(16_u8, 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(20_u8, 3);
        let rhs = ColumnType::Decimal75(10_u8, 2);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(31_u8, 5);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(14_u8, -2);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(16_u8, -2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(40_u8, -13);
        let rhs = ColumnType::Decimal75(15_u8, 5);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(56_u8, -8);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(61_u8, -13);
        let rhs = ColumnType::Decimal75(13_u8, -14);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(75_u8, -27);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_multiply_non_numeric_types() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_multiply_some_numeric_types_due_to_decimal_issues() {
        // Invalid precision
        let lhs = ColumnType::Decimal75(38_u8, 4);
        let rhs = ColumnType::Decimal75(37_u8, 4);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(65_u8, 0);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));

        // Invalid scale
        let lhs = ColumnType::Decimal75(5_u8, -64_i8);
        let rhs = ColumnType::Decimal75(5_u8, -65_i8);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale { .. }
            })
        ));

        let lhs = ColumnType::Decimal75(5_u8, 64_i8);
        let rhs = ColumnType::Decimal75(5_u8, 64_i8);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale { .. }
            })
        ));
    }

    #[test]
    fn we_can_divide_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::TinyInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Int;
        assert_eq!(expected, actual);

        // lhs is a decimal with nonnegative scale and rhs is an integer
        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(14_u8, 6);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(10_u8, 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(16_u8, 8);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with nonnegative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(10_u8, 2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(16_u8, 11);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(10_u8, 2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(18_u8, 11);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(20_u8, 3);
        let rhs = ColumnType::Decimal75(10_u8, 2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(33_u8, 14);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(12_u8, 11);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(10_u8, -2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(14_u8, 11);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(40_u8, -13);
        let rhs = ColumnType::Decimal75(15_u8, 5);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(64_u8, 6);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(70_u8, -13);
        let rhs = ColumnType::Decimal75(13_u8, -14);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(75_u8, 6);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_divide_non_numeric_or_scalar_types() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::Scalar;
        let rhs = ColumnType::Scalar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_divide_some_numeric_types_due_to_decimal_issues() {
        // Invalid precision
        let lhs = ColumnType::Decimal75(71_u8, -13);
        let rhs = ColumnType::Decimal75(13_u8, -14);
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(68_u8, 67);
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidPrecision { .. }
            })
        ));

        // Invalid scale
        let lhs = ColumnType::Decimal75(15_u8, 53_i8);
        let rhs = ColumnType::Decimal75(75_u8, 40_i8);
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale { .. }
            })
        ));
    }

    #[test]
    fn we_can_get_correct_column_types_for_divide_and_modulo() {
        let eligible_columns = [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ];
        for (numerator, denominator) in iproduct!(eligible_columns, eligible_columns) {
            let remainder = try_divide_modulo_column_types(numerator, denominator).unwrap();
            assert_eq!(remainder, (numerator, numerator));
        }
        let ineligible_columns = [
            ColumnType::Uint8,
            ColumnType::Scalar,
            ColumnType::Boolean,
            ColumnType::VarBinary,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::new(1)),
            ColumnType::Decimal75(1_u8, 1),
            ColumnType::VarChar,
        ];
        for (left_type, right_type) in iproduct!(eligible_columns, ineligible_columns)
            .chain(iproduct!(ineligible_columns, eligible_columns))
        {
            let err = try_divide_modulo_column_types(left_type, right_type).unwrap_err();
            assert!(matches!(
                err,
                ColumnOperationError::BinaryOperationInvalidColumnType {
                    operator: _,
                    left_type: _,
                    right_type: _
                }
            ));
        }
    }

    #[test]
    fn we_can_cast_bool_to_signed_ints() {
        for to in [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ] {
            try_cast_types(ColumnType::Boolean, to).unwrap();
        }
    }

    #[test]
    fn we_cannot_cast_integers_to_decimal_with_lower_precision() {
        for from in [
            ColumnType::TinyInt,
            ColumnType::Uint8,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
            ColumnType::Decimal75(60_u8, 0),
        ] {
            assert!(matches!(
                try_cast_types(from, ColumnType::Decimal75(2_u8, 0)),
                Err(ColumnOperationError::CastingError { .. })
            ));
        }
    }

    #[test]
    fn we_can_cast_integers_and_decimal_to_decimal() {
        for from in [
            ColumnType::TinyInt,
            ColumnType::Uint8,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
            ColumnType::Decimal75(60_u8, 0),
        ] {
            try_cast_types(from, ColumnType::Decimal75(60_u8, 0)).unwrap();
            try_cast_types(
                from,
                ColumnType::Decimal75(from.precision_value().unwrap(), 0),
            )
            .unwrap();
            try_cast_types(from, ColumnType::Decimal75(2_u8, 0)).unwrap_err();
        }
    }

    #[test]
    fn we_can_cast_decimal_to_decimal_with_same_scale() {
        try_cast_types(
            ColumnType::Decimal75(2_u8, 1),
            ColumnType::Decimal75(3_u8, 1),
        )
        .unwrap();
        try_cast_types(
            ColumnType::Decimal75(2_u8, 1),
            ColumnType::Decimal75(3_u8, 2),
        )
        .unwrap_err();
    }

    #[test]
    fn we_can_cast_integers_to_signed_integers() {
        try_cast_types(ColumnType::Uint8, ColumnType::Uint8).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::Uint8, ColumnType::SmallInt).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::TinyInt).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::SmallInt).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::SmallInt, ColumnType::SmallInt).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::Int, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::Int, ColumnType::SmallInt).unwrap_err();
        try_cast_types(ColumnType::Int, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::Int, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::Int, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::BigInt, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::BigInt, ColumnType::SmallInt).unwrap_err();
        try_cast_types(ColumnType::BigInt, ColumnType::Int).unwrap_err();
        try_cast_types(ColumnType::BigInt, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::BigInt, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::Int128, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::SmallInt).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::Int).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::BigInt).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::Int128).unwrap();
    }

    #[test]
    fn we_can_cast_timestamp_to_big_int() {
        try_cast_types(
            ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::new(1)),
            ColumnType::BigInt,
        )
        .unwrap();
    }

    #[test]
    fn we_cannot_cast_uncastable_type() {
        let err = try_cast_types(ColumnType::BigInt, ColumnType::Boolean).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::CastingError {
                left_type: ColumnType::BigInt,
                right_type: ColumnType::Boolean
            }
        ));
    }

    #[expect(clippy::too_many_lines)]
    #[test]
    fn we_can_properly_determine_if_types_are_scale_castable() {
        for from in [
            ColumnType::Uint8,
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ] {
            let from_precision = from.precision_value().unwrap();
            let two_prec = 2_u8;
            let forty_prec = 40_u8;
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(two_prec, 0)).unwrap_err();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(two_prec, -1)).unwrap_err();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(two_prec, 1)).unwrap_err();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(from_precision, 0)).unwrap();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(from_precision, -1))
                .unwrap_err();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(from_precision, 1))
                .unwrap_err();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(forty_prec, 0)).unwrap();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(forty_prec, -1)).unwrap_err();
            try_decimal_scale_cast_types(from, ColumnType::Decimal75(forty_prec, 1)).unwrap();
        }

        let twenty_prec = 20_u8;

        // from_with_negative_scale
        let neg_scale = ColumnType::Decimal75(twenty_prec, -3);

        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, -4))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, -3)).unwrap();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, -2))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, 0)).unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, 1)).unwrap_err();

        let nineteen_prec = 19_u8;
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, -4))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, -3))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, -2))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, 0))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, 1))
            .unwrap_err();

        let twenty_one_prec = 21_u8;
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, -4))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, -3))
            .unwrap();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, -2))
            .unwrap();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, 0))
            .unwrap_err();
        try_decimal_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, 1))
            .unwrap_err();

        // from_with_zero_scale
        let zero_scale = ColumnType::Decimal75(twenty_prec, 0);

        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_prec, -1))
            .unwrap_err();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_prec, 0)).unwrap();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_prec, 1))
            .unwrap_err();

        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, -1))
            .unwrap_err();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, 0))
            .unwrap_err();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, 1))
            .unwrap_err();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, 2))
            .unwrap_err();

        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, -1))
            .unwrap_err();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, 0))
            .unwrap();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, 1))
            .unwrap();
        try_decimal_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, 2))
            .unwrap_err();

        // from_with_positive_scale
        let pos_scale = ColumnType::Decimal75(twenty_prec, 3);

        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, -1))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 0)).unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 2)).unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 3)).unwrap();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 4)).unwrap_err();

        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, -1))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 0))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 2))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 3))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 4))
            .unwrap_err();

        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, -1))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 0))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 2))
            .unwrap_err();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 3)).unwrap();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 4)).unwrap();
        try_decimal_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 5))
            .unwrap_err();
    }

    #[test]
    fn we_cannot_scale_cast_nonsense_pairings() {
        try_decimal_scale_cast_types(ColumnType::Int128, ColumnType::Boolean).unwrap_err();
    }
}
