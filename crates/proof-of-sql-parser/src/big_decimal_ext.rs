use bigdecimal::BigDecimal; // Import BigDecimal
use num_bigint::{BigInt, ToBigInt}; // Import BigInt and ToBigInt
use snafu::Snafu;

pub trait BigDecimalExt {
    fn precision(&self) -> u8;
    fn scale(&self) -> i8;
    fn try_into_scalar_with_precision_and_scale(
        &self,
        precision: u8,
        scale: i8,
    ) -> Result<BigInt, BigDecimalError>; // Updated return type
}

#[derive(Snafu, Debug, PartialEq)]
pub enum BigDecimalError {
    #[snafu(display("Value out of range for target type"))]
    OutOfRange,
    #[snafu(display("Fractional part of decimal is non-zero"))]
    LossyCast,
}

impl BigDecimalExt for BigDecimal {
    /// Calculate precision after trimming leading and trailing zeros
    fn precision(&self) -> u8 {
        let trimmed = self.normalized();
        trimmed.digits() as u8
    }

    /// Calculate scale (fractional digits) after trimming trailing zeros
    fn scale(&self) -> i8 {
        let trimmed = self.normalized();
        trimmed.fractional_digit_count() as i8
    }

    /// Try to convert BigDecimal to BigInt with precision and scale adjustment
    fn try_into_scalar_with_precision_and_scale(
        &self,
        target_precision: u8,
        target_scale: i8,
    ) -> Result<BigInt, BigDecimalError> {
        let scaled_decimal = self.with_scale(target_scale.into()); // Adjust scale
        if scaled_decimal.digits() > target_precision.into() {
            // Check if precision is too high
            return Err(BigDecimalError::OutOfRange);
        }

        // Convert to BigInt using to_bigint method from num-bigint
        let bigint = scaled_decimal
            .to_bigint()
            .ok_or(BigDecimalError::OutOfRange)?; // Convert to BigInt

        Ok(bigint) // Return the BigInt directly
    }
}
