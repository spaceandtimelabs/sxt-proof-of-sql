use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Errors related to time operations, including timezone and timestamp conversions.
#[derive(Snafu, Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum PoSQLTimestampError {
    /// Error when the timezone string provided cannot be parsed into a valid timezone.
    #[snafu(display("invalid timezone string: {timezone}"))]
    InvalidTimezone {
        /// The invalid timezone
        timezone: String,
    },

    /// Error indicating an invalid timezone offset was provided.
    #[snafu(display("invalid timezone offset"))]
    InvalidTimezoneOffset,

    /// Indicates a failure to convert between different representations of time units.
    #[snafu(display("Invalid time unit"))]
    InvalidTimeUnit {
        /// The underlying error
        error: String,
    },

    /// Represents a failure to parse a provided time unit precision value, `PoSQL` supports
    /// Seconds, Milliseconds, Microseconds, and Nanoseconds
    #[snafu(display("Unsupported precision for timestamp: {error}"))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_errors_convert_to_display_strings() {
        let invalid_timezone: String = PoSQLTimestampError::InvalidTimezone {
            timezone: "Mars/Phobos".to_string(),
        }
        .into();
        assert_eq!(invalid_timezone, "invalid timezone string: Mars/Phobos");

        let invalid_offset: String = PoSQLTimestampError::InvalidTimezoneOffset.into();
        assert_eq!(invalid_offset, "invalid timezone offset");

        let invalid_unit: String = PoSQLTimestampError::InvalidTimeUnit {
            error: "fortnight".to_string(),
        }
        .into();
        assert_eq!(invalid_unit, "Invalid time unit");

        let unsupported_precision: String = PoSQLTimestampError::UnsupportedPrecision {
            error: "weeks".to_string(),
        }
        .into();
        assert_eq!(
            unsupported_precision,
            "Unsupported precision for timestamp: weeks"
        );
    }
}
