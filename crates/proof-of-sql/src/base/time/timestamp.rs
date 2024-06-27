use arrow::datatypes::TimeUnit as ArrowTimeUnit;
use core::fmt;
use serde::{Deserialize, Serialize};

/// A wrapper around i64 to mitigate conflicting From<i64>
/// implementations
#[derive(Clone, Copy)]
pub struct Time {
    /// i64 count of timeunits since unix epoch
    pub timestamp: i64,
    /// Timeunit of this time
    pub unit: PoSQLTimeUnit,
}

/// Specifies different units of time measurement relative to the Unix epoch. It is essentially
/// a wrapper over [arrow::datatypes::TimeUnit] so that we can derive Copy and implement custom traits
/// such as bit distribution and Hash.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, Hash)]
pub enum PoSQLTimeUnit {
    /// Represents a time unit of one second.
    Second,
    /// Represents a time unit of one millisecond (1/1,000 of a second).
    Millisecond,
    /// Represents a time unit of one microsecond (1/1,000,000 of a second).
    Microsecond,
    /// Represents a time unit of one nanosecond (1/1,000,000,000 of a second).
    Nanosecond,
}

impl From<PoSQLTimeUnit> for ArrowTimeUnit {
    fn from(unit: PoSQLTimeUnit) -> Self {
        match unit {
            PoSQLTimeUnit::Second => ArrowTimeUnit::Second,
            PoSQLTimeUnit::Millisecond => ArrowTimeUnit::Millisecond,
            PoSQLTimeUnit::Microsecond => ArrowTimeUnit::Microsecond,
            PoSQLTimeUnit::Nanosecond => ArrowTimeUnit::Nanosecond,
        }
    }
}

impl fmt::Display for PoSQLTimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoSQLTimeUnit::Second => write!(f, "Second"),
            PoSQLTimeUnit::Millisecond => write!(f, "Millisecond"),
            PoSQLTimeUnit::Microsecond => write!(f, "Microsecond"),
            PoSQLTimeUnit::Nanosecond => write!(f, "Nanosecond"),
        }
    }
}

impl From<ArrowTimeUnit> for PoSQLTimeUnit {
    fn from(unit: ArrowTimeUnit) -> Self {
        match unit {
            ArrowTimeUnit::Second => PoSQLTimeUnit::Second,
            ArrowTimeUnit::Millisecond => PoSQLTimeUnit::Millisecond,
            ArrowTimeUnit::Microsecond => PoSQLTimeUnit::Microsecond,
            ArrowTimeUnit::Nanosecond => PoSQLTimeUnit::Nanosecond,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_convert_from_arrow_time_units() {
        assert_eq!(
            PoSQLTimeUnit::from(ArrowTimeUnit::Second),
            PoSQLTimeUnit::Second
        );
        assert_eq!(
            PoSQLTimeUnit::from(ArrowTimeUnit::Millisecond),
            PoSQLTimeUnit::Millisecond
        );
        assert_eq!(
            PoSQLTimeUnit::from(ArrowTimeUnit::Microsecond),
            PoSQLTimeUnit::Microsecond
        );
        assert_eq!(
            PoSQLTimeUnit::from(ArrowTimeUnit::Nanosecond),
            PoSQLTimeUnit::Nanosecond
        );
    }

    #[test]
    fn we_can_convert_to_arrow_time_units() {
        assert_eq!(
            ArrowTimeUnit::from(PoSQLTimeUnit::Second),
            ArrowTimeUnit::Second
        );
        assert_eq!(
            ArrowTimeUnit::from(PoSQLTimeUnit::Millisecond),
            ArrowTimeUnit::Millisecond
        );
        assert_eq!(
            ArrowTimeUnit::from(PoSQLTimeUnit::Microsecond),
            ArrowTimeUnit::Microsecond
        );
        assert_eq!(
            ArrowTimeUnit::from(PoSQLTimeUnit::Nanosecond),
            ArrowTimeUnit::Nanosecond
        );
    }
}
