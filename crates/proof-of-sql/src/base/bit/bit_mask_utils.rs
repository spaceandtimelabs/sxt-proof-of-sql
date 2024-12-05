use crate::base::scalar::Scalar;
use bnum::types::U256;
use core::u64;

pub fn make_bit_mask<S: Scalar>(x: S) -> [u64; 4] {
    let x_as_limbs: [u64; 4] = x.into();
    let x_as_u256: U256 = x_as_limbs.into();
    if x > S::MAX_SIGNED {
        let max_signed_as_limbs: [u64; 4] = S::MAX_SIGNED.into();
        let max_signed_as_u256: U256 = max_signed_as_limbs.into();
        (x_as_u256 + U256::from([0u64, 0, 0, 1 << 63]) - max_signed_as_u256 - max_signed_as_u256)
            .into()
    } else {
        (x_as_u256 + U256::from([0u64, 0, 0, 1 << 63])).into()
    }
}

pub fn is_bit_mask_negative_representation(bit_mask: [u64; 4]) -> bool {
    bit_mask[3] & 1 << 63 == 0
}

pub(super) fn make_sign_bit_mask<S: Scalar>(x: S) -> [u64; 4] {
    let bit_mask = make_bit_mask(x);
    if is_bit_mask_negative_representation(bit_mask) {
        bit_mask.map(|u| u ^ u64::MAX)
    } else {
        bit_mask
    }
}
