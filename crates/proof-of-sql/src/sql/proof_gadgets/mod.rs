//! This module contains shared proof logic for multiple `ProofExpr` / `ProofPlan` implementations.
mod membership_check;
mod monotonic;
mod shift;
#[allow(unused_imports, dead_code)]
use membership_check::{
    final_round_evaluate_membership_check, first_round_evaluate_membership_check,
    verify_membership_check,
};
#[cfg(test)]
mod membership_check_test;
use shift::{final_round_evaluate_shift, first_round_evaluate_shift, verify_shift};
#[cfg(test)]
mod shift_test;
mod sign_expr;
pub(crate) use sign_expr::{prover_evaluate_sign, result_evaluate_sign, verifier_evaluate_sign};
#[allow(clippy::non_minimal_cfg)] // need to add test feature back in at some point
#[cfg(feature = "blitzar")]
#[allow(unused_imports)]
pub mod range_check;
#[allow(clippy::non_minimal_cfg)]
#[cfg(all(feature = "blitzar"))]
#[allow(missing_docs)]
pub mod range_check_test;
#[cfg(all(test, feature = "blitzar"))]
mod sign_expr_test;
#[allow(unused_imports, dead_code)]
use monotonic::{final_round_evaluate_monotonic, first_round_evaluate_monotonic, verify_monotonic};
#[cfg(test)]
mod monotonic_test;
