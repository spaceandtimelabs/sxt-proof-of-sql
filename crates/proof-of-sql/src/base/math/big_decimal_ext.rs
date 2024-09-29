use bigdecimal::{BigDecimal, ToPrimitive, num_bigint::BigInt};
use snafu::Snafu;

pub trait BigDecimalExt {
    fn precision(&self) -> u8;
    fn scale(&self) -> i8;
    fn try_into_scalar_with_precision_and_scale(
        &self,
        precision: u8,
        scale: i8,
    ) -> Result<BigInt, BigDecimalError>;
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
    fn try_into_scalar_with_precision_and_scale<S: Scalar>(
        &self,
        target_precision: u8,
        target_scale: i8,
    ) -> Result<S, DecimalError> {
        let scaled_decimal = self.with_scale(target_scale.into());  // Adjust scale
        if scaled_decimal.digits() > target_precision.into() {  // Check if precision is too high
            return Err(DecimalError::InvalidDecimal { error: "Precision exceeded".to_string() });
        }
        
        let bigint = scaled_decimal.into_bigint(); // Convert to BigInt
        // Try converting BigInt into the target scalar type `S`
        S::from_bigint(bigint).map_err(|e| DecimalError::InvalidDecimal {
            error: e.to_string(),
        })
    }
}


impl fmt::Display for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl FromStr for BigDecimal {
    type Err = IntermediateDecimalError;

    fn from_str(decimal_string: &str) -> Result<Self, Self::Err> {
        BigDecimal::from_str(decimal_string)
            .map(|value|  value.normalized())
            .map_err(|err| ParseError { error: err })
    }
}



impl TryFrom<&str> for BigDecimal {
    type Error = IntermediateDecimalError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        BigDecimal::from_str(s)
    }
}

impl TryFrom<String> for BigDecimal {
    type Error = IntermediateDecimalError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        BigDecimal::from_str(&s)
    }
}

impl TryFrom<BigDecimal> for i128 {
    type Error = IntermediateDecimalError;

    fn try_from(decimal: BigDecimal) -> Result<Self, Self::Error> {
        if !decimal.value.is_integer() {
            return Err(LossyCast);
        }

        match decimal.value.to_i128() {
            Some(value) if (i128::MIN..=i128::MAX).contains(&value) => Ok(value),
            _ => Err(OutOfRange),
        }
    }
}

impl TryFrom<BigDecimal> for i64 {
    type Error = IntermediateDecimalError;

    fn try_from(decimal: BigDecimal) -> Result<Self, Self::Error> {
        if !decimal.value.is_integer() {
            return Err(LossyCast);
        }

        match decimal.value.to_i64() {
            Some(value) if (i64::MIN..=i64::MAX).contains(&value) => Ok(value),
            _ => Err(OutOfRange),
        }
    }
}
