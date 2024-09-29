//! A parser conforming to standard postgreSQL to parse the precision and scale
//! from a decimal token obtained from the lalrpop lexer. This module
//! exists to resolve a cyclic dependency between proof-of-sql
//! and proof-of-sql-parser.
//!
//! A decimal must have a decimal point. The lexer does not route
//! whole integers to this contructor.
use crate::intermediate_decimal::IntermediateDecimalError::{LossyCast, OutOfRange, ParseError};
use alloc::string::String;
use bigdecimal::{num_bigint::BigInt, BigDecimal, ParseBigDecimalError, ToPrimitive};
use core::{fmt, hash::Hash, str::FromStr};
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use proof_of_sql::base::math::big_decimal_ext::BigDecimalExt;

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
impl From<ParseBigDecimalError> for IntermediateDecimalError {
    fn from(value: ParseBigDecimalError) -> Self {
        IntermediateDecimalError::ParseError { error: value }
    }
}

impl Eq for IntermediateDecimalError {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_valid_decimal_simple() {
        let decimal = "123.45".parse();
        assert!(decimal.is_ok());
        let unwrapped_decimal: BigDecimal = decimal.unwrap();
        assert_eq!(unwrapped_decimal.to_string(), "123.45");
        assert_eq!(unwrapped_decimal.precision(), 5);
        assert_eq!(unwrapped_decimal.scale(), 2);
    }

    #[test]
    fn test_valid_decimal_with_leading_and_trailing_zeros() {
        let decimal = "000123.45000".parse();
        assert!(decimal.is_ok());
        let unwrapped_decimal: BigDecimal = decimal.unwrap();
        assert_eq!(unwrapped_decimal.to_string(), "123.45");
        assert_eq!(unwrapped_decimal.precision(), 5);
        assert_eq!(unwrapped_decimal.scale(), 2);
    }

    #[test]
    fn test_accessors() {
        let decimal: BigDecimal = "123.456".parse().unwrap();
        assert_eq!(decimal.to_string(), "123.456");
        assert_eq!(decimal.precision(), 6);
        assert_eq!(decimal.scale(), 3);
    }

    
   
 
}
