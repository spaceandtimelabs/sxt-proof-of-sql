use super::{is_within_acceptable_range, verify_constant_sign_decomposition};
use crate::base::bit::{compute_varying_bit_matrix, BitDistribution};

use crate::base::proof::ProofError;
use crate::base::scalar::ArkScalar;
use crate::sql::proof::{
    CountBuilder, MultilinearExtensionImpl, ProofBuilder, SumcheckSubpolynomial,
    VerificationBuilder,
};

use bumpalo::Bump;

use curve25519_dalek::ristretto::RistrettoPoint;
use num_traits::{One, Zero};

/// Count the number of components needed to prove a sign decomposition
pub fn count_sign(builder: &mut CountBuilder) -> Result<(), ProofError> {
    let dist = builder.consume_bit_distribution()?;
    if !is_within_acceptable_range(&dist) {
        return Err(ProofError::VerificationError(
            "bit distribution outside of acceptable range",
        ));
    }
    if dist.num_varying_bits() == 0 {
        return Ok(());
    }
    builder.count_intermediate_mles(dist.num_varying_bits());
    builder.count_subpolynomials(dist.num_varying_bits());
    builder.count_degree(3);
    if !dist.has_varying_sign_bit() {
        return Ok(());
    }
    todo!();
}

/// Prove the sign decomposition for a column of scalars.
///
/// If x1, ..., xn denotes the data, prove the column of
/// booleans, i.e. sign bits, s1, ..., sn where si == 1 if xi > MID and
/// si == 1 if xi <= MID and MID is defined in base/bit/abs_bit_mask.rs
///
/// Note: We can only prove the sign bit for non-zero scalars, and we restict
/// the range of non-zero scalar so that there is a unique sign representation.
pub fn prover_evaluate_sign<'a>(
    builder: &mut ProofBuilder<'a>,
    alloc: &'a Bump,
    expr: &'a [ArkScalar],
) -> &'a [bool] {
    let table_length = expr.len();
    // bit_distribution
    let dist = BitDistribution::new(expr);
    builder.produce_bit_distribution(dist.clone());

    // handle the constant case
    if dist.num_varying_bits() == 0 {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    // prove that the bits are binary
    let bits = compute_varying_bit_matrix(alloc, expr, &dist);
    prove_bits_are_binary(builder, &bits);
    if !dist.has_varying_sign_bit() {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    todo!();
}

/// Verify the sign decomposition for a column of scalars.
///
/// See prover_evaluate_sign.
pub fn verifier_evaluate_sign(
    builder: &mut VerificationBuilder,
    commit: &RistrettoPoint,
    one_commit: &RistrettoPoint,
) -> Result<ArkScalar, ProofError> {
    // bit_distribution
    let dist = builder.consume_bit_distribution();
    let num_varying_bits = dist.num_varying_bits();

    // extract evaluations and commitmens of the multilinear extensions for the varying
    // bits of the expression
    let mut bit_evals = Vec::with_capacity(num_varying_bits);
    let mut bit_commits = Vec::with_capacity(num_varying_bits);
    for _ in 0..num_varying_bits {
        let (eval, commit) = builder.consume_intermediate_mle_with_commit();
        bit_evals.push(eval);
        bit_commits.push(commit);
    }

    // establish that the bits are binary
    verify_bits_are_binary(builder, &bit_evals);

    if !dist.has_varying_sign_bit() {
        return verifier_const_sign_evaluate(builder, &dist, commit, one_commit, &bit_commits);
    }

    todo!();
}

fn verifier_const_sign_evaluate(
    builder: &VerificationBuilder,
    dist: &BitDistribution,
    commit: &RistrettoPoint,
    one_commit: &RistrettoPoint,
    bit_commits: &[RistrettoPoint],
) -> Result<ArkScalar, ProofError> {
    verify_constant_sign_decomposition(dist, commit, one_commit, bit_commits)?;
    if dist.sign_bit() {
        Ok(builder.mle_evaluations.one_evaluation)
    } else {
        Ok(ArkScalar::zero())
    }
}

fn prove_bits_are_binary<'a>(builder: &mut ProofBuilder<'a>, bits: &[&'a [bool]]) {
    for seq in bits.iter() {
        builder.produce_intermediate_mle(seq);
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                ArkScalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(seq))],
            ),
            (
                -ArkScalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(seq)),
                    Box::new(MultilinearExtensionImpl::new(seq)),
                ],
            ),
        ]));
    }
}

fn verify_bits_are_binary(builder: &mut VerificationBuilder, bit_evals: &[ArkScalar]) {
    for bit_eval in bit_evals.iter() {
        let mut eval = *bit_eval - *bit_eval * *bit_eval;
        eval *= builder.mle_evaluations.random_evaluation;
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
}
