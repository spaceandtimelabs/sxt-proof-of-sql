use super::InvalidPrecisionError;
use crate::base::scalar::ScalarConversionError;
use alloc::string::{String, ToString};
use bigdecimal::ParseBigDecimalError;
use snafu::Snafu;

/// Errors related to decimal operations.
#[derive(Snafu, Debug, PartialEq)]
pub enum DecimalError {
    #[snafu(transparent)]
    /// Error when a decimal format or value is incorrect,
    /// the string isn't even a decimal e.g. "notastring",
    /// "-21.233.122" etc aka `InvalidDecimal`
    InvalidDecimal {
        /// The underlying error
        source: ScalarConversionError,
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

    /// Represents an error encountered during the parsing of a decimal string.
    #[snafu(display("{error}"))]
    ParseError {
        /// The underlying error
        error: ParseBigDecimalError,
    },
}

impl Eq for DecimalError {}

/// Result type for decimal operations.
pub type DecimalResult<T> = Result<T, DecimalError>;

// This exists because `TryFrom<arrow::datatypes::DataType>` for `ColumnType` error is String
impl From<DecimalError> for String {
    fn from(error: DecimalError) -> Self {
        error.to_string()
    }
}
