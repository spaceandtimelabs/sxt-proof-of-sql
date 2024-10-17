//! Module for parsing an `IntermediateDecimal` into a `Decimal75`.
use crate::base::{
    math::BigDecimalExt,
    scalar::{Scalar, ScalarConversionError},
};
use alloc::string::{String, ToString};
use bigdecimal::{BigDecimal, ParseBigDecimalError};
use snafu::Snafu;

use super::{InvalidPrecisionError, Precision};

/// Errors related to the processing of decimal values in proof-of-sql
#[derive(Snafu, Debug, PartialEq)]
pub enum IntermediateDecimalError {
    /// Represents an error encountered during the parsing of a decimal string.
    #[snafu(display("{error}"))]
    ParseError {
        /// The underlying error
        error: ParseBigDecimalError,
    },
    /// Error occurs when this decimal cannot fit in a primitive.
    #[snafu(display("Value out of range for target type"))]
    OutOfRange,
    /// Error occurs when this decimal cannot be losslessly cast into a primitive.
    #[snafu(display("Fractional part of decimal is non-zero"))]
    LossyCast,
    /// Cannot cast this decimal to a big integer
    #[snafu(display("Conversion to integer failed"))]
    ConversionFailure,
}

impl Eq for IntermediateDecimalError {}

/// Errors related to decimal operations.
#[derive(Snafu, Debug, Eq, PartialEq)]
pub enum DecimalError {
    #[snafu(display("Invalid decimal format or value: {error}"))]
    /// Error when a decimal format or value is incorrect,
    /// the string isn't even a decimal e.g. "notastring",
    /// "-21.233.122" etc aka `InvalidDecimal`
    InvalidDecimal {
        /// The underlying error
        error: String,
    },

    #[snafu(transparent)]
    /// Decimal precision exceeds the allowed limit,
    /// e.g. precision above 75/76/whatever set by Scalar
    /// or non-positive aka `InvalidPrecision`
    InvalidPrecision {
        /// The underlying error
        source: InvalidPrecisionError,
    },

    #[snafu(display("Decimal scale is not valid: {scale}"))]
    /// Decimal scale is not valid. Here we use i16 in order to include
    /// invalid scale values
    InvalidScale {
        /// The invalid scale value
        scale: String,
    },

    #[snafu(display("Unsupported operation: cannot round decimal: {error}"))]
    /// This error occurs when attempting to scale a
    /// decimal in such a way that a loss of precision occurs.
    RoundingError {
        /// The underlying error
        error: String,
    },

    /// Errors that may occur when parsing an intermediate decimal
    /// into a posql decimal
    #[snafu(transparent)]
    IntermediateDecimalConversionError {
        /// The underlying source error
        source: IntermediateDecimalError,
    },
}

/// Result type for decimal operations.
pub type DecimalResult<T> = Result<T, DecimalError>;

// This exists because `TryFrom<arrow::datatypes::DataType>` for `ColumnType` error is String
impl From<DecimalError> for String {
    fn from(error: DecimalError) -> Self {
        error.to_string()
    }
}

/// Fallibly attempts to convert an `IntermediateDecimal` into the
/// native proof-of-sql [Scalar] backing store. This function adjusts
/// the decimal to the specified `target_precision` and `target_scale`,
/// and validates that the adjusted decimal does not exceed the specified precision.
/// If the conversion is successful, it returns the `Scalar` representation;
/// otherwise, it returns a `DecimalError` indicating the type of failure
/// (e.g., exceeding precision limits).
///
/// ## Arguments
/// * `d` - The `IntermediateDecimal` to convert.
/// * `target_precision` - The maximum number of digits the scalar can represent.
/// * `target_scale` - The scale (number of decimal places) to use in the scalar.
///
/// ## Errors
/// Returns `DecimalError::InvalidPrecision` error if the number of digits in
/// the decimal exceeds the `target_precision` before or after adjusting for
/// `target_scale`, or if the target precision is zero.
pub(crate) fn try_convert_intermediate_decimal_to_scalar<S: Scalar>(
    d: &BigDecimal,
    target_precision: Precision,
    target_scale: i8,
) -> DecimalResult<S> {
    d.try_into_bigint_with_precision_and_scale(target_precision.value(), target_scale)?
        .try_into()
        .map_err(|e: ScalarConversionError| DecimalError::InvalidDecimal {
            error: e.to_string(),
        })
}

#[cfg(test)]
mod scale_adjust_test {

    use super::*;
    use crate::base::{math::precision::MAX_SUPPORTED_PRECISION, scalar::Curve25519Scalar};
    use num_bigint::BigInt;

    #[test]
    fn we_cannot_scale_past_max_precision() {
        let decimal = "12345678901234567890123456789012345678901234567890123456789012345678900.0"
            .parse()
            .unwrap();

        let target_scale = 5;

        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
                target_scale
            )
            .is_err()
        );
    }

    #[test]
    fn we_can_match_exact_decimals_from_queries_to_db() {
        let decimal: BigDecimal = "123.45".parse().unwrap();
        let target_scale = 2;
        let target_precision = 20;
        let big_int =
            decimal.try_into_bigint_with_precision_and_scale(target_precision, target_scale);
        let expected_big_int: BigInt = "12345".parse().unwrap();
        assert_eq!(big_int, Ok(expected_big_int));
    }

    #[test]
    fn we_can_match_decimals_with_negative_scale() {
        let decimal = "120.00".parse().unwrap();
        let target_scale = -1;
        let expected = [12, 0, 0, 0];
        let result = try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale,
        )
        .unwrap();
        assert_eq!(result, Curve25519Scalar::from(expected));
    }

    #[test]
    fn we_can_match_integers_with_negative_scale() {
        let decimal = "12300".parse().unwrap();
        let target_scale = -2;
        let expected_limbs = [123, 0, 0, 0];

        let limbs = try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
            target_scale,
        )
        .unwrap();

        assert_eq!(limbs, Curve25519Scalar::from(expected_limbs));
    }

    #[test]
    fn we_can_match_negative_decimals() {
        let decimal = "-123.45".parse().unwrap();
        let target_scale = 2;
        let expected_limbs = [12345, 0, 0, 0];
        let limbs = try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
            target_scale,
        )
        .unwrap();
        assert_eq!(limbs, -Curve25519Scalar::from(expected_limbs));
    }

    #[allow(clippy::cast_possible_wrap)]
    #[test]
    fn we_can_match_decimals_at_extrema() {
        // a big decimal cannot scale up past the supported precision
        let decimal = "1234567890123456789012345678901234567890123456789012345678901234567890.0"
            .parse()
            .unwrap();
        let target_scale = 6; // now precision exceeds maximum
        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
                target_scale
            )
            .is_err()
        );

        // maximum decimal value we can support
        let decimal =
            "99999999999999999999999999999999999999999999999999999999999999999999999999.0"
                .parse()
                .unwrap();
        let target_scale = 1;
        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_ok()
        );

        // scaling larger than max will fail
        let decimal =
            "999999999999999999999999999999999999999999999999999999999999999999999999999.0"
                .parse()
                .unwrap();
        let target_scale = 1;
        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_err()
        );

        // smallest possible decimal value we can support (either signed/unsigned)
        let decimal =
            "0.000000000000000000000000000000000000000000000000000000000000000000000000001"
                .parse()
                .unwrap();
        let target_scale = MAX_SUPPORTED_PRECISION as i8;
        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
                target_scale
            )
            .is_ok()
        );

        // this is ok because it can be scaled to 75 precision
        let decimal = "0.1".parse().unwrap();
        let target_scale = MAX_SUPPORTED_PRECISION as i8;
        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_ok()
        );

        // this exceeds max precision
        let decimal = "1.0".parse().unwrap();
        let target_scale = 75;
        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
                target_scale
            )
            .is_err()
        );

        // but this is ok
        let decimal = "1.0".parse().unwrap();
        let target_scale = 74;
        assert!(
            try_convert_intermediate_decimal_to_scalar::<Curve25519Scalar>(
                &decimal,
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_ok()
        );
    }
}
