use crate::error::PoSQLTimestampError;
use arrow::datatypes::TimeUnit as ArrowTimeUnit;
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
    use crate::{error::PoSQLTimestampError, posql_time::timestamp::PoSQLTimestamp};
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_valid_precisions() {
        assert_eq!(PoSQLTimeUnit::try_from("0"), Ok(PoSQLTimeUnit::Second));
        assert_eq!(PoSQLTimeUnit::try_from("3"), Ok(PoSQLTimeUnit::Millisecond));
        assert_eq!(PoSQLTimeUnit::try_from("6"), Ok(PoSQLTimeUnit::Microsecond));
        assert_eq!(PoSQLTimeUnit::try_from("9"), Ok(PoSQLTimeUnit::Nanosecond));
    }

    #[test]
    fn test_invalid_precisions() {
        // Test some random incorrect values
        assert!(
            matches!(PoSQLTimeUnit::try_from("1"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "1")
        );
        assert!(
            matches!(PoSQLTimeUnit::try_from("2"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "2")
        );
        assert!(
            matches!(PoSQLTimeUnit::try_from("4"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "4")
        );
        assert!(
            matches!(PoSQLTimeUnit::try_from("5"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "5")
        );
        assert!(
            matches!(PoSQLTimeUnit::try_from("7"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "7")
        );
        assert!(
            matches!(PoSQLTimeUnit::try_from("8"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "8")
        );
        assert!(
            matches!(PoSQLTimeUnit::try_from("10"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "10")
        );
        // Non-numeric strings
        assert!(
            matches!(PoSQLTimeUnit::try_from("abc"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "abc")
        );
        // Empty string
        assert!(
            matches!(PoSQLTimeUnit::try_from(""), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x.is_empty())
        );
    }

    #[test]
    fn test_edge_cases_with_non_numeric_inputs() {
        assert!(
            matches!(PoSQLTimeUnit::try_from("zero"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "zero")
        );
        assert!(
            matches!(PoSQLTimeUnit::try_from("three"), Err(PoSQLTimestampError::UnsupportedPrecision(x)) if x == "three")
        );
    }

    #[test]
    fn test_invalid_precision() {
        let invalid_precisions = ["1", "2", "4", "5", "7", "8", "10"]; // Testing various invalid inputs
        for &value in invalid_precisions.iter() {
            let result = PoSQLTimeUnit::try_from(value);
            assert!(
                matches!(result, Err(PoSQLTimestampError::UnsupportedPrecision(err)) if err == *value),
                "Failed with value: {}",
                value
            );
        }
    }

    #[test]
    fn test_rfc3339_timestamp_with_milliseconds() {
        let input = "2023-06-26T12:34:56.123Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_milli(12, 34, 56, 123);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit, PoSQLTimeUnit::Millisecond);
        assert_eq!(result.timestamp, expected);
    }

    #[test]
    fn test_rfc3339_timestamp_with_microseconds() {
        let input = "2023-06-26T12:34:56.123456Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_micro(12, 34, 56, 123456);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit, PoSQLTimeUnit::Microsecond);
        assert_eq!(result.timestamp, expected);
    }
    #[test]
    fn test_rfc3339_timestamp_with_nanoseconds() {
        let input = "2023-06-26T12:34:56.123456789Z";
        let expected = Utc.ymd(2023, 6, 26).and_hms_nano(12, 34, 56, 123456789);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit, PoSQLTimeUnit::Nanosecond);
        assert_eq!(result.timestamp, expected);
    }
}
