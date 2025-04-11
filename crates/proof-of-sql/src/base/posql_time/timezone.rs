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
}
