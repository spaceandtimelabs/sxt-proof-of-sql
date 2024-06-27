use super::error::TimeError;
use chrono::{DateTime, FixedOffset, Utc};
use chrono_tz::Tz;
use core::fmt;
use proof_of_sql_parser::intermediate_time::IntermediateTimeZone;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

/// A typed TimeZone for a [`TimeStamp`]. It is optionally
/// used to define a timezone other than UTC for a new TimeStamp.
/// It exists as a wrapper around chrono-tz because chrono-tz does
/// not implement uniform bit distribution
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PoSQLTimeZone(Tz);

impl PoSQLTimeZone {
    /// Convenience constant for the UTC timezone
    pub const UTC: PoSQLTimeZone = PoSQLTimeZone(Tz::UTC);

    /// Create a new ProofsTimeZone from a chrono TimeZone
    pub fn new(tz: Tz) -> Self {
        PoSQLTimeZone(tz)
    }
}

impl TryFrom<IntermediateTimeZone> for PoSQLTimeZone {
    type Error = TimeError;

    fn try_from(tz: IntermediateTimeZone) -> Result<Self, Self::Error> {
        match tz {
            IntermediateTimeZone::Utc => Ok(PoSQLTimeZone::UTC),
            IntermediateTimeZone::FixedOffset(seconds) => FixedOffset::east_opt(seconds)
                .ok_or(TimeError::InvalidTimezoneOffset)
                .and_then(|fixed_offset| {
                    let datetime: DateTime<Utc> = Utc::now();
                    let offset_datetime = datetime.with_timezone(&fixed_offset);
                    let tz_string = offset_datetime.format("%Z").to_string();
                    Tz::from_str(&tz_string)
                        .map(PoSQLTimeZone)
                        .map_err(|_| TimeError::TimeZoneStringParseError)
                }),
        }
    }
}

impl From<&PoSQLTimeZone> for Arc<str> {
    fn from(timezone: &PoSQLTimeZone) -> Self {
        Arc::from(timezone.0.name())
    }
}

impl From<Tz> for PoSQLTimeZone {
    fn from(tz: Tz) -> Self {
        PoSQLTimeZone(tz)
    }
}

impl fmt::Display for PoSQLTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<Option<Arc<str>>> for PoSQLTimeZone {
    type Error = TimeError;

    fn try_from(value: Option<Arc<str>>) -> Result<Self, Self::Error> {
        match value {
            Some(arc_str) => Tz::from_str(&arc_str)
                .map(PoSQLTimeZone)
                .map_err(|_| TimeError::InvalidTimezone("Invalid timezone string".to_string())),
            None => Ok(PoSQLTimeZone(Tz::UTC)), // Default to UTC
        }
    }
}

impl TryFrom<&str> for PoSQLTimeZone {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Tz::from_str(value)
            .map(PoSQLTimeZone)
            .map_err(|_| "Invalid timezone string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Tz;

    #[test]
    fn valid_timezones_convert_correctly() {
        let valid_timezones = ["Europe/London", "America/New_York", "Asia/Tokyo", "UTC"];

        for tz_str in &valid_timezones {
            let arc_tz = Arc::new(tz_str.to_string());
            // Convert Arc<String> to Arc<str> by dereferencing to &str then creating a new Arc
            let arc_tz_str: Arc<str> = Arc::from(&**arc_tz);
            let timezone = PoSQLTimeZone::try_from(Some(arc_tz_str));
            assert!(timezone.is_ok(), "Timezone should be valid: {}", tz_str);
            assert_eq!(
                timezone.unwrap().0,
                Tz::from_str(tz_str).unwrap(),
                "Timezone mismatch for {}",
                tz_str
            );
        }
    }

    #[test]
    fn test_edge_timezone_strings() {
        let edge_timezones = ["Etc/GMT+12", "Etc/GMT-14", "America/Argentina/Ushuaia"];
        for tz_str in &edge_timezones {
            let arc_tz = Arc::from(*tz_str);
            let result = PoSQLTimeZone::try_from(Some(arc_tz));
            assert!(result.is_ok(), "Edge timezone should be valid: {}", tz_str);
            assert_eq!(
                result.unwrap().0,
                Tz::from_str(tz_str).unwrap(),
                "Mismatch for edge timezone {}",
                tz_str
            );
        }
    }

    #[test]
    fn test_empty_timezone_string() {
        let empty_tz = Arc::from("");
        let result = PoSQLTimeZone::try_from(Some(empty_tz));
        assert!(result.is_err(), "Empty timezone string should fail");
    }

    #[test]
    fn test_unicode_timezone_strings() {
        let unicode_tz = Arc::from("Europe/Paris\u{00A0}"); // Non-breaking space character
        let result = PoSQLTimeZone::try_from(Some(unicode_tz));
        assert!(
            result.is_err(),
            "Unicode characters should not be valid in timezone strings"
        );
    }

    #[test]
    fn test_null_option() {
        let result = PoSQLTimeZone::try_from(None);
        assert!(result.is_ok(), "None should convert without error");
        assert_eq!(result.unwrap().0, Tz::UTC, "None should default to UTC");
    }
}
