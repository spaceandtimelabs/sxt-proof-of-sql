//! This module contains shared proof logic for multiple `ProofExpr` / `ProofPlan` implementations.
mod membership_check;
mod shift;
mod uniqueness;
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
#[allow(clippy::needless_range_loop)] // keep the loop for readability for now, refactor later
pub(crate) mod range_check;
#[cfg(all(test, feature = "blitzar"))]
mod range_check_test;
#[cfg(all(test, feature = "blitzar"))]
mod sign_expr_test;
#[allow(unused_imports, dead_code)]
use uniqueness::{
    final_round_evaluate_uniqueness, first_round_evaluate_uniqueness, verify_uniqueness,
};
#[cfg(test)]
mod uniqueness_test;
