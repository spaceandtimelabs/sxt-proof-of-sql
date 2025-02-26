//! This crate converts a DataFusion `LogicalPlan` to a `ProofPlan` and `Postprocessing`
extern crate alloc;
mod context;
pub use context::{PoSqlContextProvider, PoSqlTableSource};
mod conversion;
mod error;
pub use error::{PlannerError, PlannerResult};
mod util;
pub(crate) use util::{
    column_as_column_ref, scalar_value_as_literal_value, table_reference_as_table_ref,
};
