use super::PoSQLTimestampError;
use core::fmt;
use serde::{Deserialize, Serialize};

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

impl PoSQLTimeUnit {
    /// Convert the numerical unit of one timeunit to another for comparison purposes.
    pub fn normalize_timeunit(
        timestamp: i64,
        from_unit: &PoSQLTimeUnit,
        to_unit: &PoSQLTimeUnit,
    ) -> i64 {
        match (from_unit, to_unit) {
            // Conversions from Seconds
            (PoSQLTimeUnit::Second, PoSQLTimeUnit::Millisecond) => timestamp * 1000,
            (PoSQLTimeUnit::Second, PoSQLTimeUnit::Microsecond) => timestamp * 1_000_000,
            (PoSQLTimeUnit::Second, PoSQLTimeUnit::Nanosecond) => timestamp * 1_000_000_000,

            // Conversions from Milliseconds
            (PoSQLTimeUnit::Millisecond, PoSQLTimeUnit::Second) => timestamp / 1000,
            (PoSQLTimeUnit::Millisecond, PoSQLTimeUnit::Microsecond) => timestamp * 1000,
            (PoSQLTimeUnit::Millisecond, PoSQLTimeUnit::Nanosecond) => timestamp * 1_000_000,

            // Conversions from Microseconds
            (PoSQLTimeUnit::Microsecond, PoSQLTimeUnit::Second) => timestamp / 1_000_000,
            (PoSQLTimeUnit::Microsecond, PoSQLTimeUnit::Millisecond) => timestamp / 1000,
            (PoSQLTimeUnit::Microsecond, PoSQLTimeUnit::Nanosecond) => timestamp * 1000,

            // Conversions from Nanoseconds
            (PoSQLTimeUnit::Nanosecond, PoSQLTimeUnit::Second) => timestamp / 1_000_000_000,
            (PoSQLTimeUnit::Nanosecond, PoSQLTimeUnit::Millisecond) => timestamp / 1_000_000,
            (PoSQLTimeUnit::Nanosecond, PoSQLTimeUnit::Microsecond) => timestamp / 1000,

            // If units are the same, no adjustment is needed
            _ => timestamp,
        }
    }
}

impl TryFrom<&str> for PoSQLTimeUnit {
    type Error = PoSQLTimestampError;
    fn try_from(value: &str) -> Result<Self, PoSQLTimestampError> {
        match value {
            "0" => Ok(PoSQLTimeUnit::Second),
            "3" => Ok(PoSQLTimeUnit::Millisecond),
            "6" => Ok(PoSQLTimeUnit::Microsecond),
            "9" => Ok(PoSQLTimeUnit::Nanosecond),
            _ => Err(PoSQLTimestampError::UnsupportedPrecision(value.into())),
        }
    }
}

impl fmt::Display for PoSQLTimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoSQLTimeUnit::Second => write!(f, "seconds (precision: 0)"),
            PoSQLTimeUnit::Millisecond => write!(f, "milliseconds (precision: 3)"),
            PoSQLTimeUnit::Microsecond => write!(f, "microseconds (precision: 6)"),
            PoSQLTimeUnit::Nanosecond => write!(f, "nanoseconds (precision: 9)"),
        }
    }
}

// allow(deprecated) for the sole purpose of testing that
// timestamp precision is parsed correctly.
#[cfg(test)]
#[allow(deprecated)]
mod time_unit_tests {
    use super::*;
    use crate::posql_time::{PoSQLTimestamp, PoSQLTimestampError};
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_valid_precisions() {
        assert_eq!(PoSQLTimeUnit::try_from("0"), Ok(PoSQLTimeUnit::Second));
        assert_eq!(PoSQLTimeUnit::try_from("3"), Ok(PoSQLTimeUnit::Millisecond));
        assert_eq!(PoSQLTimeUnit::try_from("6"), Ok(PoSQLTimeUnit::Microsecond));
        assert_eq!(PoSQLTimeUnit::try_from("9"), Ok(PoSQLTimeUnit::Nanosecond));
    }

    #[test]
    fn test_invalid_precision() {
        let invalid_precisions = [
            "1", "2", "4", "5", "7", "8", "10", "zero", "three", "cat", "-1", "-2",
        ]; // Testing all your various invalid inputs
        for &value in invalid_precisions.iter() {
            let result = PoSQLTimeUnit::try_from(value);
            assert!(matches!(
                result,
                Err(PoSQLTimestampError::UnsupportedPrecision(_))
            ));
        }
    }

    #[test]
    fn test_rfc3339_timestamp_with_milliseconds() {
        let input = "2023-06-26T12:34:56.123Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_milli(12, 34, 56, 123);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit(), PoSQLTimeUnit::Millisecond);
        assert_eq!(
            result.timestamp().timestamp_millis(),
            expected.timestamp_millis()
        );
    }

    #[test]
    fn test_rfc3339_timestamp_with_microseconds() {
        let input = "2023-06-26T12:34:56.123456Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_micro(12, 34, 56, 123456);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit(), PoSQLTimeUnit::Microsecond);
        assert_eq!(
            result.timestamp().timestamp_micros(),
            expected.timestamp_micros()
        );
    }
    #[test]
    fn test_rfc3339_timestamp_with_nanoseconds() {
        let input = "2023-06-26T12:34:56.123456789Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_nano(12, 34, 56, 123456789);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit(), PoSQLTimeUnit::Nanosecond);
        assert_eq!(
            result.timestamp().timestamp_nanos_opt().unwrap(),
            expected.timestamp_nanos_opt().unwrap()
        );
    }
    #[test]
    fn test_normalize_timeunit_seconds_to_milliseconds() {
        let timestamp = 1231006505; // seconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Second,
            &PoSQLTimeUnit::Millisecond,
        );
        assert_eq!(result, 1231006505000); // converted to milliseconds
    }

    #[test]
    fn test_normalize_timeunit_seconds_to_microseconds() {
        let timestamp = 1231006505; // seconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Second,
            &PoSQLTimeUnit::Microsecond,
        );
        assert_eq!(result, 1231006505000000); // converted to microseconds
    }

    #[test]
    fn test_normalize_timeunit_seconds_to_nanoseconds() {
        let timestamp = 1231006505; // seconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Second,
            &PoSQLTimeUnit::Nanosecond,
        );
        assert_eq!(result, 1231006505000000000); // converted to nanoseconds
    }

    #[test]
    fn test_normalize_timeunit_milliseconds_to_seconds() {
        let timestamp = 1231006505000; // milliseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Millisecond,
            &PoSQLTimeUnit::Second,
        );
        assert_eq!(result, 1231006505); // converted to seconds
    }

    #[test]
    fn test_normalize_timeunit_milliseconds_to_microseconds() {
        let timestamp = 1231006505; // milliseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Millisecond,
            &PoSQLTimeUnit::Microsecond,
        );
        assert_eq!(result, 1231006505000); // converted to microseconds
    }

    #[test]
    fn test_normalize_timeunit_milliseconds_to_nanoseconds() {
        let timestamp = 1231006505; // milliseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Millisecond,
            &PoSQLTimeUnit::Nanosecond,
        );
        assert_eq!(result, 1231006505000000); // converted to nanoseconds
    }

    #[test]
    fn test_normalize_timeunit_microseconds_to_seconds() {
        let timestamp = 1231006505000000; // microseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Microsecond,
            &PoSQLTimeUnit::Second,
        );
        assert_eq!(result, 1231006505); // converted to seconds
    }

    #[test]
    fn test_normalize_timeunit_microseconds_to_milliseconds() {
        let timestamp = 1231006505000; // microseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Microsecond,
            &PoSQLTimeUnit::Millisecond,
        );
        assert_eq!(result, 1231006505); // converted to milliseconds
    }

    #[test]
    fn test_normalize_timeunit_microseconds_to_nanoseconds() {
        let timestamp = 1231006505; // microseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Microsecond,
            &PoSQLTimeUnit::Nanosecond,
        );
        assert_eq!(result, 1231006505000); // converted to nanoseconds
    }

    #[test]
    fn test_normalize_timeunit_nanoseconds_to_seconds() {
        let timestamp = 1231006505000000000; // nanoseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Nanosecond,
            &PoSQLTimeUnit::Second,
        );
        assert_eq!(result, 1231006505); // converted to seconds
    }

    #[test]
    fn test_normalize_timeunit_nanoseconds_to_milliseconds() {
        let timestamp = 1231006505000000; // nanoseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Nanosecond,
            &PoSQLTimeUnit::Millisecond,
        );
        assert_eq!(result, 1231006505); // converted to milliseconds
    }

    #[test]
    fn test_normalize_timeunit_nanoseconds_to_microseconds() {
        let timestamp = 1231006505000; // nanoseconds
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Nanosecond,
            &PoSQLTimeUnit::Microsecond,
        );
        assert_eq!(result, 1231006505); // converted to microseconds
    }

    #[test]
    fn test_normalize_timeunit_same_units() {
        // If from_unit and to_unit are the same, it should return the timestamp as is
        let timestamp = 1231006505;
        let result = PoSQLTimeUnit::normalize_timeunit(
            timestamp,
            &PoSQLTimeUnit::Second,
            &PoSQLTimeUnit::Second,
        );
        assert_eq!(result, timestamp); // No conversion needed
    }
}
