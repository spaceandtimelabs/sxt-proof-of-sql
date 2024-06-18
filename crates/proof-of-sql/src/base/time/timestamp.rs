use crate::base::database::ArrowArrayToColumnConversionError;
use arrow::datatypes::TimeUnit as ArrowTimeUnit;
use chrono_tz::Tz;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Timestamp {
    time: i64,
    timeunit: ProofsTimeUnit,
    timezone: Tz,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ProofsTimeZone(Tz);

impl fmt::Display for ProofsTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ProofsTimeUnit> for ArrowTimeUnit {
    fn from(unit: ProofsTimeUnit) -> Self {
        match unit {
            ProofsTimeUnit::Second => ArrowTimeUnit::Second,
            ProofsTimeUnit::Millisecond => ArrowTimeUnit::Millisecond,
            ProofsTimeUnit::Microsecond => ArrowTimeUnit::Microsecond,
            ProofsTimeUnit::Nanosecond => ArrowTimeUnit::Nanosecond,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, Hash)]
pub enum ProofsTimeUnit {
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl fmt::Display for ProofsTimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProofsTimeUnit::Second => write!(f, "Second"),
            ProofsTimeUnit::Millisecond => write!(f, "Millisecond"),
            ProofsTimeUnit::Microsecond => write!(f, "Microsecond"),
            ProofsTimeUnit::Nanosecond => write!(f, "Nanosecond"),
        }
    }
}

impl From<ArrowTimeUnit> for ProofsTimeUnit {
    fn from(unit: ArrowTimeUnit) -> Self {
        match unit {
            ArrowTimeUnit::Second => ProofsTimeUnit::Second,
            ArrowTimeUnit::Millisecond => ProofsTimeUnit::Millisecond,
            ArrowTimeUnit::Microsecond => ProofsTimeUnit::Microsecond,
            ArrowTimeUnit::Nanosecond => ProofsTimeUnit::Nanosecond,
        }
    }
}

impl TryFrom<Option<Arc<str>>> for ProofsTimeZone {
    type Error = &'static str; // Explicitly state the error type

    fn try_from(value: Option<Arc<str>>) -> Result<Self, Self::Error> {
        match value {
            Some(arc_str) => Tz::from_str(&arc_str)
                .map(ProofsTimeZone)
                .map_err(|_| "Invalid timezone string"),
            None => Ok(ProofsTimeZone(Tz::UTC)), // Default to UTC
        }
    }
}

impl From<&ProofsTimeZone> for Arc<str> {
    fn from(timezone: &ProofsTimeZone) -> Self {
        Arc::from(timezone.0.name())
    }
}

impl From<&'static str> for ArrowArrayToColumnConversionError {
    fn from(error: &'static str) -> Self {
        ArrowArrayToColumnConversionError::TimezoneConversionError(error.to_string())
    }
}

impl From<Tz> for ProofsTimeZone {
    fn from(tz: Tz) -> Self {
        ProofsTimeZone(tz)
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
            let timezone = ProofsTimeZone::try_from(Some(arc_tz_str));
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
            let result = ProofsTimeZone::try_from(Some(arc_tz));
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
        let result = ProofsTimeZone::try_from(Some(empty_tz));
        assert!(result.is_err(), "Empty timezone string should fail");
    }

    #[test]
    fn test_unicode_timezone_strings() {
        let unicode_tz = Arc::from("Europe/Paris\u{00A0}"); // Non-breaking space character
        let result = ProofsTimeZone::try_from(Some(unicode_tz));
        assert!(
            result.is_err(),
            "Unicode characters should not be valid in timezone strings"
        );
    }

    #[test]
    fn test_null_option() {
        let result = ProofsTimeZone::try_from(None);
        assert!(result.is_ok(), "None should convert without error");
        assert_eq!(result.unwrap().0, Tz::UTC, "None should default to UTC");
    }

    #[test]
    fn we_can_convert_from_arrow_time_units() {
        assert_eq!(
            ProofsTimeUnit::from(ArrowTimeUnit::Second),
            ProofsTimeUnit::Second
        );
        assert_eq!(
            ProofsTimeUnit::from(ArrowTimeUnit::Millisecond),
            ProofsTimeUnit::Millisecond
        );
        assert_eq!(
            ProofsTimeUnit::from(ArrowTimeUnit::Microsecond),
            ProofsTimeUnit::Microsecond
        );
        assert_eq!(
            ProofsTimeUnit::from(ArrowTimeUnit::Nanosecond),
            ProofsTimeUnit::Nanosecond
        );
    }

    #[test]
    fn we_can_convert_to_arrow_time_units() {
        assert_eq!(
            ArrowTimeUnit::from(ProofsTimeUnit::Second),
            ArrowTimeUnit::Second
        );
        assert_eq!(
            ArrowTimeUnit::from(ProofsTimeUnit::Millisecond),
            ArrowTimeUnit::Millisecond
        );
        assert_eq!(
            ArrowTimeUnit::from(ProofsTimeUnit::Microsecond),
            ArrowTimeUnit::Microsecond
        );
        assert_eq!(
            ArrowTimeUnit::from(ProofsTimeUnit::Nanosecond),
            ArrowTimeUnit::Nanosecond
        );
    }
}
