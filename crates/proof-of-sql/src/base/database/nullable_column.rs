use crate::base::database::Column;
use crate::base::scalar::Scalar;

/// Represents a nullable column with values and an optional presence slice
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct NullableColumn<'a, T: Scalar> {
    values: Column<'a, T>,
    presence: Option<&'a [bool]>,
}

impl<'a, T: Scalar> NullableColumn<'a, T> {
    /// Create a new nullable column
    pub fn new(values: Column<'a, T>, presence: Option<&'a [bool]>) -> Self {
        Self { values, presence }
    }

    /// Check if a specific value is null
    pub fn is_null(&self, index: usize) -> bool {
        self.presence
            .map_or(false, |p| !p.get(index).map(|v| *v).unwrap_or_default())
    }
}
