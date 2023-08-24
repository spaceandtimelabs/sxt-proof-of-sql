use super::{is_within_acceptable_range, verify_constant_decomposition};
use crate::base::bit::BitDistribution;

use crate::base::proof::ProofError;
use crate::base::scalar::ArkScalar;
use crate::sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder};

use bumpalo::Bump;

use curve25519_dalek::ristretto::RistrettoPoint;
use num_traits::Zero;

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
    let bit_distribution = BitDistribution::new(expr);
    builder.produce_bit_distribution(bit_distribution.clone());

    // handle the constant case
    if bit_distribution.num_varying_bits() == 0 {
        return alloc.alloc_slice_fill_copy(table_length, bit_distribution.sign_bit());
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
    let bit_distribution = builder.consume_bit_distribution();

    // handle constant case
    if bit_distribution.num_varying_bits() == 0 {
        return verifier_const_evaluate(builder, &bit_distribution, commit, one_commit);
    }

    todo!();
}

fn verifier_const_evaluate(
    builder: &VerificationBuilder,
    dist: &BitDistribution,
    commit: &RistrettoPoint,
    one_commit: &RistrettoPoint,
) -> Result<ArkScalar, ProofError> {
    verify_constant_decomposition(dist, commit, one_commit)?;
    if dist.sign_bit() {
        Ok(builder.mle_evaluations.one_evaluation)
    } else {
        Ok(ArkScalar::zero())
    }
}
