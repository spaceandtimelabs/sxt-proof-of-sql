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
