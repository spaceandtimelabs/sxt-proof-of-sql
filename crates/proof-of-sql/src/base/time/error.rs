use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors related to time operations, including timezone and timestamp conversions.
#[derive(Error, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TimeError {
    /// Error indicating the timestamp unit provided is not supported.
    #[error("unsupported timestamp unit: {0}")]
    UnsupportedTimestampUnit(String),

    /// Error when the timezone string provided cannot be parsed into a valid timezone.
    #[error("invalid timezone string: {0}")]
    InvalidTimezone(String),

    /// Error indicating an invalid timezone offset was provided.
    #[error("invalid timezone offset")]
    InvalidTimezoneOffset,

    /// Error when a timezone string fails to parse correctly.
    #[error("failed to parse timezone string")]
    TimeZoneStringParseError,

    /// Error indicating a general failure to convert a timezone.
    #[error("failed to convert timezone")]
    TimeZoneConversionFailure,
}

// This exists because TryFrom<DataType> for ColumnType error is String
impl From<TimeError> for String {
    fn from(error: TimeError) -> Self {
        error.to_string()
    }
}
