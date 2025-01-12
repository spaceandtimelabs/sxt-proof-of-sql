use alloc::{
    fmt,
    fmt::{Display, Formatter},
};
use serde::{Deserialize, Serialize};

/// Aggregation operators
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// Aggregation operators
pub enum AggOperator {
    /// Maximum
    Max,
    /// Minimum
    Min,
    /// Sum
    Sum,
    /// Count
    Count,
    /// Return the first value
    First,
}

impl Display for AggOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AggOperator::Max => write!(f, "max"),
            AggOperator::Min => write!(f, "min"),
            AggOperator::Sum => write!(f, "sum"),
            AggOperator::Count => write!(f, "count"),
            AggOperator::First => write!(f, "first"),
        }
    }
}
