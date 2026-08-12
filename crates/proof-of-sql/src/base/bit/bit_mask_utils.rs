use crate::base::scalar::Scalar;
use bnum::types::U256;

/// Mask with only the most significant bit (255) set
/// Equivalent to `U256::ONE.shl(255)`
const MSB_MASK: U256 = U256::from_digits([0, 0, 0, 1 << 63]);

#[inline]
pub fn make_bit_mask<S: Scalar>(x: S) -> U256 {
    let x_as_u256 = x.into_u256_wrapping();
    if x > S::MAX_SIGNED {
        x_as_u256 - S::MAX_SIGNED_U256 + MSB_MASK - S::MAX_SIGNED_U256 - U256::ONE
    } else {
        x_as_u256 + MSB_MASK
    }
}

#[inline]
pub fn is_bit_mask_negative_representation(bit_mask: U256) -> bool {
    bit_mask & MSB_MASK == U256::ZERO
}
