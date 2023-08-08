use crate::base::scalar::ArkScalar;
use lazy_static::lazy_static;
use num_traits::{Inv, One};
use std::convert::Into;

pub fn make_abs_bit_mask(x: ArkScalar) -> [u64; 4] {
    lazy_static! {
        static ref MID: ArkScalar = -ArkScalar::one() * ArkScalar::from(2).inv();
    }
    let (sign, x) = if MID.0 < x.0 { (1, -x) } else { (0, x) };
    let mut res: [u64; 4] = x.into();
    res[3] |= sign << 63;
    res
}
