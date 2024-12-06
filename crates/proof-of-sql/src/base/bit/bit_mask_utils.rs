use crate::base::scalar::Scalar;
use bnum::types::U256;

pub fn make_bit_mask<S: Scalar>(x: S) -> U256 {
    let x_as_limbs: [u64; 4] = x.into();
    let x_as_u256: U256 = x_as_limbs.into();
    if x > S::MAX_SIGNED {
        let max_signed_as_limbs: [u64; 4] = S::MAX_SIGNED.into();
        let max_signed_as_u256: U256 = max_signed_as_limbs.into();
        (x_as_u256 + U256::from([0u64, 0, 0, 1 << 63]) - U256::from(2u8) * max_signed_as_u256)
            .into()
    } else {
        (x_as_u256 + U256::from([0u64, 0, 0, 1 << 63])).into()
    }
}

pub fn is_bit_mask_negative_representation(bit_mask: U256) -> bool {
    bit_mask & (U256::from(1u8)) << 255 == U256::from(0u8)
}
