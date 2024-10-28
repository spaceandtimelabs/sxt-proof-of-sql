use super::PoSQLTimestampError;
use alloc::{string::ToString, sync::Arc};
use core::fmt;
use serde::{Deserialize, Serialize};

/// Captures a timezone from a timestamp query
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoSQLTimeZone {
    /// Default variant for UTC timezone
    Utc,
    /// `TImezone` offset in seconds
    FixedOffset(i32),
}

impl PoSQLTimeZone {
    /// Parse a timezone from a count of seconds
    #[must_use]
    pub fn from_offset(offset: i32) -> Self {
        if offset == 0 {
            PoSQLTimeZone::Utc
        } else {
            PoSQLTimeZone::FixedOffset(offset)
        }
    }

    /// For comparisons, normalize a timestamp based on a timezone offset so
    /// it can be compared to another timestamp.
    pub fn normalize_to_utc(timestamp: i64, tz: &PoSQLTimeZone) -> i64 {
        match tz {
            PoSQLTimeZone::Utc => timestamp, // No adjustment needed
            PoSQLTimeZone::FixedOffset(offset) => timestamp - *offset as i64, // Adjust by offset in seconds
        }
    }
}

impl TryFrom<&Option<Arc<str>>> for PoSQLTimeZone {
    type Error = PoSQLTimestampError;

    fn try_from(value: &Option<Arc<str>>) -> Result<Self, Self::Error> {
        match value {
            Some(tz_str) => {
                let tz = Arc::as_ref(tz_str).to_uppercase();
                match tz.as_str() {
                    "Z" | "UTC" | "00:00" | "+00:00" | "0:00" | "+0:00" => Ok(PoSQLTimeZone::Utc),
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
                        Ok(PoSQLTimeZone::FixedOffset(total_seconds))
                    }
                    _ => Err(PoSQLTimestampError::InvalidTimezone {
                        timezone: tz.to_string(),
                    }),
                }
            }
            None => Ok(PoSQLTimeZone::Utc),
        }
    }
}

impl fmt::Display for PoSQLTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PoSQLTimeZone::Utc => {
                write!(f, "+00:00")
            }
            PoSQLTimeZone::FixedOffset(seconds) => {
                let hours = seconds / 3600;
                let minutes = (seconds.abs() % 3600) / 60;
                if seconds < 0 {
                    write!(f, "-{:02}:{:02}", hours.abs(), minutes)
                } else {
                    write!(f, "+{hours:02}:{minutes:02}")
                }
            }
        }
    }
}

#[cfg(test)]
mod timezone_parsing_tests {
    use crate::posql_time::{timezone, PoSQLTimeZone, PoSQLTimestamp};
    use alloc::format;

    #[test]
    fn test_display_fixed_offset_positive() {
        let timezone = timezone::PoSQLTimeZone::FixedOffset(4500); // +01:15
        assert_eq!(format!("{timezone}"), "+01:15");
    }

    #[test]
    fn test_display_fixed_offset_negative() {
        let timezone = timezone::PoSQLTimeZone::FixedOffset(-3780); // -01:03
        assert_eq!(format!("{timezone}"), "-01:03");
    }

    #[test]
    fn test_display_utc() {
        let timezone = timezone::PoSQLTimeZone::Utc;
        assert_eq!(format!("{timezone}"), "+00:00");
    }

    #[test]
    fn test_utc_timezone() {
        let input = "2023-06-26T12:34:56Z";
        let expected_timezone = timezone::PoSQLTimeZone::Utc;
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }

    #[test]
    fn test_positive_offset_timezone() {
        let input = "2023-06-26T12:34:56+03:30";
        let expected_timezone = timezone::PoSQLTimeZone::from_offset(12600); // 3 hours and 30 minutes in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }

    #[test]
    fn test_negative_offset_timezone() {
        let input = "2023-06-26T12:34:56-04:00";
        let expected_timezone = timezone::PoSQLTimeZone::from_offset(-14400); // -4 hours in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }

    #[test]
    fn test_zero_offset_timezone() {
        let input = "2023-06-26T12:34:56+00:00";
        let expected_timezone = timezone::PoSQLTimeZone::Utc; // Zero offset defaults to UTC
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone(), expected_timezone);
    }

    #[test]
    fn test_from_offset() {
        // UTC case
        let tz = PoSQLTimeZone::from_offset(0);
        assert!(matches!(tz, PoSQLTimeZone::Utc));

        // Fixed offset case
        let tz = PoSQLTimeZone::from_offset(3600); // UTC+1
        assert!(matches!(tz, PoSQLTimeZone::FixedOffset(3600)));

        // Negative offset case (UTC-5)
        let tz = PoSQLTimeZone::from_offset(-18000);
        assert!(matches!(tz, PoSQLTimeZone::FixedOffset(-18000)));
    }

    #[test]
    fn test_normalize_to_utc_with_utc() {
        let timestamp = 1231006505; // Unix timestamp in seconds
        let tz = PoSQLTimeZone::Utc;
        let normalized = PoSQLTimeZone::normalize_to_utc(timestamp, &tz);
        assert_eq!(normalized, timestamp); // No adjustment for UTC
    }

    #[test]
    fn test_normalize_to_utc_with_positive_offset() {
        let timestamp = 1231006505; // Unix timestamp in seconds
        let tz = PoSQLTimeZone::FixedOffset(3600); // UTC+1 (3600 seconds offset)
        let normalized = PoSQLTimeZone::normalize_to_utc(timestamp, &tz);
        assert_eq!(normalized, 1231006505 - 3600); // Adjusted by 1 hour
    }

    #[test]
    fn test_normalize_to_utc_with_negative_offset() {
        let timestamp = 1231006505; // Unix timestamp in seconds
        let tz = PoSQLTimeZone::FixedOffset(-18000); // UTC-5 (18000 seconds offset)
        let normalized = PoSQLTimeZone::normalize_to_utc(timestamp, &tz);
        assert_eq!(normalized, 1231006505 + 18000); // Adjusted by 5 hours
    }
}
