use super::ColumnType;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// This type is used to represent the metadata
/// of a column in a table. Namely: it's name and type.
///
/// This is the analog of a `Field` in Apache Arrow.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnField {
    name: Ident,
    data_type: ColumnType,
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    #[must_use]
    pub fn new(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
    }

    /// Returns the name of the column
    #[must_use]
    pub fn name(&self) -> Ident {
        self.name.clone()
    }

    /// Returns the type of the column
    #[must_use]
    pub fn data_type(&self) -> ColumnType {
        self.data_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_column_field_getters() {
        let ident = Ident::new("my_column");
        let field = ColumnField::new(ident.clone(), ColumnType::BigInt);

        assert_eq!(field.name(), ident);
        assert_eq!(field.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_column_field_serde_roundtrip() {
        let field = ColumnField::new(Ident::new("my_column"), ColumnType::BigInt);
        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: ColumnField = serde_json::from_str(&serialized).unwrap();
        assert_eq!(field, deserialized);
    }
}
