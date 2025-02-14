//! This module contains the main logic for Proof of SQL.

/// This module holds the [`EVMProofPlan`] struct and its implementation, which allows for EVM compatible serialization.
pub mod evm_proof_plan;
pub mod new_parse;
pub mod parse;
pub mod postprocessing;
pub mod proof;
pub mod proof_exprs;
pub mod proof_gadgets;
pub mod proof_plans;
