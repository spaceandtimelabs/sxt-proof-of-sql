// Re-exports and module organization for the time unit functionality
mod types;
mod conversions;
mod display;

#[cfg(test)]
#[path = "tests/unit_tests.rs"]
mod tests;

pub use types::PoSQLTimeUnit;

// We don't need to re-export the implementations since they're automatically
// available when the type is used, but we do need to ensure they're all
// properly implemented in their respective modules:
// - From<PoSQLTimeUnit> for u64 (in conversions)
// - TryFrom<&str> for PoSQLTimeUnit (in conversions)
// - Display for PoSQLTimeUnit (in display)
// - Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq (derived in types)