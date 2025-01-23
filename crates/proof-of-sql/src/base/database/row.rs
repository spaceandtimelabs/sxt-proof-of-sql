use super::LiteralValue;
use alloc::vec::Vec;

/// A single row of data, holding a list of literal values.
pub struct Row {
    values: Vec<LiteralValue>,
}

impl Row {
    /// Create a new `Row` from a vector of values.
    pub fn new(values: Vec<LiteralValue>) -> Self {
        Row { values }
    }

    /// Get the values of the row.
    #[must_use]
    pub fn values(&self) -> &[LiteralValue] {
        &self.values
    }

    /// Get the length of the row.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Is the row empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
