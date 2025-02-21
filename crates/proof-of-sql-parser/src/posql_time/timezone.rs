use super::PoSQLTimestampError;
use alloc::{string::ToString, sync::Arc};
use core::fmt;
use serde::{Deserialize, Serialize};

/// Captures a timezone from a timestamp query
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct PoSQLTimeZone {
    offset: i32,
}

impl PoSQLTimeZone {
    /// Create a timezone from a count of seconds
    #[must_use]
    pub const fn new(offset: i32) -> Self {
        PoSQLTimeZone { offset }
    }
    #[must_use]
    /// The UTC timezone
    pub const fn utc() -> Self {
        PoSQLTimeZone::new(0)
    }
    /// Get the underlying offset in seconds
    #[must_use]
    pub const fn offset(self) -> i32 {
        self.offset
    }
}

impl TryFrom<&Option<Arc<str>>> for PoSQLTimeZone {
    type Error = PoSQLTimestampError;

    fn try_from(value: &Option<Arc<str>>) -> Result<Self, Self::Error> {
        match value {
            Some(tz_str) => {
                let tz = Arc::as_ref(tz_str).to_uppercase();
                match tz.as_str() {
                    "Z" | "UTC" | "00:00" | "+00:00" | "0:00" | "+0:00" => Ok(PoSQLTimeZone::utc()),
                    tz if tz.chars().count() == 6
                        && (tz.starts_with('+') || tz.starts_with('-')) =>
                    {
                        let sign = if tz.starts_with('-') { -1 } else { 1 };
                        let hours = tz[1..3]
                            .parse::<i32>()
                            .map_err(|_| PoSQLTimestampError::InvalidTimezoneOffset)?;
                        let minutes = tz[4..6]
                            .parse::<i32>()
                            .map_err(|_| PoSQLTimestampError::InvalidTimezoneOffset)?;
                        let total_seconds = sign * ((hours * 3600) + (minutes * 60));
                        Ok(PoSQLTimeZone::new(total_seconds))
                    }
                    _ => Err(PoSQLTimestampError::InvalidTimezone {
                        timezone: tz.to_string(),
                    }),
                }
            }
            None => Ok(PoSQLTimeZone::utc()),
        }
    }
}

impl fmt::Display for PoSQLTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.offset();
        let hours = seconds / 3600;
        let minutes = (seconds.abs() % 3600) / 60;
        if seconds < 0 {
            write!(f, "-{:02}:{:02}", hours.abs(), minutes)
        } else {
            write!(f, "+{hours:02}:{minutes:02}")
        }
    }
}

#[cfg(test)]
mod timezone_parsing_tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_display_fixed_offset_positive() {
        let timezone = PoSQLTimeZone::new(4500); // +01:15
        assert_eq!(format!("{timezone}"), "+01:15");
    }

    #[test]
    fn test_display_fixed_offset_negative() {
        let timezone = PoSQLTimeZone::new(-3780); // -01:03
        assert_eq!(format!("{timezone}"), "-01:03");
    }

    #[test]
    fn test_display_utc() {
        let timezone = PoSQLTimeZone::utc();
        assert_eq!(format!("{timezone}"), "+00:00");
    }

    #[test]
    fn test_try_from_timezone_string() {
        // Test UTC variants
        let utc_variants = [
            Some(Arc::from("Z")),
            Some(Arc::from("UTC")),
            Some(Arc::from("00:00")),
            Some(Arc::from("+00:00")),
            Some(Arc::from("0:00")),
            Some(Arc::from("+0:00")),
            None,
        ];
        for variant in utc_variants {
            let tz = PoSQLTimeZone::try_from(&variant).unwrap();
            assert_eq!(tz, PoSQLTimeZone::utc());
        }

        // Test valid offsets
        let valid_offsets = [
            ("+01:30", 5400),   // 1.5 hours
            ("-02:45", -9900),  // -2.75 hours
            ("+14:00", 50400),  // Max offset
            ("-12:00", -43200), // Min offset
            ("+00:30", 1800),   // 30 minutes positive
            ("-00:30", -1800),  // 30 minutes negative
            ("+23:59", 86340),  // Near max
            ("-23:59", -86340), // Near min
        ];
        for (offset_str, seconds) in valid_offsets {
            let tz = PoSQLTimeZone::try_from(&Some(Arc::from(offset_str))).unwrap();
            assert_eq!(tz.offset(), seconds);
        }

        // Test invalid formats
        let invalid_formats = [
            "ABC",
            "1:00",
            "25:00",
            "12:60",
            "+1:00",
            "-1:00",
            "+12:0",
            "-12:0",
            "UTC+01:00", // No compound formats
            "GMT",       // No named timezones
            "+:",        // Missing numbers
            ":",         // Just separator
            "",          // Empty string
            "+24:00",    // Invalid hour
            "-24:00",    // Invalid hour
            "+00:60",    // Invalid minute
            "-00:60",    // Invalid minute
        ];
        for invalid in invalid_formats {
            let result = PoSQLTimeZone::try_from(&Some(Arc::from(invalid)));
            assert!(result.is_err(), "Should fail for input: {invalid}");
            assert!(matches!(
                result,
                Err(PoSQLTimestampError::InvalidTimezone { .. })
            ));
        }
    }

    #[test]
    fn test_invalid_timezone_offset_parsing() {
        // Test invalid hour/minute values
        let invalid_offsets = [
            "+aa:00", // Invalid hour format
            "+00:xx", // Invalid minute format
            "+24:00", // Hour too large
            "+00:60", // Minute too large
            "-24:00", // Hour too large negative
            "-00:60", // Minute too large negative
            "+0a:00", // Partial invalid hour
            "+00:0x", // Partial invalid minute
        ];
        for invalid in invalid_offsets {
            let result = PoSQLTimeZone::try_from(&Some(Arc::from(invalid)));
            assert!(matches!(
                result,
                Err(PoSQLTimestampError::InvalidTimezoneOffset)
            ));
        }
    }

    #[test]
    fn test_timezone_offset_values() {
        // Test edge cases for offset values
        let test_cases = [
            (0, "+00:00"),      // UTC
            (3600, "+01:00"),   // +1 hour
            (-3600, "-01:00"),  // -1 hour
            (5400, "+01:30"),   // +1.5 hours
            (-5400, "-01:30"),  // -1.5 hours
            (43200, "+12:00"),  // +12 hours
            (-43200, "-12:00"), // -12 hours
            (50400, "+14:00"),  // +14 hours (max)
            (-50400, "-14:00"), // -14 hours (min)
            (1800, "+00:30"),   // +30 minutes
            (-1800, "-00:30"),  // -30 minutes
            (86340, "+23:59"),  // +23:59
            (-86340, "-23:59"), // -23:59
        ];

        for (offset, expected) in test_cases {
            let tz = PoSQLTimeZone::new(offset);
            assert_eq!(format!("{tz}"), expected);
            assert_eq!(tz.offset(), offset);
        }
    }

    #[test]
    fn test_timezone_equality() {
        let tz1 = PoSQLTimeZone::new(3600);
        let tz2 = PoSQLTimeZone::new(3600);
        let tz3 = PoSQLTimeZone::new(-3600);
        let tz4 = PoSQLTimeZone::utc();
        let tz5 = PoSQLTimeZone::new(0);

        assert_eq!(tz1, tz2);
        assert_ne!(tz1, tz3);
        assert_eq!(tz4, tz5); // UTC equals offset 0
        assert_ne!(tz1, tz4); // Different offsets not equal
    }
}

#[cfg(test)]
mod timezone_offset_tests {
    use crate::posql_time::{timestamp::PoSQLTimestamp, timezone};

    #[test]
    fn test_utc_timezone() {
        let input = "2023-06-26T12:34:56Z";
        let expected_timezone = timezone::PoSQLTimeZone::utc();
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }

    #[test]
    fn test_positive_offset_timezone() {
        let input = "2023-06-26T12:34:56+03:30";
        let expected_timezone = timezone::PoSQLTimeZone::new(12600); // 3 hours and 30 minutes in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }

    #[test]
    fn test_negative_offset_timezone() {
        let input = "2023-06-26T12:34:56-04:00";
        let expected_timezone = timezone::PoSQLTimeZone::new(-14400); // -4 hours in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }

    #[test]
    fn test_zero_offset_timezone() {
        let input = "2023-06-26T12:34:56+00:00";
        let expected_timezone = timezone::PoSQLTimeZone::utc(); // Zero offset defaults to UTC
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }
}
