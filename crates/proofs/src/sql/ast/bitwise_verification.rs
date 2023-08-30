use crate::base::bit::BitDistribution;
use crate::base::proof::ProofError;
use crate::base::scalar::ArkScalar;
use curve25519_dalek::ristretto::RistrettoPoint;
use num_traits::Zero;

/// In order to avoid cases with large numbers where there can be both a positive and negative
/// representation, we restrict the range of bit distributions that we accept.
///
/// Currently this is set to be the minimal value that will include the sum of two signed 64-bit
/// integers. The range will likely be expanded in the future as we support additional expressions.
pub fn is_within_acceptable_range(dist: &BitDistribution) -> bool {
    // handle the case of everything zero
    if dist.num_varying_bits() == 0 && dist.constant_part() == ArkScalar::zero() {
        return true;
    }

    // signed 64 bit numbers range from
    //      -2^63 to 2^63-1
    // the maximum absolute value of the sum of two signed 64-integers is
    // then
    //       2 * (2^63) = 2^64
    dist.most_significant_abs_bit() <= 64
}

/// Given a bit distribution for a column of data with a constant sign, the commitment of a column
/// of ones, the constant column's commitment, and the commitment of varying absolute bits, verify
/// that the bit distribution is correct.
pub fn verify_constant_sign_decomposition(
    dist: &BitDistribution,
    commit: &RistrettoPoint,
    one_commit: &RistrettoPoint,
    bit_commits: &[RistrettoPoint],
) -> Result<(), ProofError> {
    assert!(
        dist.is_valid()
            && is_within_acceptable_range(dist)
            && dist.num_varying_bits() == bit_commits.len()
            && !dist.has_varying_sign_bit()
    );
    let lhs = if dist.sign_bit() { -commit } else { *commit };
    let mut rhs = dist.constant_part() * one_commit;
    let mut vary_index = 0;
    dist.for_each_abs_varying_bit(|int_index: usize, bit_index: usize| {
        let mut mult = [0u64; 4];
        mult[int_index] = 1u64 << bit_index;
        rhs += ArkScalar::from_bigint(mult) * bit_commits[vary_index];
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

pub fn verify_constant_abs_decomposition(
    dist: &BitDistribution,
    commit: &RistrettoPoint,
    one_commit: &RistrettoPoint,
    sign_commit: &RistrettoPoint,
) -> Result<(), ProofError> {
    assert!(
        dist.is_valid()
            && is_within_acceptable_range(dist)
            && dist.num_varying_bits() == 1
            && dist.has_varying_sign_bit()
    );
    let t = one_commit - ArkScalar::from(2) * sign_commit;
    if dist.constant_part() * t == *commit {
        Ok(())
    } else {
        Err(ProofError::VerificationError(
            "constant absolute bitwise decomposition is invalid",
        ))
    }
}
