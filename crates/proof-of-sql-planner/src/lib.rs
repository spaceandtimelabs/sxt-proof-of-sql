//! This crate converts a `DataFusion` `LogicalPlan` to a `ProofPlan`
#![cfg_attr(test, expect(clippy::missing_panics_doc))]
extern crate alloc;
mod aggregate;
pub(crate) use aggregate::{aggregate_function_to_proof_expr, AggregateFunc};
pub(crate) mod config;
pub use config::datafusion_config_no_normalization;
mod context;
pub use context::PoSqlContextProvider;
#[cfg(test)]
pub(crate) use context::PoSqlTableSource;
mod conversion;
pub use conversion::{get_table_refs_from_statement, sql_to_proof_plans};
#[cfg(test)]
mod df_util;
mod expr;
pub use expr::expr_to_proof_expr;
pub(crate) use expr::get_column_idents_from_expr;
mod error;
pub use error::{
    AggregatePlanError, JoinPlanError, LogicalPlanNodeKind, PlannerError, PlannerResult,
};
mod plan;
pub use plan::logical_plan_to_proof_plan;
mod uppercase_column_visitor;
pub use uppercase_column_visitor::{statement_with_uppercase_identifiers, uppercase_identifier};
mod util;
pub use util::column_fields_to_schema;
pub(crate) use util::{
    column_to_column_ref, placeholder_to_placeholder_expr, scalar_value_to_literal_value,
    schema_to_column_fields, table_reference_to_table_ref,
};
