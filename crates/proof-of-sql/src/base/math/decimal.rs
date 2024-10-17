//! Module for parsing an `IntermediateDecimal` into a `Decimal75`.
use crate::base::scalar::Scalar;
use alloc::string::{String, ToString};
use bigdecimal::ParseBigDecimalError;
use snafu::Snafu;

use super::InvalidPrecisionError;

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

#[cfg(test)]
mod scale_adjust_test {

    use crate::base::{
        math::{precision::MAX_SUPPORTED_PRECISION, BigDecimalExt, Precision},
        scalar::Curve25519Scalar,
    };
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;

    #[test]
    fn we_cannot_scale_past_max_precision() {
        let decimal = "12345678901234567890123456789012345678901234567890123456789012345678900.0"
            .parse::<BigDecimal>()
            .unwrap();

        let target_scale = 5;

        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
                target_scale
            )
            .is_err());
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
        let decimal = "120.00".parse::<BigDecimal>().unwrap();
        let target_scale = -1;
        let expected = [12, 0, 0, 0];
        let result = decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale,
            )
            .unwrap();
        assert_eq!(result, Curve25519Scalar::from(expected));
    }

    #[test]
    fn we_can_match_integers_with_negative_scale() {
        let decimal = "12300".parse::<BigDecimal>().unwrap();
        let target_scale = -2;
        let expected_limbs = [123, 0, 0, 0];

        let limbs = decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
                target_scale,
            )
            .unwrap();

        assert_eq!(limbs, Curve25519Scalar::from(expected_limbs));
    }

    #[test]
    fn we_can_match_negative_decimals() {
        let decimal = "-123.45".parse::<BigDecimal>().unwrap();
        let target_scale = 2;
        let expected_limbs = [12345, 0, 0, 0];
        let limbs = decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
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
            .parse::<BigDecimal>()
            .unwrap();
        let target_scale = 6; // now precision exceeds maximum
        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
                target_scale
            )
            .is_err());

        // maximum decimal value we can support
        let decimal =
            "99999999999999999999999999999999999999999999999999999999999999999999999999.0"
                .parse::<BigDecimal>()
                .unwrap();
        let target_scale = 1;
        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_ok());

        // scaling larger than max will fail
        let decimal =
            "999999999999999999999999999999999999999999999999999999999999999999999999999.0"
                .parse::<BigDecimal>()
                .unwrap();
        let target_scale = 1;
        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_err());

        // smallest possible decimal value we can support (either signed/unsigned)
        let decimal =
            "0.000000000000000000000000000000000000000000000000000000000000000000000000001"
                .parse::<BigDecimal>()
                .unwrap();
        let target_scale = MAX_SUPPORTED_PRECISION as i8;
        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
                target_scale
            )
            .is_ok());

        // this is ok because it can be scaled to 75 precision
        let decimal = "0.1".parse::<BigDecimal>().unwrap();
        let target_scale = MAX_SUPPORTED_PRECISION as i8;
        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_ok());

        // this exceeds max precision
        let decimal = "1.0".parse::<BigDecimal>().unwrap();
        let target_scale = 75;
        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
                target_scale
            )
            .is_err());

        // but this is ok
        let decimal = "1.0".parse::<BigDecimal>().unwrap();
        let target_scale = 74;
        assert!(decimal
            .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
                Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
                target_scale
            )
            .is_ok());
    }
}
