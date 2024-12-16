use super::types::PoSQLTimeUnit;
use crate::posql_time::PoSQLTimestampError;

impl From<PoSQLTimeUnit> for u64 {
    fn from(value: PoSQLTimeUnit) -> u64 {
        match value {
            PoSQLTimeUnit::Second => 0,
            PoSQLTimeUnit::Millisecond => 3,
            PoSQLTimeUnit::Microsecond => 6,
            PoSQLTimeUnit::Nanosecond => 9,
        }
    }
}

impl TryFrom<&str> for PoSQLTimeUnit {
    type Error = PoSQLTimestampError;
    fn try_from(value: &str) -> Result<Self, PoSQLTimestampError> {
        match value {
            "0" => Ok(PoSQLTimeUnit::Second),
            "3" => Ok(PoSQLTimeUnit::Millisecond),
            "6" => Ok(PoSQLTimeUnit::Microsecond),
            "9" => Ok(PoSQLTimeUnit::Nanosecond),
            _ => Err(PoSQLTimestampError::UnsupportedPrecision {
                error: value.into(),
            }),
        }
    }
}