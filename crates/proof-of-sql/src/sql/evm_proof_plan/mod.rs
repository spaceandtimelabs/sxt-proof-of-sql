mod error;
pub(crate) use error::{EVMProofPlanError, EVMProofPlanResult};
mod exprs;
pub(crate) use exprs::EVMDynProofExpr;
mod plans;
mod proof_plan;
#[cfg(test)]
mod tests;

pub use proof_plan::EVMProofPlan;

#[cfg(all(test, feature = "hyperkzg_proof"))]
mod evm_tests;
