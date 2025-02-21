use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Errors related to time operations, including timezone and timestamp conversions.
#[allow(clippy::module_name_repetitions)]
#[derive(Snafu, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

    /// The local time does not exist because there is a gap in the local time.
    /// This variant may also be returned if there was an error while resolving the local time,
    /// caused by for example missing time zone data files, an error in an OS API, or overflow.
    #[snafu(display("Local time does not exist because there is a gap in the local time"))]
    LocalTimeDoesNotExist,

    /// The local time is ambiguous because there is a fold in the local time.
    /// This variant contains the two possible results, in the order (earliest, latest).
    #[snafu(display("Unix timestamp is ambiguous because there is a fold in the local time."))]
    Ambiguous {
        /// The underlying error
        error: String,
    },

    /// Represents a catch-all for parsing errors not specifically covered by other variants.
    #[snafu(display("Timestamp parsing error: {error}"))]
    ParsingError {
        /// The underlying error
        error: String,
    },

    /// Represents a failure to parse a provided time unit precision value, `PoSQL` supports
    /// Seconds, Milliseconds, Microseconds, and Nanoseconds
    #[snafu(display("Unsupported precision for timestamp:: {error}"))]
    UnsupportedPrecision {
        /// The underlying error
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_parsing_error() {
        let error = PoSQLTimestampError::ParsingError {
            error: "test error".into(),
        };
        assert_eq!(error.to_string(), "Unable to parse timestamp");
    }

    #[test]
    fn test_invalid_timezone() {
        let error = PoSQLTimestampError::InvalidTimezone {
            timezone: "invalid".into(),
        };
        assert_eq!(error.to_string(), "Invalid timezone: invalid");
    }

    #[test]
    fn test_invalid_timezone_offset() {
        let error = PoSQLTimestampError::InvalidTimezoneOffset;
        assert_eq!(error.to_string(), "Invalid timezone offset");
    }

    #[test]
    fn test_unsupported_precision() {
        let error = PoSQLTimestampError::UnsupportedPrecision {
            error: "7".into(),
        };
        assert_eq!(error.to_string(), "Unsupported precision: 7");
    }

    #[test]
    fn test_local_time_does_not_exist() {
        let error = PoSQLTimestampError::LocalTimeDoesNotExist;
        assert_eq!(error.to_string(), "Local time does not exist");
    }

    #[test]
    fn test_ambiguous() {
        let error = PoSQLTimestampError::Ambiguous {
            error: "test error".into(),
        };
        assert_eq!(error.to_string(), "Ambiguous local time: test error");
    }

    #[test]
    fn test_error_equality() {
        let error1 = PoSQLTimestampError::ParsingError {
            error: "test error".into(),
        };
        let error2 = PoSQLTimestampError::ParsingError {
            error: "test error".into(),
        };
        let error3 = PoSQLTimestampError::ParsingError {
            error: "different error".into(),
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}
