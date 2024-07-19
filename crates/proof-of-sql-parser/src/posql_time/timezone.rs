use crate::error::PoSQLTimestampError;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Captures a timezone from a timestamp query
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoSQLTimeZone {
    /// Default variant for UTC timezone
    Utc,
    /// TImezone offset in seconds
    FixedOffset(i32),
}

impl PoSQLTimeZone {
    /// Parse a timezone from a count of seconds
    pub fn from_offset(offset: i32) -> Self {
        if offset == 0 {
            PoSQLTimeZone::Utc
        } else {
            PoSQLTimeZone::FixedOffset(offset)
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
                    _ => Err(PoSQLTimestampError::InvalidTimezone(tz.to_string())),
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
                    write!(f, "+{:02}:{:02}", hours, minutes)
                }
            }
        }
    }
}

#[cfg(test)]
mod timezone_parsing_tests {
    use crate::posql_time::timezone;

    #[test]
    fn test_display_fixed_offset_positive() {
        let timezone = timezone::PoSQLTimeZone::FixedOffset(4500); // +01:15
        assert_eq!(format!("{}", timezone), "+01:15");
    }

    #[test]
    fn test_display_fixed_offset_negative() {
        let timezone = timezone::PoSQLTimeZone::FixedOffset(-3780); // -01:03
        assert_eq!(format!("{}", timezone), "-01:03");
    }

    #[test]
    fn test_display_utc() {
        let timezone = timezone::PoSQLTimeZone::Utc;
        assert_eq!(format!("{}", timezone), "+00:00");
    }
}

#[cfg(test)]
mod timezone_offset_tests {
    use crate::posql_time::{timestamp::PoSQLTimestamp, timezone};

    #[test]
    fn test_utc_timezone() {
        let input = "2023-06-26T12:34:56Z";
        let expected_timezone = timezone::PoSQLTimeZone::Utc;
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_positive_offset_timezone() {
        let input = "2023-06-26T12:34:56+03:30";
        let expected_timezone = timezone::PoSQLTimeZone::from_offset(12600); // 3 hours and 30 minutes in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_negative_offset_timezone() {
        let input = "2023-06-26T12:34:56-04:00";
        let expected_timezone = timezone::PoSQLTimeZone::from_offset(-14400); // -4 hours in seconds
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }

    #[test]
    fn test_zero_offset_timezone() {
        let input = "2023-06-26T12:34:56+00:00";
        let expected_timezone = timezone::PoSQLTimeZone::Utc; // Zero offset defaults to UTC
        let result = PoSQLTimestamp::try_from(input).unwrap();
        assert_eq!(result.timezone, expected_timezone);
    }
}
