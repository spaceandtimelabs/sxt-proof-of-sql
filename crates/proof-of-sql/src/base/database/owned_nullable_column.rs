use crate::base::database::OwnedColumn;
use crate::base::scalar::Scalar;

/// Represents a nullable column with values and an optional presence slice
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct NullableOwnedColumn<T: Scalar> {
    pub values: OwnedColumn<T>,
    pub presence: Option<Vec<bool>>,
}

impl<T: Scalar> NullableOwnedColumn<T> {
    /// Create a new nullable column
    pub fn new(values: OwnedColumn<T>, presence: Option<Vec<bool>>) -> Self {
        Self { values, presence }
    }

    /// Check if a specific value is null
    pub fn is_null(&self, index: usize) -> bool {
        self.presence
            .as_ref()
            .map_or(false, |p| !p.get(index).map(|v| *v).unwrap_or_default())
    }
}
