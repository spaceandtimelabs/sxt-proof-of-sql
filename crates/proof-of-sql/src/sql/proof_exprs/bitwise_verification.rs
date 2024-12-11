use crate::base::{bit::BitDistribution, proof::ProofError, scalar::Scalar};

#[allow(
    clippy::missing_panics_doc,
    reason = "All assertions check for validity within the context, ensuring no panic can occur"
)]
/// Given a bit distribution for a column of data with a constant sign, the evaluation of a column
/// of ones, the constant column's evaluation, and the evaluation of varying absolute bits, verify
/// that the bit distribution is correct.
pub fn verify_constant_sign_decomposition<S: Scalar>(
    dist: &BitDistribution,
    eval: S,
    one_eval: S,
    bit_evals: &[S],
) -> Result<(), ProofError> {
    assert!(
        dist.is_valid()
            && dist.is_within_acceptable_range()
            && dist.num_varying_bits() == bit_evals.len()
            && !dist.has_varying_sign_bit()
    );
    let lhs = if dist.sign_bit() { -eval } else { eval };
    let mut rhs = S::from(dist.constant_part()) * one_eval;
    let mut vary_index = 0;
    dist.for_each_abs_varying_bit(|int_index: usize, bit_index: usize| {
        let mut mult = [0u64; 4];
        mult[int_index] = 1u64 << bit_index;
        rhs += S::from(mult) * bit_evals[vary_index];
        vary_index += 1;
    });
    if lhs == rhs {
        Ok(())
    } else {
        Err(ProofError::VerificationError {
            error: "constant sign bitwise decomposition is invalid",
        })
    }
}

#[allow(
    clippy::missing_panics_doc,
    reason = "The assertion checks ensure that conditions are valid, preventing panics"
)]
pub fn verify_constant_abs_decomposition<S: Scalar>(
    dist: &BitDistribution,
    eval: S,
    one_eval: S,
    sign_eval: S,
) -> Result<(), ProofError> {
    assert!(
        dist.is_valid()
            && dist.is_within_acceptable_range()
            && dist.num_varying_bits() == 1
            && dist.has_varying_sign_bit()
    );
    let t = one_eval - S::TWO * sign_eval;
    if S::from(dist.constant_part()) * t == eval {
        Ok(())
    } else {
        Err(ProofError::VerificationError {
            error: "constant absolute bitwise decomposition is invalid",
        })
    }
}
