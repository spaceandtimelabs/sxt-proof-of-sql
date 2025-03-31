//! This module contains shared proof logic for multiple `ProofExpr` / `ProofPlan` implementations.
#[cfg(test)]
mod divide_and_modulo_expr;
mod membership_check;
mod monotonic;
#[allow(dead_code)]
mod permutation_check;
mod shift;
pub(crate) use membership_check::{
    final_round_evaluate_membership_check, first_round_evaluate_membership_check,
    verify_membership_check,
};
#[cfg(test)]
mod membership_check_test;
#[expect(unused_imports)]
use permutation_check::{final_round_evaluate_permutation_check, verify_permutation_check};
#[cfg(test)]
mod permutation_check_test;
use shift::{final_round_evaluate_shift, first_round_evaluate_shift, verify_shift};
#[cfg(test)]
mod shift_test;
mod sign_expr;
pub(crate) use sign_expr::{
    final_round_evaluate_sign, first_round_evaluate_sign, verifier_evaluate_sign,
};
#[cfg(feature = "blitzar")]
#[allow(dead_code)]
mod range_check;
#[cfg(all(test, feature = "blitzar"))]
mod range_check_test;
#[cfg(all(test, feature = "blitzar"))]
mod sign_expr_test;
pub(crate) use monotonic::{
    final_round_evaluate_monotonic, first_round_evaluate_monotonic, verify_monotonic,
};
#[cfg(test)]
mod monotonic_test;
