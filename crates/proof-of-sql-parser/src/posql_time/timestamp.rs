use super::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestampError};
use alloc::{format, string::ToString};
use chrono::{offset::LocalResult, DateTime, TimeZone, Utc};
use core::hash::Hash;
use serde::{Deserialize, Serialize};

/// Represents a fully parsed timestamp with detailed time unit and timezone information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PoSQLTimestamp {
    /// The datetime representation in UTC.
    timestamp: DateTime<Utc>,

    /// The precision of the datetime value, e.g., seconds, milliseconds.
    timeunit: PoSQLTimeUnit,

    /// The timezone of the datetime, either UTC or a fixed offset from UTC.
    timezone: PoSQLTimeZone,
}

impl PoSQLTimestamp {
    /// Returns the combined date and time with time zone.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Returns the [`PoSQLTimeUnit`] for this timestamp
    pub fn timeunit(&self) -> PoSQLTimeUnit {
        self.timeunit
    }

    /// Returns the [`PoSQLTimeZone`] for this timestamp
    pub fn timezone(&self) -> PoSQLTimeZone {
        self.timezone
    }

    /// Attempts to parse a timestamp string into an [`PoSQLTimestamp`] structure.
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
    /// use proof_of_sql_parser::posql_time::{PoSQLTimestamp, PoSQLTimeZone};
    ///
    /// // Parsing an RFC 3339 timestamp without a timezone:
    /// let timestamp_str = "2009-01-03T18:15:05Z";
    /// let intermediate_timestamp = PoSQLTimestamp::try_from(timestamp_str).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone(), PoSQLTimeZone::Utc);
    ///
    /// // Parsing an RFC 3339 timestamp with a positive timezone offset:
    /// let timestamp_str_with_tz = "2009-01-03T18:15:05+03:00";
    /// let intermediate_timestamp = PoSQLTimestamp::try_from(timestamp_str_with_tz).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone(), PoSQLTimeZone::FixedOffset(10800)); // 3 hours in seconds
    /// ```
    pub fn try_from(timestamp_str: &str) -> Result<Self, PoSQLTimestampError> {
        let dt = DateTime::parse_from_rfc3339(timestamp_str).map_err(|e| {
            PoSQLTimestampError::ParsingError {
                error: e.to_string(),
            }
        })?;

        let offset_seconds = dt.offset().local_minus_utc();
        let timezone = PoSQLTimeZone::from_offset(offset_seconds);
        let nanoseconds = dt.timestamp_subsec_nanos();
        let timeunit = if nanoseconds % 1_000 != 0 {
            PoSQLTimeUnit::Nanosecond
        } else if nanoseconds % 1_000_000 != 0 {
            PoSQLTimeUnit::Microsecond
        } else if nanoseconds % 1_000_000_000 != 0 {
            PoSQLTimeUnit::Millisecond
        } else {
            PoSQLTimeUnit::Second
        };

        Ok(PoSQLTimestamp {
            timestamp: dt.with_timezone(&Utc),
            timeunit,
            timezone,
        })
    }

    /// Attempts to parse a timestamp string into an `PoSQLTimestamp` structure.
    /// This function supports two primary formats:
    ///
    /// **Unix Epoch Time Parsing**:
    ///    - Since Unix epoch timestamps don't inherently carry timezone information,
    ///      any Unix time parsed directly from an integer is assumed to be in UTC.
    ///
    /// # Examples
    /// ```
    /// use chrono::{DateTime, Utc};
    /// use proof_of_sql_parser::posql_time::{PoSQLTimestamp, PoSQLTimeZone};
    ///
    /// // Parsing a Unix epoch timestamp (assumed to be seconds and UTC):
    /// let unix_time = 1231006505;
    /// let intermediate_timestamp = PoSQLTimestamp::to_timestamp(unix_time).unwrap();
    /// assert_eq!(intermediate_timestamp.timezone(), PoSQLTimeZone::Utc);
    /// ```
    pub fn to_timestamp(epoch: i64) -> Result<Self, PoSQLTimestampError> {
        match Utc.timestamp_opt(epoch, 0) {
            LocalResult::Single(timestamp) => Ok(PoSQLTimestamp {
                timestamp,
                timeunit: PoSQLTimeUnit::Second,
                timezone: PoSQLTimeZone::Utc,
            }),
            LocalResult::Ambiguous(earliest, latest) => Err(PoSQLTimestampError::Ambiguous{ error:
                format!("The local time is ambiguous because there is a fold in the local time: earliest: {earliest} latest: {latest} "),
        }),
            LocalResult::None => Err(PoSQLTimestampError::LocalTimeDoesNotExist),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_epoch_time_timezone() {
        let unix_time = 1231006505; // Unix time as string
        let expected_timezone = PoSQLTimeZone::Utc; // Unix time should always be UTC
        let result = PoSQLTimestamp::to_timestamp(unix_time).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_unix_epoch_timestamp_parsing() {
        let unix_time = 1231006505; // Example Unix timestamp (seconds since epoch)
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
        assert_eq!(
            PoSQLTimestamp::try_from(input),
            Err(PoSQLTimestampError::ParsingError {
                error: "input contains invalid characters".into()
            })
        );
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
    fn test_general_parsing_error() {
        // This test assumes that there's a catch-all parsing error case that isn't covered by the more specific errors.
        let malformed_input = "2009-01-03T::00Z"; // Intentionally malformed timestamp
        let result = PoSQLTimestamp::try_from(malformed_input);
        assert!(matches!(
            result,
            Err(PoSQLTimestampError::ParsingError { .. })
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
        assert!(PoSQLTimestamp::try_from(input).is_ok());
    }

    #[test]
    fn test_leap_seconds_ranges() {
        // Timestamp just before the leap second
        let before_leap_second = "1998-12-31T23:59:59Z";
        // Timestamp during the leap second
        let leap_second = "1998-12-31T23:59:60Z";
        // Timestamp just after the leap second
        let after_leap_second = "1999-01-01T00:00:00Z";

        // Parse timestamps
        let before_leap_dt = PoSQLTimestamp::try_from(before_leap_second).unwrap();
        let leap_second_dt = PoSQLTimestamp::try_from(leap_second).unwrap();
        let after_leap_dt = PoSQLTimestamp::try_from(after_leap_second).unwrap();

        // Ensure that "23:59:60Z" - 1 second is considered equivalent to "23:59:59Z"
        assert_eq!(
            before_leap_dt.timestamp,
            leap_second_dt.timestamp - chrono::Duration::seconds(1)
        );

        // Ensure that "23:59:60Z" + 1 second is "1999-01-01T00:00:00Z"
        assert_eq!(
            after_leap_dt.timestamp,
            leap_second_dt.timestamp + chrono::Duration::seconds(1)
        );
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
