use super::decimal::{IntermediateDecimalError, IntermediateDecimalError::LossyCast};
use bigdecimal::BigDecimal;
use num_bigint::BigInt;

pub trait IntermediateDecimalExt {
    fn precision(&self) -> u64;
    fn scale(&self) -> i64;
    fn try_into_bigint_with_precision_and_scale(
        &self,
        precision: u8,
        scale: i8,
    ) -> Result<BigInt, IntermediateDecimalError>;
}
impl IntermediateDecimalExt for BigDecimal {
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
