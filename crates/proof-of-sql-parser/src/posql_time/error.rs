use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors related to time operations, including timezone and timestamp conversions.s
#[derive(Error, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoSQLTimestampError {
    /// Error when the timezone string provided cannot be parsed into a valid timezone.
    #[error("invalid timezone string: {timezone}")]
    InvalidTimezone {
        /// The invalid timezone
        timezone: String,
    },

    /// Error indicating an invalid timezone offset was provided.
    #[error("invalid timezone offset")]
    InvalidTimezoneOffset,

    /// Indicates a failure to convert between different representations of time units.
    #[error("Invalid time unit")]
    InvalidTimeUnit {
        /// The underlying error
        error: String,
    },

    /// The local time does not exist because there is a gap in the local time.
    /// This variant may also be returned if there was an error while resolving the local time,
    /// caused by for example missing time zone data files, an error in an OS API, or overflow.
    #[error("Local time does not exist because there is a gap in the local time")]
    LocalTimeDoesNotExist,

    /// The local time is ambiguous because there is a fold in the local time.
    /// This variant contains the two possible results, in the order (earliest, latest).
    #[error("Unix timestamp is ambiguous because there is a fold in the local time.")]
    Ambiguous {
        /// The underlying error
        error: String,
    },

    /// Represents a catch-all for parsing errors not specifically covered by other variants.
    #[error("Timestamp parsing error: {error}")]
    ParsingError {
        /// The underlying error
        error: String,
    },

    /// Represents a failure to parse a provided time unit precision value, PoSQL supports
    /// Seconds, Milliseconds, Microseconds, and Nanoseconds
    #[error("Timestamp parsing error: {error}")]
    UnsupportedPrecision {
        /// The underlying error
        error: String,
    },
}

// This exists because TryFrom<DataType> for ColumnType error is String
impl From<PoSQLTimestampError> for String {
    fn from(error: PoSQLTimestampError) -> Self {
        error.to_string()
    }
}
