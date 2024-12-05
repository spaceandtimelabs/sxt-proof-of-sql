use crate::base::bit::BitDistribution;
/// In order to avoid cases with large numbers where there can be both a positive and negative
/// representation, we restrict the range of bit distributions that we accept.
///
/// Currently this is set to be the minimal value that will include the sum of two signed 128-bit
/// integers. The range will likely be expanded in the future as we support additional expressions.
pub fn is_within_acceptable_range(dist: &BitDistribution) -> bool {
    // signed 128 bit numbers range from
    //      -2^127 to 2^127-1
    // the maximum absolute value of the sum of two signed 128-integers is
    // then
    //       2 * (2^127) = 2^128
    dist.inverse_sign_mask[2] == u64::MAX && dist.inverse_sign_mask[3] == u64::MAX - (1 << 63)
}
