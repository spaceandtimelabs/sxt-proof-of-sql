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
                        if hours > 12 || minutes >= 60 {
                            return Err(PoSQLTimestampError::InvalidTimezoneOffset);
                        }
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
mod timezone_arc_str_parsing {

    use super::*;
    use crate::posql_time::{timezone, PoSQLTimestampError::InvalidTimezoneOffset};
    use alloc::format;

    #[test]
    fn test_parsing_from_arc_str_fixed_offset() {
        let ss = "00:00";
        let timezone_arc: Arc<str> = Arc::from(ss);
        let timezone = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)).unwrap(); // +01:15
        assert_eq!(format!("{timezone}"), "+00:00");
    }

    #[test]
    fn test_parsing_from_arc_str_fixed_offset_positive() {
        let input_timezone = "+01:15";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let timezone = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)).unwrap(); // +01:15
        assert_eq!(format!("{timezone}"), "+01:15");
    }

    #[test]
    fn test_parsing_from_arc_str_fixed_offset_negative() {
        let input_timezone = "-01:03";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let timezone = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)).unwrap(); // +01:15
        assert_eq!(format!("{timezone}"), "-01:03");
    }

    #[test]
    fn check_for_invalid_timezone_hour_offset() {
        let input_timezone = "-0A:03";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)); // should be invalid time offset error
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));

        let input_timezone = "-13:03";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)); // should be invalid time offset error
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));

        let input_timezone = "-11:60";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)); // should be invalid time offset error
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));
    }

    #[test]
    fn check_for_invalid_timezone_minute_offset() {
        let input_timezone = "-00:B3";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)); // should be invalid time offset error
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));
        let input_timezone = "-00:83";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)); // should be invalid time offset error
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));
    }

    #[test]
    fn test_invalid_timezone() {
        let expected = PoSQLTimestampError::InvalidTimezone {
            timezone: "WRONG".to_string(),
        };
        let timezone_input = "WRONG";
        let timezone_arc: Arc<str> = Arc::from(timezone_input);
        let timezone_err = timezone::PoSQLTimeZone::try_from(&Some(timezone_arc)); // +01:15
        assert_eq!(expected, timezone_err.err().unwrap());
    }

    #[test]
    fn test_when_none() {
        let timezone = timezone::PoSQLTimeZone::try_from(&None).unwrap(); // +01:15
        assert_eq!(format!("{timezone}"), "+00:00");
    }
}

#[cfg(test)]
mod timezone_parsing_tests {
    use crate::posql_time::timezone;
    use alloc::format;

    #[test]
    fn test_display_fixed_offset_positive() {
        let timezone = timezone::PoSQLTimeZone::new(4500); // +01:15
        assert_eq!(format!("{timezone}"), "+01:15");
    }

    #[test]
    fn test_display_fixed_offset_negative() {
        let timezone = timezone::PoSQLTimeZone::new(-3780); // -01:03
        assert_eq!(format!("{timezone}"), "-01:03");
    }

    #[test]
    fn test_display_utc() {
        let timezone = timezone::PoSQLTimeZone::utc();
        assert_eq!(format!("{timezone}"), "+00:00");
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
