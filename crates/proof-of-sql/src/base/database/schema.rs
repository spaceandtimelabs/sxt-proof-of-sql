use super::ColumnField;
use alloc::vec::Vec;

/// Table Schema
pub struct Schema {
    /// Column Fields
    fields: Vec<ColumnField>,
}

impl Schema {
    /// Create a new schema
    #[must_use]
    pub fn new(fields: Vec<ColumnField>) -> Self {
        Self { fields }
    }

    /// Get the fields
    #[must_use]
    pub fn fields(&self) -> &[ColumnField] {
        &self.fields
    }

    /// Get the i-th field
    #[must_use]
    pub fn field(&self, i: usize) -> &ColumnField {
        &self.fields[i]
    }

    /// Get index of a field by name
    ///
    /// If the field does not exist, return None
    #[must_use]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.fields
            .iter()
            .position(|field| field.name() == name.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::database::ColumnType;

    #[test]
    fn we_can_make_a_schema() {
        let schema = Schema::new(vec![
            ColumnField::new("name".into(), ColumnType::VarChar),
            ColumnField::new("age".into(), ColumnType::Int),
        ]);

        assert_eq!(schema.fields().len(), 2);
        assert_eq!(schema.field(0).name(), "name".into());
        assert_eq!(schema.field(0).data_type(), ColumnType::VarChar);
        assert_eq!(schema.field(1).name(), "age".into());
        assert_eq!(schema.field(1).data_type(), ColumnType::Int);
    }

    #[test]
    fn we_can_get_the_index_of_a_field() {
        let schema = Schema::new(vec![
            ColumnField::new("name".into(), ColumnType::VarChar),
            ColumnField::new("age".into(), ColumnType::Int),
        ]);

        assert_eq!(schema.index_of("name"), Some(0));
        assert_eq!(schema.index_of("age"), Some(1));
        // For a field that doesn't exist we should get None
        assert_eq!(schema.index_of("not_a_column"), None);
    }

    #[test]
    fn we_can_make_an_empty_schema() {
        let schema = Schema::new(vec![]);
        assert_eq!(schema.fields().len(), 0);
        // For a field that doesn't exist we should get None
        assert_eq!(schema.index_of("not_a_column"), None);
    }
}
