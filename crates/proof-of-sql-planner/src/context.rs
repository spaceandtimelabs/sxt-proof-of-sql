use alloc::sync::Arc;
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
use proof_of_sql::base::map::IndexMap;

pub struct PoSqlContextProvider {
    tables: IndexMap<String, Arc<dyn TableSource>>,
    options: ConfigOptions,
}

impl ContextProvider for PoSqlContextProvider {
    fn get_table_source(
        &self,
        name: TableReference,
    ) -> Result<Arc<dyn TableSource>, DataFusionError> {
        let str_name = name.to_quoted_string();
        self.tables
            .get(&str_name)
            .cloned()
            .ok_or_else(|| DataFusionError::Plan(format!("Table {} not found", str_name)))
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

pub struct PoSqlTableSource {
    schema: SchemaRef,
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
        Ok(vec![
            TableProviderFilterPushDown::Unsupported;
            filters.len()
        ])
    }
}
