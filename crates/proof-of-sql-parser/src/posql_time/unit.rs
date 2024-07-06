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

    use crate::posql_time::{timestamp::PoSQLTimestamp, unit::PoSQLTimeUnit};
    use chrono::{TimeZone, Utc};

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
