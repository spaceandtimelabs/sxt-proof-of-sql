//! This crate converts a `DataFusion` `LogicalPlan` to a `ProofPlan` and `Postprocessing`
extern crate alloc;
mod context;
pub use context::PoSqlContextProvider;
mod error;
pub use error::{PlannerError, PlannerResult};
mod util;
pub(crate) use util::{column_fields_to_schema, table_reference_to_table_ref};
