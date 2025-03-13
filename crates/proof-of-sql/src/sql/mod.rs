//! This module contains the main logic for Proof of SQL.

mod error;
/// This module holds the [`EVMProofPlan`] struct and its implementation, which allows for EVM compatible serialization.
pub mod evm_proof_plan;
pub mod parse;
/// This temporarily exists until we switch to using Datafusion Analyzer to handle type checking.
pub use error::{AnalyzeError, AnalyzeResult};
pub mod postprocessing;
pub mod proof;
pub mod proof_exprs;
pub mod proof_gadgets;
pub mod proof_plans;
