use chrono::{DateTime, NaiveDateTime, Offset, TimeZone, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum IntermediateTimestampError {
    #[error("Invalid timeunit")]
    /// Error converting intermediate time units to PoSQL time units
    InvalidTimeUnit,

    #[error("Invalid timezone")]
    /// Error converting intermediate time zones to PoSQL timezones
    InvalidTimeZone,
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
    /// Default variant for UTC timezoen
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

/// Error encountered during timestamp parsing
#[derive(Debug, Error)]
pub enum TimeParseError {
    /// Could not parse a timestamp from string
    #[error("Invalid timestamp format")]
    InvalidFormat,
}

/// Intermediate Time
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntermediateTimestamp {
    /// Count of time units since the unix epoch
    pub timestamp: i64,
    /// Seconds, milliseconds, microseconds, or nanoseconds
    pub unit: IntermediateTimeUnit,
    /// Timezone captured from parsed string
    pub timezone: IntermediateTimeZone,
}

impl TryFrom<&str> for IntermediateTimestamp {
    type Error = TimeParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_intermediate_timestamp(value).map_err(|_| TimeParseError::InvalidFormat)
    }
}

/// Parses a timestamp from valid strings obtained from the lexer
pub fn parse_intermediate_timestamp(ts: &str) -> Result<IntermediateTimestamp, &'static str> {
    let format_with_tz = "%Y-%m-%d %H:%M:%S%.f%:z";
    let format_without_tz = "%Y-%m-%d %H:%M:%S%.f";

    // Helper function to determine the precision of the fractional seconds
    fn determine_precision(fraction: &str) -> IntermediateTimeUnit {
        match fraction.len() {
            0 => IntermediateTimeUnit::Second,
            1..=3 => IntermediateTimeUnit::Millisecond,
            4..=6 => IntermediateTimeUnit::Microsecond,
            _ => IntermediateTimeUnit::Nanosecond,
        }
    }

    // Extract the fractional part correctly
    fn extract_fraction(ts: &str) -> &str {
        if let Some((_, fractional)) = ts.split_once('.') {
            if let Some((fractional, _)) = fractional.split_once(|c| c == '+' || c == '-') {
                return fractional;
            }
            return fractional;
        }
        ""
    }

    // First try parsing with timezone
    if let Ok(dt) = DateTime::parse_from_str(ts, format_with_tz) {
        if let Some(timestamp_nanos) = dt.timestamp_nanos_opt() {
            let offset_seconds = dt.offset().fix().local_minus_utc();
            let fraction = extract_fraction(ts);
            let unit = determine_precision(fraction);
            return Ok(IntermediateTimestamp {
                timestamp: timestamp_nanos,
                unit,
                timezone: IntermediateTimeZone::from_offset(offset_seconds),
            });
        } else {
            return Err("Failed to convert datetime to nanoseconds");
        }
    }

    // If that fails, try parsing without timezone and assume UTC
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(ts, format_without_tz) {
        let datetime_utc = Utc.from_utc_datetime(&naive_dt);
        if let Some(timestamp_nanos) = datetime_utc.timestamp_nanos_opt() {
            let fraction = extract_fraction(ts);
            let unit = determine_precision(fraction);
            return Ok(IntermediateTimestamp {
                timestamp: timestamp_nanos,
                unit,
                timezone: IntermediateTimeZone::Utc,
            });
        } else {
            return Err("Failed to convert datetime to nanoseconds");
        }
    }

    Err("Invalid timestamp format")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone, Timelike, Utc};

    #[test]
    fn test_display_intermediate_timezone() {
        // Test Utc
        let tz_utc = IntermediateTimeZone::Utc;
        assert_eq!(format!("{}", tz_utc), "Z");

        // Test positive offsets
        let tz_offset_1 = IntermediateTimeZone::FixedOffset(3600); // +01:00
        assert_eq!(format!("{}", tz_offset_1), "+01:00");

        let tz_offset_2 = IntermediateTimeZone::FixedOffset(19800); // +05:30
        assert_eq!(format!("{}", tz_offset_2), "+05:30");

        let tz_offset_3 = IntermediateTimeZone::FixedOffset(3600 * 12); // +12:00
        assert_eq!(format!("{}", tz_offset_3), "+12:00");

        // Test negative offsets
        let tz_offset_4 = IntermediateTimeZone::FixedOffset(-3600); // -01:00
        assert_eq!(format!("{}", tz_offset_4), "-01:00");

        let tz_offset_5 = IntermediateTimeZone::FixedOffset(-12600); // -03:30
        assert_eq!(format!("{}", tz_offset_5), "-03:30");

        let tz_offset_6 = IntermediateTimeZone::FixedOffset(-3600 * 12); // -12:00
        assert_eq!(format!("{}", tz_offset_6), "-12:00");

        // Test edge cases
        let tz_offset_7 = IntermediateTimeZone::FixedOffset(0); // +00:00
        assert_eq!(format!("{}", tz_offset_7), "Z");

        let tz_offset_8 = IntermediateTimeZone::FixedOffset(3600 * 14); // +14:00
        assert_eq!(format!("{}", tz_offset_8), "+14:00");

        let tz_offset_9 = IntermediateTimeZone::FixedOffset(-3600 * 14); // -14:00
        assert_eq!(format!("{}", tz_offset_9), "-14:00");
    }

    #[test]
    fn test_parse_with_timezone() {
        let ts_with_tz = "2024-06-20 12:34:56+02:00";
        let result = parse_intermediate_timestamp(ts_with_tz)
            .expect("Failed to parse timestamp with timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Second);

        let ts_with_tz = "2024-06-20 12:34:56.123+02:00";
        let result = parse_intermediate_timestamp(ts_with_tz)
            .expect("Failed to parse timestamp with timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Millisecond);

        let ts_with_tz = "2024-06-20 12:34:56.123456+02:00";
        let result = parse_intermediate_timestamp(ts_with_tz)
            .expect("Failed to parse timestamp with timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Microsecond);

        let ts_with_tz = "2024-06-20 12:34:56.123456789+02:00";
        let result = parse_intermediate_timestamp(ts_with_tz)
            .expect("Failed to parse timestamp with timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Nanosecond);
        assert_eq!(result.timezone, IntermediateTimeZone::FixedOffset(7200)); // +02:00 is 7200 seconds
        let expected_timestamp: DateTime<FixedOffset> = FixedOffset::east_opt(7200)
            .unwrap()
            .with_ymd_and_hms(2024, 6, 20, 12, 34, 56)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();
        assert_eq!(
            result.timestamp,
            expected_timestamp.timestamp_nanos_opt().unwrap()
        );
    }

    #[test]
    fn test_parse_without_timezone() {
        let ts_without_tz = "2024-06-20 12:34:56";
        let result = parse_intermediate_timestamp(ts_without_tz)
            .expect("Failed to parse timestamp without timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Second);
        assert_eq!(result.timezone, IntermediateTimeZone::Utc);

        let ts_without_tz = "2024-06-20 12:34:56.123";
        let result = parse_intermediate_timestamp(ts_without_tz)
            .expect("Failed to parse timestamp without timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Millisecond);
        assert_eq!(result.timezone, IntermediateTimeZone::Utc);

        let ts_without_tz = "2024-06-20 12:34:56.123456";
        let result = parse_intermediate_timestamp(ts_without_tz)
            .expect("Failed to parse timestamp without timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Microsecond);
        assert_eq!(result.timezone, IntermediateTimeZone::Utc);

        let ts_without_tz = "2024-06-20 12:34:56.123456789";
        let result = parse_intermediate_timestamp(ts_without_tz)
            .expect("Failed to parse timestamp without timezone");

        assert_eq!(result.unit, IntermediateTimeUnit::Nanosecond);
        assert_eq!(result.timezone, IntermediateTimeZone::Utc);
        let expected_timestamp = Utc
            .with_ymd_and_hms(2024, 6, 20, 12, 34, 56)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();
        assert_eq!(
            result.timestamp,
            expected_timestamp.timestamp_nanos_opt().unwrap()
        );
    }

    #[test]
    fn test_parse_invalid_format() {
        let invalid_ts = "invalid timestamp";
        let result = parse_intermediate_timestamp(invalid_ts);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid timestamp format");
    }

    #[test]
    fn test_parse_missing_fractional_seconds() {
        let ts_missing_fractional = "2024-06-20 12:34:56+02:00";
        let result = parse_intermediate_timestamp(ts_missing_fractional)
            .expect("Failed to parse timestamp without fractional seconds");

        assert_eq!(result.unit, IntermediateTimeUnit::Second);
        assert_eq!(result.timezone, IntermediateTimeZone::FixedOffset(7200));
        let expected_timestamp: DateTime<FixedOffset> = FixedOffset::east_opt(7200)
            .unwrap()
            .with_ymd_and_hms(2024, 6, 20, 12, 34, 56)
            .unwrap();
        assert_eq!(
            result.timestamp,
            expected_timestamp.timestamp_nanos_opt().unwrap()
        );
    }

    #[test]
    fn test_parse_different_timezones() {
        let timezones = [
            ("2024-06-20 12:34:56.123456789-05:00", -18000), // -05:00 is -18000 seconds
            ("2024-06-20 12:34:56.123456789+00:00", 0),      // +00:00 is 0 seconds
            ("2024-06-20 12:34:56.123456789+05:30", 19800),  // +05:30 is 19800 seconds
            ("2024-06-20 12:34:56.123456789-08:00", -28800), // -08:00 is -28800 seconds
            ("2024-06-20 12:34:56.123456789+09:00", 32400),  // +09:00 is 32400 seconds
            ("2024-06-20 12:34:56.123456789-03:30", -12600), // -03:30 is -12600 seconds
            ("2024-06-20 12:34:56.123456789+12:00", 43200),  // +12:00 is 43200 seconds
            ("2024-06-20 12:34:56.123456789-12:00", -43200), // -12:00 is -43200 seconds
        ];

        for (ts, offset_seconds) in &timezones {
            let result = parse_intermediate_timestamp(ts)
                .unwrap_or_else(|_| panic!("Failed to parse timestamp with timezone {}", ts));

            assert_eq!(result.unit, IntermediateTimeUnit::Nanosecond);
            assert_eq!(
                result.timezone,
                IntermediateTimeZone::from_offset(*offset_seconds)
            );
            let expected_timestamp: DateTime<FixedOffset> = FixedOffset::east_opt(*offset_seconds)
                .unwrap()
                .with_ymd_and_hms(2024, 6, 20, 12, 34, 56)
                .unwrap()
                .with_nanosecond(123_456_789)
                .unwrap();
            assert_eq!(
                result.timestamp,
                expected_timestamp.timestamp_nanos_opt().unwrap()
            );
        }
    }
}
