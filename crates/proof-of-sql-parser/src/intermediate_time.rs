use chrono::{offset::LocalResult, DateTime, TimeZone, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from parsing a query string into an intermediate timestamp
#[derive(Error, Debug, PartialEq, Eq)]
pub enum IntermediateTimestampError {
    /// Indicates a failure to convert between different representations of time units.
    #[error("Invalid time unit")]
    InvalidTimeUnit,

    /// Indicates a failure to convert or parse timezone data correctly.
    #[error("Invalid timezone")]
    InvalidTimeZone,

    /// Indicates that the timestamp string does not match an expected format.
    #[error("Invalid timestamp format: {0}")]
    InvalidFormat(String),

    /// The local time does not exist because there is a gap in the local time.
    /// This variant may also be returned if there was an error while resolving the local time,
    /// caused by for example missing time zone data files, an error in an OS API, or overflow.
    #[error("Local time does not exist because there is a gap in the local time")]
    LocalTimeDoesNotExist,

    /// The local time is ambiguous because there is a fold in the local time.
    /// This variant contains the two possible results, in the order (earliest, latest).
    #[error("Unix timestamp is ambiguous because there is a fold in the local time.")]
    Ambiguous,

    /// Represents a catch-all for parsing errors not specifically covered by other variants.
    #[error("Timestamp parsing error: {0}")]
    ParsingError(String),
}

/// An initermediate type of components extracted from a timestamp string.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntermediateTimeUnit {
    /// Represents seconds with precision 0: ex "2024-06-20 12:34:56"
    Second,
    /// Represents milliseconds with precision 3: ex "2024-06-20 12:34:56.123"
    Millisecond,
    /// Represents microseconds with precision 6: ex "2024-06-20 12:34:56.123456"
    Microsecond,
    /// Represents nanoseconds with precision 9: ex "2024-06-20 12:34:56.123456789"
    Nanosecond,
}

impl fmt::Display for IntermediateTimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntermediateTimeUnit::Second => write!(f, "Second"),
            IntermediateTimeUnit::Millisecond => write!(f, "Millisecond"),
            IntermediateTimeUnit::Microsecond => write!(f, "Microsecond"),
            IntermediateTimeUnit::Nanosecond => write!(f, "Nanosecond"),
        }
    }
}

/// Captures a timezone from a timestamp query
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntermediateTimeZone {
    /// Default variant for UTC timezone
    Utc,
    /// TImezone offset in seconds
    FixedOffset(i32),
}

impl IntermediateTimeZone {
    /// Parse a timezone from a count of seconds
    pub fn from_offset(offset: i32) -> Self {
        if offset == 0 {
            IntermediateTimeZone::Utc
        } else {
            IntermediateTimeZone::FixedOffset(offset)
        }
    }
}

impl fmt::Display for IntermediateTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntermediateTimeZone::Utc => write!(f, "Z"),
            IntermediateTimeZone::FixedOffset(offset) => {
                if *offset == 0 {
                    write!(f, "Z")
                } else {
                    let total_minutes = offset / 60;
                    let hours = total_minutes / 60;
                    let minutes = total_minutes.abs() % 60;
                    write!(f, "{:+03}:{:02}", hours, minutes)
                }
            }
        }
    }
}

/// Represents a fully parsed timestamp with detailed time unit and timezone information.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntermediateTimestamp {
    /// The datetime representation in UTC.
    pub timestamp: DateTime<Utc>,

    /// The precision of the datetime value, e.g., seconds, milliseconds.
    pub timeunit: IntermediateTimeUnit,

    /// The timezone of the datetime, either UTC or a fixed offset from UTC.
    pub timezone: IntermediateTimeZone,
}

impl IntermediateTimestamp {
    /// Attempts to parse a timestamp string into an `IntermediateTimestamp` structure.
    /// This function supports two primary formats:
    ///
    /// 1. **RFC 3339 Parsing**:
    ///    - Parses the timestamp along with its timezone.
    ///    - If parsing succeeds, it extracts the timezone offset using `dt.offset().local_minus_utc()`
    ///      and then uses this to construct the appropriate `IntermediateTimeZone`.
    ///
    /// 2. **Timezone Parsing and Conversion**:
    ///    - The `from_offset` method is used to determine whether the timezone should be represented
    ///      as `Utc` or `FixedOffset`. This function simplifies the decision based on the offset value.
    ///
    /// # Examples
    /// ```
    /// use chrono::{DateTime, Utc};
    /// use proof_of_sql_parser::intermediate_time::{IntermediateTimestamp, IntermediateTimeZone};
    ///
    /// // Parsing an RFC 3339 timestamp without a timezone:
    /// let timestamp_str = "2009-01-03T18:15:05Z";
    /// let intermediate_timestamp = IntermediateTimestamp::try_from(timestamp_str).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone, IntermediateTimeZone::Utc);
    ///
    /// // Parsing an RFC 3339 timestamp with a positive timezone offset:
    /// let timestamp_str_with_tz = "2009-01-03T18:15:05+03:00";
    /// let intermediate_timestamp = IntermediateTimestamp::try_from(timestamp_str_with_tz).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone, IntermediateTimeZone::FixedOffset(10800)); // 3 hours in seconds
    /// ```
    pub fn try_from(timestamp_str: &str) -> Result<Self, IntermediateTimestampError> {
        DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| {
                let offset_seconds = dt.offset().local_minus_utc();
                IntermediateTimestamp {
                    timestamp: dt.with_timezone(&Utc),
                    timeunit: IntermediateTimeUnit::Second,
                    timezone: IntermediateTimeZone::from_offset(offset_seconds),
                }
            })
            .map_err(|e| IntermediateTimestampError::ParsingError(e.to_string()))
    }

    /// Attempts to parse a timestamp string into an `IntermediateTimestamp` structure.
    /// This function supports two primary formats:
    ///
    /// 1. **Unix Epoch Time Parsing**:
    ///    - Since Unix epoch timestamps don't inherently carry timezone information,
    ///      any Unix time parsed directly from an integer is assumed to be in UTC.
    ///
    /// # Examples
    /// ```
    /// use chrono::{DateTime, Utc};
    /// use proof_of_sql_parser::intermediate_time::{IntermediateTimestamp, IntermediateTimeZone};
    ///
    /// // Parsing a Unix epoch timestamp (assumed to be seconds and UTC):
    /// let unix_time_str = "1231006505";
    /// let intermediate_timestamp = IntermediateTimestamp::to_timestamp(unix_time_str).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone, IntermediateTimeZone::Utc);
    /// ```
    pub fn to_timestamp(timestamp_str: &str) -> Result<Self, IntermediateTimestampError> {
        timestamp_str
            .parse::<i64>()
            .map_err(|e| IntermediateTimestampError::InvalidFormat(e.to_string()))
            .and_then(|epoch| match Utc.timestamp_opt(epoch, 0) {
                LocalResult::Single(timestamp) => Ok(IntermediateTimestamp {
                    timestamp,
                    timeunit: IntermediateTimeUnit::Second,
                    timezone: IntermediateTimeZone::Utc,
                }),
                LocalResult::Ambiguous(_, _) => Err(IntermediateTimestampError::Ambiguous),
                LocalResult::None => Err(IntermediateTimestampError::LocalTimeDoesNotExist),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_utc_timezone() {
        let input = "2023-06-26T12:34:56Z";
        let expected_timezone = IntermediateTimeZone::Utc;
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_positive_offset_timezone() {
        let input = "2023-06-26T12:34:56+03:30";
        let expected_timezone = IntermediateTimeZone::FixedOffset(12600); // 3 hours and 30 minutes in seconds
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_negative_offset_timezone() {
        let input = "2023-06-26T12:34:56-04:00";
        let expected_timezone = IntermediateTimeZone::FixedOffset(-14400); // -4 hours in seconds
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_zero_offset_timezone() {
        let input = "2023-06-26T12:34:56+00:00";
        let expected_timezone = IntermediateTimeZone::Utc; // Zero offset defaults to UTC
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_unix_epoch_time_timezone() {
        let unix_time = 1_593_000_000.to_string(); // Unix time as string
        let expected_timezone = IntermediateTimeZone::Utc; // Unix time should always be UTC
        let result = IntermediateTimestamp::to_timestamp(&unix_time).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_unix_epoch_timestamp_parsing() {
        let unix_time = 1_593_000_000; // Example Unix timestamp (seconds since epoch)
        let expected_datetime = Utc.timestamp_opt(unix_time, 0).unwrap();
        let expected_unit = IntermediateTimeUnit::Second; // Assuming basic second precision for Unix timestamp
        let input = unix_time.to_string(); // Simulate input as string since Unix times are often transmitted as strings
        let result = IntermediateTimestamp::to_timestamp(&input).unwrap();

        assert_eq!(result.timestamp, expected_datetime);
        assert_eq!(result.timeunit, expected_unit);
    }

    #[test]
    fn test_basic_rfc3339_timestamp() {
        let input = "2023-06-26T12:34:56Z";
        let expected = Utc.with_ymd_and_hms(2023, 6, 26, 12, 34, 56).unwrap();
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_rfc3339_timestamp_with_positive_offset() {
        let input = "2023-06-26T08:00:00+04:30";
        let expected = Utc.with_ymd_and_hms(2023, 6, 26, 3, 30, 0).unwrap(); // Adjusted to UTC
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_rfc3339_timestamp_with_negative_offset() {
        let input = "2023-06-26T20:00:00-05:00";
        let expected = Utc.with_ymd_and_hms(2023, 6, 27, 1, 0, 0).unwrap(); // Adjusted to UTC
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_rfc3339_timestamp_with_utc_designator() {
        let input = "2023-06-26T12:34:56Z";
        let expected = Utc.with_ymd_and_hms(2023, 6, 26, 12, 34, 56).unwrap();
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_invalid_rfc3339_timestamp() {
        let input = "not-a-timestamp";
        assert!(IntermediateTimestamp::try_from(input).is_err());
    }

    #[test]
    fn test_timestamp_with_seconds() {
        let input = "2023-06-26T12:34:56Z";
        let expected_time = Utc.with_ymd_and_hms(2023, 6, 26, 12, 34, 56).unwrap();
        let expected_unit = IntermediateTimeUnit::Second;
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected_time);
        assert_eq!(result.timeunit, expected_unit);
    }

    #[test]
    #[allow(deprecated)]
    fn test_rfc3339_timestamp_with_milliseconds() {
        let input = "2023-06-26T12:34:56.123Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_milli(12, 34, 56, 123);
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    #[allow(deprecated)]
    fn test_rfc3339_timestamp_with_microseconds() {
        let input = "2023-06-26T12:34:56.123456Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_micro(12, 34, 56, 123456);
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }
    #[test]
    #[allow(deprecated)]
    fn test_rfc3339_timestamp_with_nanoseconds() {
        let input = "2023-06-26T12:34:56.123456789Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_nano(12, 34, 56, 123456789);
        let result = IntermediateTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_general_parsing_error() {
        // This test assumes that there's a catch-all parsing error case that isn't covered by the more specific errors.
        let malformed_input = "2009-01-03T::00Z"; // Intentionally malformed timestamp
        let result = IntermediateTimestamp::try_from(malformed_input);
        assert!(matches!(
            result,
            Err(IntermediateTimestampError::ParsingError(_))
        ));
    }

    #[test]
    fn test_basic_date_time_support() {
        let inputs = ["2009-01-03T18:15:05Z", "2009-01-03T18:15:05+02:00"];
        for input in inputs {
            assert!(
                DateTime::parse_from_rfc3339(input).is_ok(),
                "Should parse correctly: {}",
                input
            );
        }
    }

    #[test]
    fn test_leap_seconds() {
        let input = "1998-12-31T23:59:60Z"; // fyi the 59:-->60<-- is the leap second
        assert!(DateTime::parse_from_rfc3339(input).is_ok());
    }

    #[test]
    fn test_rejecting_incorrect_formats() {
        let incorrect_formats = [
            "2009-January-03",
            "25:61:61",
            "20090103",
            "181505",
            "18:15:05",
        ];
        for input in incorrect_formats {
            assert!(
                DateTime::parse_from_rfc3339(input).is_err(),
                "Should reject incorrect format: {}",
                input
            );
        }
    }
}
