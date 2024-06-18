use arrow::datatypes::TimeUnit as ArrowTimeUnit;
use chrono_tz::Tz;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc}; // Tz implements the TimeZone trait and provides access to IANA time zones

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
    type Error = &'static str; // Or use a more descriptive error type

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
