use crate::base::bit::BitDistribution;
use crate::base::proof::ProofError;
use curve25519_dalek::ristretto::RistrettoPoint;

/// In order to avoid cases with large numbers where there can be both a positive and negative
/// representation, we restrict the range of bit distributions that we accept.
///
/// Currently this is set to be the minimal value that will include the sum of two signed 64-bit
/// integers. The range will likely be expanded in the future as we support additional expressions.
pub fn is_within_acceptable_range(dist: &BitDistribution) -> bool {
    // signed 64 bit numbers range from
    //      -2^63 to 2^63-1
    // the maximum absolute value of the sum of two signed 64-integers is
    // then
    //       2 * (2^63) = 2^64
    dist.most_significant_abs_bit() <= 64
}

/// Given a bit distribution for a column of constant data, the commitment of a column of ones,
/// and the constant column's commitment, verify that the bit distribution is correct.
pub fn verify_constant_decomposition(
    dist: &BitDistribution,
    commit: &RistrettoPoint,
    one_commit: &RistrettoPoint,
) -> Result<(), ProofError> {
    assert!(dist.is_valid());
    assert!(is_within_acceptable_range(dist));
    assert_eq!(dist.num_varying_bits(), 0);
    let equal = if dist.sign_bit() {
        *commit == (-dist.constant_part()) * *one_commit
    } else {
        *commit == dist.constant_part() * *one_commit
    };
    if equal {
        Ok(())
    } else {
        Err(ProofError::VerificationError(
            "bitwise decomposition is invalid",
        ))
    }
}
