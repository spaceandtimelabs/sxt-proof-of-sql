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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ProofsTimeZone(pub Tz);

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

impl TryFrom<Option<&str>> for ProofsTimeZone {
    type Error = &'static str;

    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        match value {
            Some(tz_str) => Tz::from_str(tz_str)
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Tz;

    #[test]
    fn we_can_convert_valid_timezones() {
        let examples = ["Europe/London", "America/New_York", "Asia/Tokyo", "UTC"];

        for &tz_str in &examples {
            let timezone = ProofsTimeZone::try_from(Some(tz_str));
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
    fn we_cannot_convert_invalid_timezones() {
        let invalid_tz_str = "Not/A_TimeZone";
        let result = ProofsTimeZone::try_from(Some(invalid_tz_str));
        assert!(
            result.is_err(),
            "Should return an error for invalid timezones"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid timezone string",
            "Error message should indicate invalid timezone string"
        );
    }

    #[test]
    fn we_can_get_utc_with_none_timezone() {
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
