use crate::base::{
    bit::BitDistribution, commitment::Commitment, proof::ProofError, scalar::Scalar,
};
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

/// Given a bit distribution for a column of data with a constant sign, the commitment of a column
/// of ones, the constant column's commitment, and the commitment of varying absolute bits, verify
/// that the bit distribution is correct.
pub fn verify_constant_sign_decomposition<C: Commitment>(
    dist: &BitDistribution,
    commit: &C,
    one_commit: &C,
    bit_commits: &[C],
) -> Result<(), ProofError> {
    assert!(
        dist.is_valid()
            && is_within_acceptable_range(dist)
            && dist.num_varying_bits() == bit_commits.len()
            && !dist.has_varying_sign_bit()
    );
    let lhs = if dist.sign_bit() { -*commit } else { *commit };
    let mut rhs = C::Scalar::from(dist.constant_part()) * one_commit;
    let mut vary_index = 0;
    dist.for_each_abs_varying_bit(|int_index: usize, bit_index: usize| {
        let mut mult = [0u64; 4];
        mult[int_index] = 1u64 << bit_index;
        rhs += C::Scalar::from(mult) * bit_commits[vary_index];
        vary_index += 1;
    });
    if lhs == rhs {
        Ok(())
    } else {
        Err(ProofError::VerificationError(
            "constant sign bitwise decomposition is invalid",
        ))
    }
}

pub fn verify_constant_abs_decomposition<C: Commitment>(
    dist: &BitDistribution,
    commit: &C,
    one_commit: &C,
    sign_commit: &C,
) -> Result<(), ProofError> {
    assert!(
        dist.is_valid()
            && is_within_acceptable_range(dist)
            && dist.num_varying_bits() == 1
            && dist.has_varying_sign_bit()
    );
    let t = *one_commit - C::Scalar::TWO * sign_commit;
    if C::Scalar::from(dist.constant_part()) * t == *commit {
        Ok(())
    } else {
        Err(ProofError::VerificationError(
            "constant absolute bitwise decomposition is invalid",
        ))
    }
}
