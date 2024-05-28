//! Module for parsing an `IntermediateDecimal` into a `Decimal75`.
use crate::{
    base::scalar::Scalar,
    sql::parse::{ConversionError, ConversionResult},
};
use proofs_sql::intermediate_decimal::IntermediateDecimal;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Copy)]
/// limit-enforced precision
pub struct Precision(u8);
pub(crate) const MAX_SUPPORTED_PRECISION: u8 = 75;

impl Precision {
    /// Constructor for creating a Precision instance
    pub fn new(value: u8) -> Result<Self, ConversionError> {
        if value > MAX_SUPPORTED_PRECISION || value == 0 {
            Err(ConversionError::PrecisionParseError(format!(
                "Failed to parse precision. Value of {} exceeds max supported precision of {}",
                value, MAX_SUPPORTED_PRECISION
            )))
        } else {
            Ok(Precision(value))
        }
    }

    /// Gets the precision as a u8 for this decimal
    pub fn value(&self) -> u8 {
        self.0
    }
}

// Custom deserializer for precision since we need to limit its value to 75
impl<'de> Deserialize<'de> for Precision {
    fn deserialize<D>(deserializer: D) -> Result<Precision, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a u8
        let value = u8::deserialize(deserializer)?;

        // Use the Precision::new method to ensure the value is within the allowed range
        Precision::new(value).map_err(serde::de::Error::custom)
    }
}

/// A decimal type that is parameterized by the scalar type
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize)]
pub struct Decimal<S: Scalar> {
    /// The raw value of the decimal as scalar
    pub value: S,
    /// The precision of the decimal
    pub precision: Precision,
    /// The scale of the decimal
    pub scale: i8,
}

impl<S: Scalar> Decimal<S> {
    /// Constructor for creating a Decimal instance
    pub fn new(value: S, precision: Precision, scale: i8) -> Self {
        Decimal {
            value,
            precision,
            scale,
        }
    }

    /// Scale the decimal to the new scale factor. Negative scaling and overflow error out.
    pub fn with_precision_and_scale(
        &self,
        new_precision: Precision,
        new_scale: i8,
    ) -> ConversionResult<Decimal<S>> {
        let scale_factor = new_scale - self.scale;
        if scale_factor < 0 || new_precision.value() < self.precision.value() + scale_factor as u8 {
            return Err(ConversionError::DecimalRoundingError(
                "Scale factor must be non-negative".to_string(),
            ));
        }
        let scaled_value = scale_scalar(self.value, scale_factor)?;
        Ok(Decimal::new(scaled_value, new_precision, new_scale))
    }

    /// Get a decimal with given precision and scale from an i64
    pub fn from_i64(value: i64, precision: Precision, scale: i8) -> ConversionResult<Self> {
        const MINIMAL_PRECISION: u8 = 19;
        let raw_precision = precision.value();
        if raw_precision < MINIMAL_PRECISION {
            return Err(ConversionError::DecimalRoundingError(
                "Precision must be at least 19".to_string(),
            ));
        }
        if scale < 0 || raw_precision < MINIMAL_PRECISION + scale as u8 {
            return Err(ConversionError::DecimalRoundingError(
                "Can not scale down a decimal".to_string(),
            ));
        }
        let scaled_value = scale_scalar(S::from(&value), scale)?;
        Ok(Decimal::new(scaled_value, precision, scale))
    }

    /// Get a decimal with given precision and scale from an i128
    pub fn from_i128(value: i128, precision: Precision, scale: i8) -> ConversionResult<Self> {
        const MINIMAL_PRECISION: u8 = 39;
        let raw_precision = precision.value();
        if raw_precision < MINIMAL_PRECISION {
            return Err(ConversionError::DecimalRoundingError(
                "Precision must be at least 19".to_string(),
            ));
        }
        if scale < 0 || raw_precision < MINIMAL_PRECISION + scale as u8 {
            return Err(ConversionError::DecimalRoundingError(
                "Can not scale down a decimal".to_string(),
            ));
        }
        let scaled_value = scale_scalar(S::from(&value), scale)?;
        Ok(Decimal::new(scaled_value, precision, scale))
    }
}

/// Fallibly attempts to convert an `IntermediateDecimal` into the
/// native proof-of-sql [Scalar] backing store. This function adjusts
/// the decimal to the specified `target_precision` and `target_scale`,
/// and validates that the adjusted decimal does not exceed the specified precision.
/// If the conversion is successful, it returns the `Scalar` representation;
/// otherwise, it returns a `ConversionError` indicating the type of failure
/// (e.g., exceeding precision limits).
///
/// ## Arguments
/// * `d` - The `IntermediateDecimal` to convert.
/// * `target_precision` - The maximum number of digits the scalar can represent.
/// * `target_scale` - The scale (number of decimal places) to use in the scalar.
///
/// ## Errors
/// Returns `ConversionError::PrecisionParseError` if the number of digits in
/// the decimal exceeds the `target_precision` after adjusting for `target_scale`.
pub(crate) fn try_into_to_scalar<S: Scalar>(
    d: &IntermediateDecimal,
    target_precision: Precision,
    target_scale: i8,
) -> Result<S, ConversionError> {
    d.try_into_bigint_with_precision_and_scale(target_precision.value(), target_scale)?
        .try_into()
}

/// Scale scalar by the given scale factor. Negative scaling is not allowed.
/// Note that we do not check for overflow.
pub(crate) fn scale_scalar<S: Scalar>(s: S, scale: i8) -> ConversionResult<S> {
    if scale < 0 {
        return Err(ConversionError::DecimalRoundingError(
            "Scale factor must be non-negative".to_string(),
        ));
    }
    let ten = S::from(10);
    let mut res = s;
    for _ in 0..scale {
        res *= ten;
    }
    Ok(res)
}

#[cfg(test)]
mod scale_adjust_test {

    use super::*;
    use crate::base::scalar::Curve25519Scalar;
    use num_bigint::BigInt;

    #[test]
    fn we_cannot_scale_past_max_precision() {
        let decimal = "12345678901234567890123456789012345678901234567890123456789012345678900.0"
            .parse()
            .unwrap();

        let target_scale = 5;

        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(decimal.value().digits() as u8).unwrap(),
            target_scale
        )
        .is_err());
    }

    #[test]
    fn we_can_match_exact_decimals_from_queries_to_db() {
        let decimal: IntermediateDecimal = "123.45".parse().unwrap();
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
        let result = try_into_to_scalar::<Curve25519Scalar>(
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

        let limbs = try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(decimal.value().digits() as u8).unwrap(),
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
        let limbs = try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(decimal.value().digits() as u8).unwrap(),
            target_scale,
        )
        .unwrap();
        assert_eq!(limbs, -Curve25519Scalar::from(expected_limbs));
    }

    #[test]
    fn we_can_match_decimals_at_extrema() {
        // a big decimal cannot scale up past the supported precision
        let decimal = "1234567890123456789012345678901234567890123456789012345678901234567890.0"
            .parse()
            .unwrap();
        let target_scale = 6; // now precision exceeds maximum
        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(decimal.value().digits() as u8,).unwrap(),
            target_scale
        )
        .is_err());

        // maximum decimal value we can support
        let decimal =
            "99999999999999999999999999999999999999999999999999999999999999999999999999.0"
                .parse()
                .unwrap();
        let target_scale = 1;
        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_ok());

        // scaling larger than max will fail
        let decimal =
            "999999999999999999999999999999999999999999999999999999999999999999999999999.0"
                .parse()
                .unwrap();
        let target_scale = 1;
        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_err());

        // smallest possible decimal value we can support (either signed/unsigned)
        let decimal =
            "0.000000000000000000000000000000000000000000000000000000000000000000000000001"
                .parse()
                .unwrap();
        let target_scale = MAX_SUPPORTED_PRECISION as i8;
        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(decimal.value().digits() as u8,).unwrap(),
            target_scale
        )
        .is_ok());

        // this is ok because it can be scaled to 75 precision
        let decimal = "0.1".parse().unwrap();
        let target_scale = MAX_SUPPORTED_PRECISION as i8;
        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_ok());

        // this exceeds max precision
        let decimal = "1.0".parse().unwrap();
        let target_scale = 75;
        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(decimal.value().digits() as u8,).unwrap(),
            target_scale
        )
        .is_err());

        // but this is ok
        let decimal = "1.0".parse().unwrap();
        let target_scale = 74;
        assert!(try_into_to_scalar::<Curve25519Scalar>(
            &decimal,
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_ok());
    }
}
