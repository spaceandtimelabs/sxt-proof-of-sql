use super::table_reference_to_table_ref;
use crate::schema_to_column_fields;
use alloc::sync::Arc;
use arrow::datatypes::{Field, Schema};
use core::any::Any;
use datafusion::{
    common::{
        arrow::datatypes::{DataType, SchemaRef},
        DataFusionError,
    },
    config::ConfigOptions,
    logical_expr::{
        AggregateUDF, Expr, ScalarUDF, TableProviderFilterPushDown, TableSource, WindowUDF,
    },
    sql::{planner::ContextProvider, TableReference},
};
use proof_of_sql::base::database::{ColumnField, SchemaAccessor};

/// A [`ContextProvider`] implementation for Proof of SQL
///
/// This provider is used to provide tables to the Proof of SQL planner
pub struct PoSqlContextProvider<A: SchemaAccessor> {
    accessor: A,
    options: ConfigOptions,
}

impl<A: SchemaAccessor> PoSqlContextProvider<A> {
    /// Create a new `PoSqlContextProvider`
    #[must_use]
    pub fn new(accessor: A) -> Self {
        Self {
            accessor,
            options: ConfigOptions::default(),
        }
    }
}

impl<A: SchemaAccessor> ContextProvider for PoSqlContextProvider<A> {
    fn get_table_source(
        &self,
        name: TableReference,
    ) -> Result<Arc<dyn TableSource>, DataFusionError> {
        let table_ref = table_reference_to_table_ref(&name)
            .map_err(|err| DataFusionError::External(Box::new(err)))?;
        let schema = self.accessor.lookup_schema(&table_ref);
        let column_fields = schema_to_column_fields(schema);
        Ok(Arc::new(PoSqlTableSource::new(column_fields)) as Arc<dyn TableSource>)
    }
    fn get_function_meta(&self, _name: &str) -> Option<Arc<ScalarUDF>> {
        None
    }
    //TODO: add count and sum
    fn get_aggregate_meta(&self, _name: &str) -> Option<Arc<AggregateUDF>> {
        None
    }
    fn get_window_meta(&self, _name: &str) -> Option<Arc<WindowUDF>> {
        None
    }
    fn get_variable_type(&self, _variable_names: &[String]) -> Option<DataType> {
        None
    }
    fn options(&self) -> &ConfigOptions {
        &self.options
    }
    fn udfs_names(&self) -> Vec<String> {
        Vec::new()
    }
    fn udafs_names(&self) -> Vec<String> {
        Vec::new()
    }
    fn udwfs_names(&self) -> Vec<String> {
        Vec::new()
    }
}

/// A [`TableSource`] implementation for Proof of SQL
pub(crate) struct PoSqlTableSource {
    schema: SchemaRef,
}

impl PoSqlTableSource {
    /// Create a new `PoSqlTableSource`
    pub(crate) fn new(column_fields: Vec<ColumnField>) -> Self {
        let arrow_schema = Schema::new(
            column_fields
                .into_iter()
                .map(|column_field| {
                    Field::new(
                        column_field.name().value.as_str(),
                        (&column_field.data_type()).into(),
                        false,
                    )
                })
                .collect::<Vec<_>>(),
        );
        Self {
            schema: Arc::new(arrow_schema),
        }
    }
}

impl TableSource for PoSqlTableSource {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>, DataFusionError> {
        Ok(vec![TableProviderFilterPushDown::Exact; filters.len()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHasher;
    use alloc::vec;
    use core::any::TypeId;
    use indexmap::indexmap_with_default;
    use proof_of_sql::base::database::{ColumnType, TableRef, TestSchemaAccessor};

    // PoSqlTableSource
    #[test]
    fn we_can_create_a_posql_table_source() {
        // Empty
        let table_source = PoSqlTableSource::new(vec![]);
        assert_eq!(table_source.schema().all_fields(), Vec::<&Field>::new());
        assert_eq!(
            table_source.as_any().type_id(),
            TypeId::of::<PoSqlTableSource>()
        );

        // Non-empty
        let column_fields = vec![
            ColumnField::new("a".into(), ColumnType::SmallInt),
            ColumnField::new("b".into(), ColumnType::VarChar),
        ];
        let table_source = PoSqlTableSource::new(column_fields);
        assert_eq!(
            table_source.schema().all_fields(),
            vec![
                &Field::new("a", DataType::Int16, false),
                &Field::new("b", DataType::Utf8, false),
            ]
        );
        assert_eq!(
            table_source.as_any().type_id(),
            TypeId::of::<PoSqlTableSource>()
        );
    }

    // PoSqlContextProvider
    #[test]
    fn we_can_create_a_posql_context_provider() {
        // Empty
        let accessor = TestSchemaAccessor::new(indexmap_with_default! {AHasher;});
        let context_provider = PoSqlContextProvider::new(accessor);
        assert_eq!(context_provider.udfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udafs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udwfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.get_variable_type(&[]), None);
        assert_eq!(context_provider.get_function_meta(""), None);
        assert_eq!(context_provider.get_aggregate_meta(""), None);
        assert_eq!(context_provider.get_window_meta(""), None);
        assert_eq!(
            context_provider
                .get_table_source(TableReference::from("namespace.table"))
                .unwrap()
                .schema(),
            PoSqlTableSource::new(Vec::new()).schema()
        );

        // Non-empty
        let accessor = TestSchemaAccessor::new(indexmap_with_default! {AHasher;
            TableRef::new("namespace", "a") => indexmap_with_default! {AHasher;
                "a".into() => ColumnType::SmallInt,
                "b".into() => ColumnType::VarChar
            },
            TableRef::new("namespace", "b") => indexmap_with_default! {AHasher;
                "c".into() => ColumnType::Int,
                "d".into() => ColumnType::BigInt
            },
        });
        let context_provider = PoSqlContextProvider::new(accessor);
        assert_eq!(context_provider.udfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udafs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udwfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.get_variable_type(&[]), None);
        assert_eq!(context_provider.get_function_meta(""), None);
        assert_eq!(context_provider.get_aggregate_meta(""), None);
        assert_eq!(context_provider.get_window_meta(""), None);
        assert_eq!(
            context_provider
                .get_table_source(TableReference::from("namespace.a"))
                .unwrap()
                .schema(),
            Arc::new(PoSqlTableSource::new(vec![
                ColumnField::new("a".into(), ColumnType::SmallInt),
                ColumnField::new("b".into(), ColumnType::VarChar)
            ]))
            .schema()
        );
    }

    #[test]
    fn we_cannot_create_a_posql_context_provider_if_catalog_provided() {
        let accessor = TestSchemaAccessor::new(indexmap_with_default! {AHasher;});
        let context_provider = PoSqlContextProvider::new(accessor);
        assert!(matches!(
            context_provider.get_table_source(TableReference::from("catalog.namespace.table")),
            Err(DataFusionError::External(_))
        ));
    }
}
