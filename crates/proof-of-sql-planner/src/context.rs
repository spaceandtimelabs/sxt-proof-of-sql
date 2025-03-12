use super::{column_fields_to_schema, table_reference_to_table_ref, PlannerResult};
use alloc::{format, sync::Arc};
use arrow::datatypes::{Field, Schema};
use core::any::Any;
use datafusion::{
    common::{
        arrow::datatypes::{DataType, SchemaRef},
        DFSchema, DataFusionError,
    },
    config::ConfigOptions,
    logical_expr::{
        AggregateUDF, Expr, ScalarUDF, TableProviderFilterPushDown, TableSource, WindowUDF,
    },
    sql::{planner::ContextProvider, TableReference},
};
use indexmap::IndexMap;
use proof_of_sql::base::{
    database::{ColumnField, Table, TableRef},
    scalar::Scalar,
};

/// A [`ContextProvider`] implementation for Proof of SQL
///
/// This provider is used to provide tables to the Proof of SQL planner
pub struct PoSqlContextProvider<'a, S: Scalar> {
    tables: IndexMap<TableRef, Table<'a, S>>,
    options: ConfigOptions,
}

impl<S: Scalar> Default for PoSqlContextProvider<'_, S> {
    fn default() -> Self {
        Self::new(IndexMap::new())
    }
}

impl<'a, S: Scalar> PoSqlContextProvider<'a, S> {
    /// Create a new `PoSqlContextProvider`
    #[must_use]
    pub fn new(tables: IndexMap<TableRef, Table<'a, S>>) -> Self {
        Self {
            tables,
            options: ConfigOptions::default(),
        }
    }

    /// Get the [`DFSchemas`] of the tables in this provider
    pub fn try_get_df_schemas(&self) -> PlannerResult<IndexMap<TableReference, DFSchema>> {
        self.tables
            .iter()
            .map(|(table_ref, table)| {
                let table_reference = TableReference::from(table_ref.to_string());
                Ok((
                    table_reference.clone(),
                    DFSchema::try_from_qualified_schema(
                        table_reference.clone(),
                        &column_fields_to_schema(table.schema()),
                    )?,
                ))
            })
            .collect::<PlannerResult<IndexMap<_, _>>>()
    }
}

impl<S: Scalar> ContextProvider for PoSqlContextProvider<'_, S> {
    fn get_table_source(
        &self,
        name: TableReference,
    ) -> Result<Arc<dyn TableSource>, DataFusionError> {
        let table_ref = table_reference_to_table_ref(&name)
            .map_err(|err| DataFusionError::External(Box::new(err)))?;
        self.tables
            .get(&table_ref)
            .ok_or_else(|| {
                DataFusionError::Plan(format!("Table {} not found", name.to_quoted_string()))
            })
            .map(|table| Arc::new(PoSqlTableSource::new(table.schema())) as Arc<dyn TableSource>)
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
    use alloc::vec;
    use bumpalo::Bump;
    use core::any::TypeId;
    use indexmap::indexmap;
    use proof_of_sql::{
        base::database::{table_utility::*, ColumnType},
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    };

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
        let context_provider = PoSqlContextProvider::<Curve25519Scalar>::default();
        assert_eq!(context_provider.tables, IndexMap::new());
        assert_eq!(
            context_provider.try_get_df_schemas().unwrap(),
            IndexMap::new()
        );
        assert_eq!(context_provider.udfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udafs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udwfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.get_variable_type(&[]), None);
        assert_eq!(context_provider.get_function_meta(""), None);
        assert_eq!(context_provider.get_aggregate_meta(""), None);
        assert_eq!(context_provider.get_window_meta(""), None);
        assert!(matches!(
            context_provider.get_table_source(TableReference::from("namespace.table")),
            Err(DataFusionError::Plan(_))
        ));

        // Non-empty
        let alloc = Bump::new();
        let tables = indexmap! {
                TableRef::new("namespace", "a") =>
                table(
                    vec![
                        borrowed_smallint("a", [1_i16, 2, 3], &alloc),
                        borrowed_varchar("b", ["Space", "and", "Time"], &alloc),
                    ]
                ),
                TableRef::new("namespace", "b") =>
                table(
                    vec![
                        borrowed_int("c", [1, 2, 3], &alloc),
                        borrowed_bigint("d", [1_i64, 2, 3], &alloc),
                    ]
                )
        };
        let context_provider = PoSqlContextProvider::<Curve25519Scalar>::new(tables.clone());
        let schema_a = Schema::new(vec![
            Field::new("a", DataType::Int16, false),
            Field::new("b", DataType::Utf8, false),
        ]);
        let schema_b = Schema::new(vec![
            Field::new("c", DataType::Int32, false),
            Field::new("d", DataType::Int64, false),
        ]);
        assert_eq!(context_provider.tables, tables);
        assert_eq!(
            context_provider.try_get_df_schemas().unwrap(),
            indexmap! {
                TableReference::from("namespace.a") => DFSchema::try_from_qualified_schema(
                    "namespace.a",
                    &schema_a
                ).unwrap(),
                TableReference::from("namespace.b") => DFSchema::try_from_qualified_schema(
                    "namespace.b",
                    &schema_b
                ).unwrap(),
            }
        );
        assert_eq!(context_provider.udfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udafs_names(), Vec::<String>::new());
        assert_eq!(context_provider.udwfs_names(), Vec::<String>::new());
        assert_eq!(context_provider.get_variable_type(&[]), None);
        assert_eq!(context_provider.get_function_meta(""), None);
        assert_eq!(context_provider.get_aggregate_meta(""), None);
        assert_eq!(context_provider.get_window_meta(""), None);
        assert!(matches!(
            context_provider.get_table_source(TableReference::from("namespace.table")),
            Err(DataFusionError::Plan(_))
        ));
    }

    #[test]
    fn we_cannot_create_a_posql_context_provider_if_catalog_provided() {
        let context_provider = PoSqlContextProvider::<Curve25519Scalar>::new(IndexMap::new());
        assert!(matches!(
            context_provider.get_table_source(TableReference::from("catalog.namespace.table")),
            Err(DataFusionError::External(_))
        ));
    }
}
