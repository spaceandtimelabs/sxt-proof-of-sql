use super::{ColumnType, SchemaAccessor, TableRef};
use crate::base::map::IndexMap;
use sqlparser::ast::Ident;

/// A simple in-memory `SchemaAccessor` for testing intermediate AST -> Provable AST conversion.
pub struct TestSchemaAccessor {
    schemas: IndexMap<TableRef, IndexMap<Ident, ColumnType>>,
}

impl TestSchemaAccessor {
    /// Create a new `TestSchemaAccessor` with the given schema.
    pub fn new(schemas: IndexMap<TableRef, IndexMap<Ident, ColumnType>>) -> Self {
        Self { schemas }
    }
}

impl SchemaAccessor for TestSchemaAccessor {
    fn lookup_column(&self, table_ref: TableRef, column_id: Ident) -> Option<ColumnType> {
        self.schemas.get(&table_ref)?.get(&column_id).copied()
    }

    fn lookup_schema(&self, table_ref: TableRef) -> Vec<(Ident, ColumnType)> {
        self.schemas
            .get(&table_ref)
            .unwrap_or(&IndexMap::default())
            .iter()
            .map(|(id, col)| (id.clone(), *col))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::map::indexmap;
    use proof_of_sql_parser::sqlparser::object_name_from;

    fn sample_test_schema_accessor() -> TestSchemaAccessor {
        let table1: TableRef = TableRef::new(object_name_from("schema.table1"));
        let table2: TableRef = TableRef::new(object_name_from("schema.table2"));
        TestSchemaAccessor::new(indexmap! {
            table1 => indexmap! {
                "col1".into() => ColumnType::BigInt,
                "col2".into() => ColumnType::VarChar,
            },
            table2 => indexmap! {
                "col1".into() => ColumnType::BigInt,
            },
        })
    }

    #[test]
    fn test_lookup_column() {
        let accessor = sample_test_schema_accessor();
        let table1: TableRef = TableRef::new(object_name_from("schema.table1"));
        let table2: TableRef = TableRef::new(object_name_from("schema.table2"));
        let not_a_table: TableRef = TableRef::new(object_name_from("schema.not_a_table"));
        assert_eq!(
            accessor.lookup_column(table1, "col1".into()),
            Some(ColumnType::BigInt)
        );
        assert_eq!(
            accessor.lookup_column(table1, "col2".into()),
            Some(ColumnType::VarChar)
        );
        assert_eq!(accessor.lookup_column(table1, "not_a_col".into()), None);
        assert_eq!(
            accessor.lookup_column(table2, "col1".into()),
            Some(ColumnType::BigInt)
        );
        assert_eq!(accessor.lookup_column(table2, "col2".into()), None);
        assert_eq!(accessor.lookup_column(not_a_table, "col1".into()), None);
        assert_eq!(accessor.lookup_column(not_a_table, "col2".into()), None);
        assert_eq!(
            accessor.lookup_column(not_a_table, "not_a_col".into()),
            None
        );
    }

    #[test]
    fn test_lookup_schema() {
        let accessor = sample_test_schema_accessor();
        let table1: TableRef = TableRef::new(object_name_from("schema.table1"));
        let table2: TableRef = TableRef::new(object_name_from("schema.table2"));
        let not_a_table: TableRef = TableRef::new(object_name_from("schema.not_a_table"));
        assert_eq!(
            accessor.lookup_schema(table1),
            vec![
                ("col1".into(), ColumnType::BigInt),
                ("col2".into(), ColumnType::VarChar),
            ]
        );
        assert_eq!(
            accessor.lookup_schema(table2),
            vec![("col1".into(), ColumnType::BigInt),]
        );
        assert_eq!(accessor.lookup_schema(not_a_table), vec![]);
    }
}
