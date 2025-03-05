//! This crate converts a `DataFusion` `LogicalPlan` to a `ProofPlan` and `Postprocessing`
#![cfg_attr(not(test), expect(dead_code))] // TODO: remove this when initial development work is done
#![cfg_attr(test, expect(clippy::missing_panics_doc))]
extern crate alloc;
mod context;
pub use context::PoSqlContextProvider;
mod error;
pub use error::{PlannerError, PlannerResult};
mod util;
pub(crate) use util::{column_fields_to_schema, table_reference_to_table_ref};
