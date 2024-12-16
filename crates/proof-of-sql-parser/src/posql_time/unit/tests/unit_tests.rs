use crate::posql_time::{PoSQLTimestamp, PoSQLTimestampError};
use crate::posql_time::unit::PoSQLTimeUnit;
use chrono::{TimeZone, Utc};

// allow(deprecated) for the sole purpose of testing that
// timestamp precision is parsed correctly.
#[allow(deprecated, clippy::missing_panics_doc)]
#[cfg(test)]
mod tests {
    use super::*;

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
        for &value in &invalid_precisions {
            let result = PoSQLTimeUnit::try_from(value);
            assert!(matches!(
                result,
                Err(PoSQLTimestampError::UnsupportedPrecision { .. })
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
        let expected = Utc.ymd(2023, 6, 26).and_hms_micro(12, 34, 56, 123_456);
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
        let expected = Utc.ymd(2023, 6, 26).and_hms_nano(12, 34, 56, 123_456_789);
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timeunit(), PoSQLTimeUnit::Nanosecond);
        assert_eq!(
            result.timestamp().timestamp_nanos_opt().unwrap(),
            expected.timestamp_nanos_opt().unwrap()
        );
    }
}