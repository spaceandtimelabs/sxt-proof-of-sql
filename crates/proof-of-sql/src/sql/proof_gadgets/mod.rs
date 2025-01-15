//! This module contains shared proof logic for multiple `ProofExpr` / `ProofPlan` implementations.
mod bitwise_verification;
use bitwise_verification::{verify_constant_abs_decomposition, verify_constant_sign_decomposition};
#[cfg(test)]
mod bitwise_verification_test;
mod membership_check;
#[allow(unused_imports, dead_code)]
use membership_check::{
    final_round_evaluate_membership_check, first_round_evaluate_membership_check,
    verify_membership_check,
};
#[cfg(test)]
mod membership_check_test;
mod sign_expr;
pub(crate) use sign_expr::{prover_evaluate_sign, result_evaluate_sign, verifier_evaluate_sign};
#[allow(clippy::needless_range_loop)] // keep the loop for readability for now, refactor later
#[allow(dead_code, missing_docs)]
pub(crate) mod range_check;
#[cfg(all(feature = "blitzar"))]
#[allow(dead_code, missing_docs)]
pub mod range_check_test;
#[cfg(all(test, feature = "blitzar"))]
mod sign_expr_test;
