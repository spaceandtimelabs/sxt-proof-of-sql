use super::{timeunit::PoSQLTimeUnit, timezone::PoSQLTimeZone};
use serde::{Deserialize, Serialize};

/// Intermediate Time
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Timestamp {
    /// Count of time units since the unix epoch
    pub timestamp: i64,
    /// Seconds, milliseconds, microseconds, or nanoseconds
    pub unit: PoSQLTimeUnit,
    /// Timezone captured from parsed string
    pub timezone: PoSQLTimeZone,
}