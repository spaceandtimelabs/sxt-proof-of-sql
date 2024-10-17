use super::{
    DecimalError, DecimalResult,
    IntermediateDecimalError::{self, LossyCast},
    Precision,
};
use crate::base::scalar::{Scalar, ScalarConversionError};
use bigdecimal::BigDecimal;
use num_bigint::BigInt;

pub trait BigDecimalExt {
    fn precision(&self) -> u64;
    fn scale(&self) -> i64;
    fn try_into_bigint_with_precision_and_scale(
        &self,
        precision: u8,
        scale: i8,
    ) -> Result<BigInt, IntermediateDecimalError>;
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

    /// Attempts to convert the decimal to `BigInt` while adjusting it to the specified precision and scale.
    /// Returns an error if the conversion cannot be performed due to precision or scale constraints.
    ///
    /// # Errors
    /// Returns an `IntermediateDecimalError::LossyCast` error if the number of digits in the scaled decimal exceeds the specified precision.
    fn try_into_bigint_with_precision_and_scale(
        &self,
        precision: u8,
        scale: i8,
    ) -> Result<BigInt, IntermediateDecimalError> {
        if self.scale() > scale.into() {
            Err(IntermediateDecimalError::ConversionFailure)?;
        }
        let scaled_decimal = self.normalized().with_scale(scale.into());
        if scaled_decimal.digits() > precision.into() {
            return Err(LossyCast);
        }
        let (d, _) = scaled_decimal.into_bigint_and_exponent();
        Ok(d)
    }

    fn try_into_scalar_with_precision_and_scale<S: Scalar>(
        &self,
        target_precision: Precision,
        target_scale: i8,
    ) -> DecimalResult<S> {
        self.try_into_bigint_with_precision_and_scale(target_precision.value(), target_scale)?
            .try_into()
            .map_err(|e: ScalarConversionError| DecimalError::InvalidDecimal {
                error: e.to_string(),
            })
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
}
