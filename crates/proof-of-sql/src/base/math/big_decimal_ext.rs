use super::{DecimalError, DecimalResult, Precision};
use crate::base::scalar::Scalar;
use bigdecimal::BigDecimal;
use num_bigint::BigInt;

fn try_convert_bigdecimal_into_bigint_with_precision_and_scale(
    decimal: &BigDecimal,
    precision: u64,
    scale: i64,
) -> Option<BigInt> {
    let normalized = decimal.normalized();
    if normalized.fractional_digit_count() > scale {
        return None;
    }
    let scaled_decimal = normalized.with_scale(scale);
    if scaled_decimal.digits() > precision {
        return None;
    }
    Some(scaled_decimal.into_bigint_and_exponent().0)
}

pub trait BigDecimalExt {
    fn precision(&self) -> u64;
    fn scale(&self) -> i64;
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
    fn try_into_scalar_with_precision_and_scale<S: Scalar>(
        &self,
        target_precision: Precision,
        target_scale: i8,
    ) -> DecimalResult<S>;
}
impl BigDecimalExt for BigDecimal {
    /// Get the precision of the fixed-point representation of this intermediate decimal.
    #[must_use]
    fn precision(&self) -> u64 {
        self.normalized().digits()
    }

    /// Get the scale of the fixed-point representation of this intermediate decimal.
    #[must_use]
    fn scale(&self) -> i64 {
        self.normalized().fractional_digit_count()
    }

    fn try_into_scalar_with_precision_and_scale<S: Scalar>(
        &self,
        target_precision: Precision,
        target_scale: i8,
    ) -> DecimalResult<S> {
        Ok(try_convert_bigdecimal_into_bigint_with_precision_and_scale(
            self,
            target_precision.value().into(),
            target_scale.into(),
        )
        .ok_or(DecimalError::RoundingError {
            error: self.to_string(),
        })?
        .try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_valid_decimal_simple() {
        let decimal = "123.45".parse::<BigDecimal>();
        assert!(decimal.is_ok());
        let unwrapped_decimal: BigDecimal = decimal.unwrap().normalized();
        assert_eq!(unwrapped_decimal.to_string(), "123.45");
        assert_eq!(unwrapped_decimal.precision(), 5);
        assert_eq!(unwrapped_decimal.scale(), 2);
    }

    #[test]
    fn test_valid_decimal_with_leading_and_trailing_zeros() {
        let decimal = "000123.45000".parse::<BigDecimal>();
        assert!(decimal.is_ok());
        let unwrapped_decimal: BigDecimal = decimal.unwrap().normalized();
        assert_eq!(unwrapped_decimal.to_string(), "123.45");
        assert_eq!(unwrapped_decimal.precision(), 5);
        assert_eq!(unwrapped_decimal.scale(), 2);
    }

    #[test]
    fn test_accessors() {
        let decimal: BigDecimal = "123.456".parse::<BigDecimal>().unwrap().normalized();
        assert_eq!(decimal.to_string(), "123.456");
        assert_eq!(decimal.precision(), 6);
        assert_eq!(decimal.scale(), 3);
    }

    #[test]
    fn we_can_convert_bigdecimal_into_bigint_with_precision_and_scale() {
        let test_cases = vec![
            ("123.45", 2, 20, Some("12345")),
            ("123.45000", 2, 20, Some("12345")),
            ("000123.45", 2, 20, Some("12345")),
            ("123.45", 6, 20, Some("123450000")),
            ("123.45", 2, 5, Some("12345")),
            ("0.0012345", 7, 20, Some("12345")),
            ("0.0012345", 9, 20, Some("1234500")),
            ("123.45", 2, 4, None),
            ("123.45", 1, 20, None),
        ];
        for (bigdecimal_str, target_scale, target_precision, expected_result_str) in test_cases {
            let decimal = bigdecimal_str.parse::<BigDecimal>().unwrap();
            let result = try_convert_bigdecimal_into_bigint_with_precision_and_scale(
                &decimal,
                target_precision,
                target_scale,
            );
            let expected_result = expected_result_str.map(|s| s.parse::<BigInt>().unwrap());
            assert_eq!(result, expected_result);
        }
    }
}
