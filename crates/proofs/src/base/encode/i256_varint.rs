use super::{VarInt, U256};
use arrow::datatypes::i256;

// Adapted from integer-encoding-rs. See third_party/license/integer-encoding.LICENSE
fn zigzag_encode_i256(from: i256) -> U256 {
    let (low, high) = ((from << 1) ^ (from >> 255)).to_parts();
    U256 {
        low,
        high: high as u128,
    }
}
// Adapted from integer-encoding-rs. See third_party/license/integer-encoding.LICENSE
// see: http://stackoverflow.com/a/2211086/56332
fn zigzag_decode_i256(from: U256) -> i256 {
    let from = i256::from_parts(from.low, from.high as i128);
    (from >> 1) ^ (-(from & i256::ONE))
}
impl VarInt for i256 {
    fn required_space(self) -> usize {
        U256::required_space(zigzag_encode_i256(self))
    }
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        U256::decode_var(src).map(|(v, s)| (zigzag_decode_i256(v), s))
    }
    fn encode_var(self, dst: &mut [u8]) -> usize {
        zigzag_encode_i256(self).encode_var(dst)
    }
}

#[cfg(test)]
mod test {
    use crate::base::{
        encode::varint_trait_test::{
            test_encode_and_decode_types_align, test_small_signed_values_encode_and_decode_properly,
        },
        scalar::{ArkScalar, MontScalar},
    };
    use arrow::datatypes::i256;
    use core::iter::repeat_with;
    use rand::Rng;

    #[test]
    fn we_can_encode_and_decode_small_i256_values() {
        test_small_signed_values_encode_and_decode_properly::<i256>(i256::ONE);
    }

    #[test]
    fn we_can_encode_and_decode_i64_and_i256_the_same() {
        let mut rng = rand::thread_rng();
        test_encode_and_decode_types_align::<i64, i256>(
            &rng.gen::<[_; 32]>(),
            &[
                i256::from(i64::MAX) + i256::from(1),
                i256::from(i64::MIN) - i256::from(1),
                i256::from(i64::MAX) * i256::from(1000),
                i256::from(i64::MIN) * i256::from(1000),
            ],
            100,
        );
    }

    #[test]
    fn we_can_encode_and_decode_ark_scalar_and_i256_the_same() {
        let mut rng = ark_std::test_rng();
        test_encode_and_decode_types_align::<ArkScalar, i256>(
            &Vec::from_iter(repeat_with(|| MontScalar(rng.gen())).take(32)),
            &[],
            100,
        );
    }
}
