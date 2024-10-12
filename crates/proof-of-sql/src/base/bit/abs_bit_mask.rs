use crate::base::scalar::Scalar;

pub fn make_abs_bit_mask<S: Scalar>(x: S) -> [u64; 4] {
    let (sign, x) = if S::MAX_SIGNED < x { (1, -x) } else { (0, x) };
    let mut res: [u64; 4] = x.into();
    res[3] |= sign << 63;
    res
}
