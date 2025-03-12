//! This crate converts a `DataFusion` `LogicalPlan` to a `ProofPlan` and `Postprocessing`
#![cfg_attr(not(test), expect(dead_code))] // TODO: remove this when initial development work is done
#![cfg_attr(test, expect(clippy::missing_panics_doc))]
extern crate alloc;
mod aggregate;
mod context;
pub use context::PoSqlContextProvider;
#[cfg(test)]
pub(crate) use context::PoSqlTableSource;
mod conversion;
pub use conversion::sql_to_proof_plans;
#[cfg(test)]
mod df_util;
mod expr;
pub use expr::expr_to_proof_expr;
mod error;
pub use error::{PlannerError, PlannerResult};
mod plan;
pub use plan::logical_plan_to_proof_plan;
mod util;
pub use util::column_fields_to_schema;
pub(crate) use util::{
    column_to_column_ref, df_schema_to_column_fields, scalar_value_to_literal_value,
    table_reference_to_table_ref,
};
