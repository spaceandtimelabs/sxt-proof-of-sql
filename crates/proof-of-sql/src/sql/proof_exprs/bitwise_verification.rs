use crate::base::{bit::BitDistribution, proof::ProofError, scalar::Scalar};
/// In order to avoid cases with large numbers where there can be both a positive and negative
/// representation, we restrict the range of bit distributions that we accept.
///
/// Currently this is set to be the minimal value that will include the sum of two signed 128-bit
/// integers. The range will likely be expanded in the future as we support additional expressions.
pub fn is_within_acceptable_range(dist: &BitDistribution) -> bool {
    // handle the case of everything zero
    if dist.num_varying_bits() == 0 && dist.constant_part() == [0; 4] {
        return true;
    }

    // signed 128 bit numbers range from
    //      -2^127 to 2^127-1
    // the maximum absolute value of the sum of two signed 128-integers is
    // then
    //       2 * (2^127) = 2^128
    dist.most_significant_abs_bit() <= 128
}

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
            && is_within_acceptable_range(dist)
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
            && is_within_acceptable_range(dist)
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
