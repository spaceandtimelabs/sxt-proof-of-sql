use arrow::datatypes::TimeUnit as ArrowTimeUnit;
use chrono::{offset::LocalResult, DateTime, FixedOffset, TimeZone, Utc};
use chrono_tz::Tz;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use thiserror::Error;

/// Errors related to time operations, including timezone and timestamp conversions.
#[derive(Error, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoSQLTimestampError {
    /// Error when the timezone string provided cannot be parsed into a valid timezone.
    #[error("invalid timezone string: {0}")]
    InvalidTimezone(String),

    /// Error indicating an invalid timezone offset was provided.
    #[error("invalid timezone offset")]
    InvalidTimezoneOffset,

    /// Error indicating a general failure to convert a timezone.
    #[error("failed to convert timezone")]
    TimeZoneConversionFailure(String),

    /// Indicates a failure to convert between different representations of time units.
    #[error("Invalid time unit")]
    InvalidTimeUnit(String),

    /// Indicates that the timestamp string does not match an expected format.
    #[error("Invalid timestamp format: {0}")]
    InvalidTimestampFormat(String),

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

// This exists because TryFrom<DataType> for ColumnType error is String
impl From<PoSQLTimestampError> for String {
    fn from(error: PoSQLTimestampError) -> Self {
        error.to_string()
    }
}

/// An intermediate type representing the time units from a parsed query
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoSQLTimeUnit {
    /// Represents seconds with precision 0: ex "2024-06-20 12:34:56"
    Second,
    /// Represents milliseconds with precision 3: ex "2024-06-20 12:34:56.123"
    Millisecond,
    /// Represents microseconds with precision 6: ex "2024-06-20 12:34:56.123456"
    Microsecond,
    /// Represents nanoseconds with precision 9: ex "2024-06-20 12:34:56.123456789"
    Nanosecond,
}

impl fmt::Display for PoSQLTimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoSQLTimeUnit::Second => write!(f, "Second"),
            PoSQLTimeUnit::Millisecond => write!(f, "Millisecond"),
            PoSQLTimeUnit::Microsecond => write!(f, "Microsecond"),
            PoSQLTimeUnit::Nanosecond => write!(f, "Nanosecond"),
        }
    }
}

/// Represents a fully parsed timestamp with detailed time unit and timezone information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PoSQLTimestamp {
    /// The datetime representation in UTC.
    pub timestamp: DateTime<Utc>,

    /// The precision of the datetime value, e.g., seconds, milliseconds.
    pub timeunit: PoSQLTimeUnit,

    /// The timezone of the datetime, either UTC or a fixed offset from UTC.
    pub timezone: PoSQLTimeZone,
}

impl PoSQLTimestamp {
    /// Attempts to parse a timestamp string into an [PoSQLTimestamp] structure.
    /// This function supports two primary formats:
    ///
    /// 1. **RFC 3339 Parsing**:
    ///    - Parses the timestamp along with its timezone.
    ///    - If parsing succeeds, it extracts the timezone offset using `dt.offset().local_minus_utc()`
    ///      and then uses this to construct the appropriate `PoSQLTimeZone`.
    ///
    /// 2. **Timezone Parsing and Conversion**:
    ///    - The `from_offset` method is used to determine whether the timezone should be represented
    ///      as `Utc` or `FixedOffset`. This function simplifies the decision based on the offset value.
    ///
    /// # Examples
    /// ```
    /// use chrono::{DateTime, Utc};
    /// use proof_of_sql_parser::intermediate_time::{PoSQLTimestamp, PoSQLTimeZone};
    ///
    /// // Parsing an RFC 3339 timestamp without a timezone:
    /// let timestamp_str = "2009-01-03T18:15:05Z";
    /// let intermediate_timestamp = PoSQLTimestamp::try_from(timestamp_str).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone, PoSQLTimeZone::Utc);
    ///
    /// // Parsing an RFC 3339 timestamp with a positive timezone offset:
    /// let timestamp_str_with_tz = "2009-01-03T18:15:05+03:00";
    /// let intermediate_timestamp = PoSQLTimestamp::try_from(timestamp_str_with_tz).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone, PoSQLTimeZone::FixedOffset(10800)); // 3 hours in seconds
    /// ```
    pub fn try_from(timestamp_str: &str) -> Result<Self, PoSQLTimestampError> {
        let dt = DateTime::parse_from_rfc3339(timestamp_str)
            .map_err(|e| PoSQLTimestampError::ParsingError(e.to_string()))?;

        let offset_seconds = dt.offset().local_minus_utc();
        let timezone = PoSQLTimeZone::from_offset(offset_seconds)
            .map_err(|e| PoSQLTimestampError::TimeZoneConversionFailure(e.to_string()))?;

        Ok(PoSQLTimestamp {
            timestamp: dt.with_timezone(&Utc),
            timeunit: PoSQLTimeUnit::Second,
            timezone,
        })
    }

    /// Attempts to parse a timestamp string into an `PoSQLTimestamp` structure.
    /// This function supports two primary formats:
    ///
    /// 1. **Unix Epoch Time Parsing**:
    ///    - Since Unix epoch timestamps don't inherently carry timezone information,
    ///      any Unix time parsed directly from an integer is assumed to be in UTC.
    ///
    /// # Examples
    /// ```
    /// use chrono::{DateTime, Utc};
    /// use proof_of_sql_parser::intermediate_time::{PoSQLTimestamp, PoSQLTimeZone};
    ///
    /// // Parsing a Unix epoch timestamp (assumed to be seconds and UTC):
    /// let unix_time = 1231006505;
    /// let intermediate_timestamp = PoSQLTimestamp::to_timestamp(unix_time).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone, PoSQLTimeZone::Utc);
    /// ```
    pub fn to_timestamp(epoch: i64) -> Result<Self, PoSQLTimestampError> {
        match Utc.timestamp_opt(epoch, 0) {
            LocalResult::Single(timestamp) => Ok(PoSQLTimestamp {
                timestamp,
                timeunit: PoSQLTimeUnit::Second,
                timezone: PoSQLTimeZone::UTC,
            }),
            LocalResult::Ambiguous(_, _) => Err(PoSQLTimestampError::Ambiguous),
            LocalResult::None => Err(PoSQLTimestampError::LocalTimeDoesNotExist),
        }
    }
}

impl From<PoSQLTimeUnit> for ArrowTimeUnit {
    fn from(unit: PoSQLTimeUnit) -> Self {
        match unit {
            PoSQLTimeUnit::Second => ArrowTimeUnit::Second,
            PoSQLTimeUnit::Millisecond => ArrowTimeUnit::Millisecond,
            PoSQLTimeUnit::Microsecond => ArrowTimeUnit::Microsecond,
            PoSQLTimeUnit::Nanosecond => ArrowTimeUnit::Nanosecond,
        }
    }
}

impl From<ArrowTimeUnit> for PoSQLTimeUnit {
    fn from(unit: ArrowTimeUnit) -> Self {
        match unit {
            ArrowTimeUnit::Second => PoSQLTimeUnit::Second,
            ArrowTimeUnit::Millisecond => PoSQLTimeUnit::Millisecond,
            ArrowTimeUnit::Microsecond => PoSQLTimeUnit::Microsecond,
            ArrowTimeUnit::Nanosecond => PoSQLTimeUnit::Nanosecond,
        }
    }
}

/// A typed TimeZone for a [`TimeStamp`]. It is optionally
/// used to define a timezone other than UTC for a new TimeStamp.
/// It exists as a wrapper around chrono-tz because chrono-tz does
/// not implement uniform bit distribution
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PoSQLTimeZone(Tz);

impl PoSQLTimeZone {
    /// Convenience constant for the UTC timezone
    pub const UTC: PoSQLTimeZone = PoSQLTimeZone(Tz::UTC);

    /// Create a new ProofsTimeZone from a chrono TimeZone
    pub fn new(tz: Tz) -> Self {
        PoSQLTimeZone(tz)
    }

    /// Create a timezone from a fixed offset in seconds
    pub fn from_offset(seconds: i32) -> Result<Self, PoSQLTimestampError> {
        FixedOffset::east_opt(seconds)
            .ok_or(PoSQLTimestampError::InvalidTimezoneOffset)
            .and_then(|offset| {
                let datetime: DateTime<Utc> = Utc::now();
                let offset_datetime = datetime.with_timezone(&offset);
                let tz_string = offset_datetime.format("%Z").to_string();
                Tz::from_str(&tz_string)
                    .map(PoSQLTimeZone)
                    .map_err(|_| PoSQLTimestampError::InvalidTimezone(tz_string))
            })
    }
}

impl From<&PoSQLTimeZone> for Arc<str> {
    fn from(timezone: &PoSQLTimeZone) -> Self {
        Arc::from(timezone.0.name())
    }
}

impl From<Tz> for PoSQLTimeZone {
    fn from(tz: Tz) -> Self {
        PoSQLTimeZone(tz)
    }
}

impl fmt::Display for PoSQLTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<Option<Arc<str>>> for PoSQLTimeZone {
    type Error = PoSQLTimestampError;

    fn try_from(value: Option<Arc<str>>) -> Result<Self, Self::Error> {
        match value {
            Some(arc_str) => Tz::from_str(&arc_str).map(PoSQLTimeZone).map_err(|_| {
                PoSQLTimestampError::InvalidTimezone("Invalid timezone string".to_string())
            }),
            None => Ok(PoSQLTimeZone(Tz::UTC)), // Default to UTC
        }
    }
}

impl TryFrom<&str> for PoSQLTimeZone {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Tz::from_str(value)
            .map(PoSQLTimeZone)
            .map_err(|_| "Invalid timezone string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Tz;

    #[test]
    fn valid_timezones_convert_correctly() {
        let valid_timezones = ["Europe/London", "America/New_York", "Asia/Tokyo", "UTC"];

        for tz_str in &valid_timezones {
            let arc_tz = Arc::new(tz_str.to_string());
            // Convert Arc<String> to Arc<str> by dereferencing to &str then creating a new Arc
            let arc_tz_str: Arc<str> = Arc::from(&**arc_tz);
            let timezone = PoSQLTimeZone::try_from(Some(arc_tz_str));
            assert!(timezone.is_ok(), "Timezone should be valid: {}", tz_str);
            assert_eq!(
                timezone.unwrap().0,
                Tz::from_str(tz_str).unwrap(),
                "Timezone mismatch for {}",
                tz_str
            );
        }
    }

    #[test]
    fn test_edge_timezone_strings() {
        let edge_timezones = ["Etc/GMT+12", "Etc/GMT-14", "America/Argentina/Ushuaia"];
        for tz_str in &edge_timezones {
            let arc_tz = Arc::from(*tz_str);
            let result = PoSQLTimeZone::try_from(Some(arc_tz));
            assert!(result.is_ok(), "Edge timezone should be valid: {}", tz_str);
            assert_eq!(
                result.unwrap().0,
                Tz::from_str(tz_str).unwrap(),
                "Mismatch for edge timezone {}",
                tz_str
            );
        }
    }

    #[test]
    fn test_empty_timezone_string() {
        let empty_tz = Arc::from("");
        let result = PoSQLTimeZone::try_from(Some(empty_tz));
        assert!(result.is_err(), "Empty timezone string should fail");
    }

    #[test]
    fn test_unicode_timezone_strings() {
        let unicode_tz = Arc::from("Europe/Paris\u{00A0}"); // Non-breaking space character
        let result = PoSQLTimeZone::try_from(Some(unicode_tz));
        assert!(
            result.is_err(),
            "Unicode characters should not be valid in timezone strings"
        );
    }

    #[test]
    fn test_null_option() {
        let result = PoSQLTimeZone::try_from(None);
        assert!(result.is_ok(), "None should convert without error");
        assert_eq!(result.unwrap().0, Tz::UTC, "None should default to UTC");
    }

    #[test]
    fn test_utc_timezone() {
        let input = "2023-06-26T12:34:56Z";
        let expected_timezone = PoSQLTimeZone::UTC;
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_positive_offset_timezone() {
        let input = "2023-06-26T12:34:56+03:30";
        let expected_timezone = PoSQLTimeZone::from_offset(12600).unwrap(); // 3 hours and 30 minutes in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_negative_offset_timezone() {
        let input = "2023-06-26T12:34:56-04:00";
        let expected_timezone = PoSQLTimeZone::from_offset(-14400).unwrap(); // -4 hours in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_zero_offset_timezone() {
        let input = "2023-06-26T12:34:56+00:00";
        let expected_timezone = PoSQLTimeZone::UTC; // Zero offset defaults to UTC
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_unix_epoch_time_timezone() {
        let unix_time = 1_593_000_000; // Unix time as string
        let expected_timezone = PoSQLTimeZone::UTC; // Unix time should always be UTC
        let result = PoSQLTimestamp::to_timestamp(unix_time).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_unix_epoch_timestamp_parsing() {
        let unix_time = 1_593_000_000; // Example Unix timestamp (seconds since epoch)
        let expected_datetime = Utc.timestamp_opt(unix_time, 0).unwrap();
        let expected_unit = PoSQLTimeUnit::Second; // Assuming basic second precision for Unix timestamp
        let input = unix_time; // Simulate input as string since Unix times are often transmitted as strings
        let result = PoSQLTimestamp::to_timestamp(input).unwrap();

        assert_eq!(result.timestamp, expected_datetime);
        assert_eq!(result.timeunit, expected_unit);
    }

    #[test]
    fn test_basic_rfc3339_timestamp() {
        let input = "2023-06-26T12:34:56Z";
        let expected = Utc.with_ymd_and_hms(2023, 6, 26, 12, 34, 56).unwrap();
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_rfc3339_timestamp_with_positive_offset() {
        let input = "2023-06-26T08:00:00+04:30";
        let expected = Utc.with_ymd_and_hms(2023, 6, 26, 3, 30, 0).unwrap(); // Adjusted to UTC
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_rfc3339_timestamp_with_negative_offset() {
        let input = "2023-06-26T20:00:00-05:00";
        let expected = Utc.with_ymd_and_hms(2023, 6, 27, 1, 0, 0).unwrap(); // Adjusted to UTC
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_rfc3339_timestamp_with_utc_designator() {
        let input = "2023-06-26T12:34:56Z";
        let expected = Utc.with_ymd_and_hms(2023, 6, 26, 12, 34, 56).unwrap();
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_invalid_rfc3339_timestamp() {
        let input = "not-a-timestamp";
        assert!(PoSQLTimestamp::try_from(input).is_err());
    }

    #[test]
    fn test_timestamp_with_seconds() {
        let input = "2023-06-26T12:34:56Z";
        let expected_time = Utc.with_ymd_and_hms(2023, 6, 26, 12, 34, 56).unwrap();
        let expected_unit = PoSQLTimeUnit::Second;
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected_time);
        assert_eq!(result.timeunit, expected_unit);
    }

    #[test]
    #[allow(deprecated)]
    fn test_rfc3339_timestamp_with_milliseconds() {
        let input = "2023-06-26T12:34:56.123Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_milli(12, 34, 56, 123);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    #[allow(deprecated)]
    fn test_rfc3339_timestamp_with_microseconds() {
        let input = "2023-06-26T12:34:56.123456Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_micro(12, 34, 56, 123456);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }
    #[test]
    #[allow(deprecated)]
    fn test_rfc3339_timestamp_with_nanoseconds() {
        let input = "2023-06-26T12:34:56.123456789Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_nano(12, 34, 56, 123456789);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_general_parsing_error() {
        // This test assumes that there's a catch-all parsing error case that isn't covered by the more specific errors.
        let malformed_input = "2009-01-03T::00Z"; // Intentionally malformed timestamp
        let result = PoSQLTimestamp::try_from(malformed_input);
        assert!(matches!(result, Err(PoSQLTimestampError::ParsingError(_))));
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
