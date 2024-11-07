//! This module proves provable execution plans.
mod empty_exec;
pub use empty_exec::EmptyExec;

mod table_exec;
pub use table_exec::TableExec;

mod projection_exec;
pub(crate) use projection_exec::ProjectionExec;
#[cfg(all(test, feature = "blitzar"))]
mod projection_exec_test;

#[cfg(test)]
pub(crate) mod test_utility;

mod filter_exec;
pub(crate) use filter_exec::FilterExec;
#[cfg(test)]
pub(crate) use filter_exec::OstensibleFilterExec;
#[cfg(all(test, feature = "blitzar"))]
mod filter_exec_test;
#[cfg(all(test, feature = "blitzar"))]
mod filter_exec_test_dishonest_prover;

mod fold_util;
pub(crate) use fold_util::{fold_columns, fold_vals};
#[cfg(test)]
mod fold_util_test;

mod group_by_exec;
pub(crate) use group_by_exec::GroupByExec;

#[cfg(all(test, feature = "blitzar"))]
mod group_by_exec_test;

mod dyn_proof_plan;
pub use dyn_proof_plan::DynProofPlan;
