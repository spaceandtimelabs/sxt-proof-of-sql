use crate::base::scalar::ScalarExt;
use bnum::types::U256;

pub fn make_bit_mask<S: ScalarExt>(x: S) -> U256 {
    let x_as_u256 = x.into_u256_wrapping();
    if x > S::MAX_SIGNED {
        x_as_u256 - S::into_u256_wrapping(S::MAX_SIGNED) + (U256::ONE << 255)
            - S::into_u256_wrapping(S::MAX_SIGNED)
            - U256::ONE
    } else {
        x_as_u256 + (U256::ONE << 255)
    }
}

pub fn is_bit_mask_negative_representation(bit_mask: U256) -> bool {
    bit_mask & (U256::ONE << 255) == U256::ZERO
}
