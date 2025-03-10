use crate::base::scalar::Scalar;
use ark_ff::BigInteger;
use core::ops::Neg;
use serde::{Deserialize, Serialize};

/// A 256-bit data type with some conversions implemented that interpret it as a signed integer.
///
/// This should only implement conversions. If anything else is needed, we should strongly consider an alternative design.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct I256([u64; 4]);

impl Neg for I256 {
    type Output = Self;
    /// Computes the wrapping negative of the value. This could perhaps be more efficient.
    fn neg(self) -> Self::Output {
        let mut res = ark_ff::BigInt([0; 4]);
        res.sub_with_borrow(&ark_ff::BigInt(self.0));
        Self(res.0)
    }
}

impl I256 {
    /// Make an `I256` from its limbs.
    #[must_use]
    pub fn new(limbs: [u64; 4]) -> Self {
        Self(limbs)
    }
    #[must_use]
    /// Conversion into a [Scalar] type. The conversion handles negative values. In other words, `-1` maps to `-S::ONE`.
    ///
    /// NOTE: the behavior of this is undefined if the absolute value is larger than the modulus.
    ///
    /// NOTE: this is not a particularly efficient method. Please either refactor or avoid when performance matters.
    pub fn into_scalar<S: Scalar>(self) -> S {
        if self.0[3] & 0x8000_0000_0000_0000 == 0 {
            self.0.into()
        } else {
            (Into::<S>::into(self.neg().0)).neg()
        }
    }

    #[must_use]
    /// Conversion from a [`num_bigint::BigInt`].
    /// The conversion handles negative values and also wraps when the value is too large for an `I256`.
    ///
    /// NOTE: this is not a particularly efficient method. Please either refactor or avoid when performance matters.
    pub fn from_num_bigint(value: &num_bigint::BigInt) -> Self {
        let (sign, limbs_vec) = value.to_u64_digits();
        let num_limbs = limbs_vec.len().min(4);
        let mut limbs = [0u64; 4];
        limbs[..num_limbs].copy_from_slice(&limbs_vec[..num_limbs]);
        limbs[3] &= 0x7FFF_FFFF_FFFF_FFFF;
        match sign {
            num_bigint::Sign::Minus => Self(limbs).neg(),
            num_bigint::Sign::Plus | num_bigint::Sign::NoSign => Self(limbs),
        }
    }
}
impl From<i32> for I256 {
    fn from(value: i32) -> Self {
        let abs = Self([value.unsigned_abs().into(), 0, 0, 0]);
        if value >= 0 {
            abs
        } else {
            abs.neg()
        }
    }
}

#[expect(clippy::cast_possible_truncation)]
impl From<i128> for I256 {
    fn from(value: i128) -> Self {
        let abs_u128 = value.unsigned_abs();
        let low = abs_u128 as u64;
        let high = (abs_u128 >> 64) as u64;
        let abs = Self([low, high, 0, 0]);
        if value >= 0 {
            abs
        } else {
            abs.neg()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::{test_scalar::TestScalar, MontScalar, Scalar};
    use ark_ff::MontFp;
    use num_bigint::BigInt;
    use rand::{thread_rng, Rng};

    const ZERO: I256 = I256([0, 0, 0, 0]);
    const ONE: I256 = I256([1, 0, 0, 0]);
    const TWO: I256 = I256([2, 0, 0, 0]);
    const NEG_ONE: I256 = I256([
        0xFFFF_FFFF_FFFF_FFFF,
        0xFFFF_FFFF_FFFF_FFFF,
        0xFFFF_FFFF_FFFF_FFFF,
        0xFFFF_FFFF_FFFF_FFFF,
    ]);
    const NEG_TWO: I256 = I256([
        0xFFFF_FFFF_FFFF_FFFE,
        0xFFFF_FFFF_FFFF_FFFF,
        0xFFFF_FFFF_FFFF_FFFF,
        0xFFFF_FFFF_FFFF_FFFF,
    ]);
    const A_STR: &str =
        "57896044618658097705508390768957273162799202909612615603626436559492530307074";
    const A: I256 = I256([2, 0, 0, 0x7FFF_FFFF_FFFF_FFFF]);
    const NEG_A: I256 = I256([
        0xFFFF_FFFF_FFFF_FFFE,
        0xFFFF_FFFF_FFFF_FFFF,
        0xFFFF_FFFF_FFFF_FFFF,
        0x8000_0000_0000_0000,
    ]);
    const B_STR: &str =
        "44514458406356786875149426309623179975904669798901350226660343085647800511238";
    const B: I256 = I256([
        0x12DE_4D02_71BF_2B06,
        0x2686_80A2_B415_EE31,
        0xBCF3_35CF_A69C_DBE3,
        0x626A_4A65_275E_1D88,
    ]);
    const NEG_B: I256 = I256([
        0xED21_B2FD_8E40_D4FA,
        0xD979_7F5D_4BEA_11CE,
        0x430C_CA30_5963_241C,
        0x9D95_B59A_D8A1_E277,
    ]);
    const C_STR: &str =
        "452312848583266388373324160190187140051835877600158453279131187530910662656";
    const C_SCALAR: TestScalar = MontScalar(MontFp!(
        "452312848583266388373324160190187140051835877600158453279131187530910662656"
    ));
    const C: I256 = I256([
        0x0000_0000_0000_0000,
        0x0000_0000_0000_0000,
        0x0000_0000_0000_0000,
        0x0100_0000_0000_0000,
    ]);
    const NEG_C: I256 = I256([
        0x0000_0000_0000_0000,
        0x0000_0000_0000_0000,
        0x0000_0000_0000_0000,
        0xFF00_0000_0000_0000,
    ]);
    const MODULUS_MINUS_ONE: I256 = I256([
        0x5812_631A_5CF5_D3EC,
        0x14DE_F9DE_A2F7_9CD6,
        0x0000_0000_0000_0000,
        0x1000_0000_0000_0000,
    ]);
    const NEG_MODULUS_PLUS_ONE: I256 = I256([
        0xa7ed_9ce5_a30a_2c14,
        0xEB21_0621_5D08_6329,
        0xFFFF_FFFF_FFFF_FFFF,
        0xEFFF_FFFF_FFFF_FFFF,
    ]);
    const MODULUS_MINUS_TWO: I256 = I256([
        0x5812_631A_5CF5_D3EB,
        0x14DE_F9DE_A2F7_9CD6,
        0x0000_0000_0000_0000,
        0x1000_0000_0000_0000,
    ]);
    const NEG_MODULUS_PLUS_TWO: I256 = I256([
        0xa7ed_9ce5_a30a_2c15,
        0xEB21_0621_5D08_6329,
        0xFFFF_FFFF_FFFF_FFFF,
        0xEFFF_FFFF_FFFF_FFFF,
    ]);

    #[test]
    fn we_can_compute_the_negative_of_i256() {
        assert_eq!(ZERO.neg(), ZERO);
        assert_eq!(ONE.neg(), NEG_ONE);
        assert_eq!(NEG_ONE.neg(), ONE);
        assert_eq!(TWO.neg(), NEG_TWO);
        assert_eq!(NEG_TWO.neg(), TWO);
        assert_eq!(A.neg(), NEG_A);
        assert_eq!(NEG_A.neg(), A);
        assert_eq!(B.neg(), NEG_B);
        assert_eq!(NEG_B.neg(), B);
        assert_eq!(C.neg(), NEG_C);
        assert_eq!(NEG_C.neg(), C);
        assert_eq!(MODULUS_MINUS_ONE.neg(), NEG_MODULUS_PLUS_ONE);
        assert_eq!(NEG_MODULUS_PLUS_ONE.neg(), MODULUS_MINUS_ONE);
        assert_eq!(MODULUS_MINUS_TWO.neg(), NEG_MODULUS_PLUS_TWO);
        assert_eq!(NEG_MODULUS_PLUS_TWO.neg(), MODULUS_MINUS_TWO);

        let mut rng = thread_rng();
        for _ in 0..10 {
            let x = I256([rng.gen(), rng.gen(), rng.gen(), rng.gen()]);
            assert_eq!(x.neg().neg(), x);
        }
    }
    #[test]
    fn we_can_convert_i256_into_scalar() {
        assert_eq!(ZERO.into_scalar::<TestScalar>(), TestScalar::ZERO);
        assert_eq!(ONE.into_scalar::<TestScalar>(), TestScalar::ONE);
        assert_eq!(NEG_ONE.into_scalar::<TestScalar>(), -TestScalar::ONE);
        assert_eq!(TWO.into_scalar::<TestScalar>(), TestScalar::TWO);
        assert_eq!(NEG_TWO.into_scalar::<TestScalar>(), -TestScalar::TWO);
        assert_eq!(C.into_scalar::<TestScalar>(), C_SCALAR);
        assert_eq!(NEG_C.into_scalar::<TestScalar>(), -C_SCALAR);
        assert_eq!(
            MODULUS_MINUS_ONE.into_scalar::<TestScalar>(),
            -TestScalar::ONE
        );
        assert_eq!(
            NEG_MODULUS_PLUS_ONE.into_scalar::<TestScalar>(),
            TestScalar::ONE
        );
        assert_eq!(
            MODULUS_MINUS_TWO.into_scalar::<TestScalar>(),
            -TestScalar::TWO
        );
        assert_eq!(
            NEG_MODULUS_PLUS_TWO.into_scalar::<TestScalar>(),
            TestScalar::TWO
        );

        let mut rng = thread_rng();
        for _ in 0..10 {
            let x = I256([rng.gen(), rng.gen(), rng.gen(), rng.gen()]);
            assert_eq!(
                x.neg().into_scalar::<TestScalar>(),
                -(x.into_scalar::<TestScalar>())
            );
        }
    }
    #[test]
    fn we_can_convert_i256_from_num_bigint() {
        assert_eq!(I256::from_num_bigint(&"0".parse().unwrap()), ZERO);
        assert_eq!(I256::from_num_bigint(&"1".parse().unwrap()), ONE);
        assert_eq!(I256::from_num_bigint(&"-1".parse().unwrap()), NEG_ONE);
        assert_eq!(I256::from_num_bigint(&"2".parse().unwrap()), TWO);
        assert_eq!(I256::from_num_bigint(&"-2".parse().unwrap()), NEG_TWO);
        assert_eq!(I256::from_num_bigint(&A_STR.parse().unwrap()), A);
        assert_eq!(
            I256::from_num_bigint(&-A_STR.parse::<BigInt>().unwrap()),
            NEG_A
        );
        assert_eq!(I256::from_num_bigint(&B_STR.parse().unwrap()), B);
        assert_eq!(
            I256::from_num_bigint(&-B_STR.parse::<BigInt>().unwrap()),
            NEG_B
        );
        assert_eq!(I256::from_num_bigint(&C_STR.parse().unwrap()), C);
        assert_eq!(
            I256::from_num_bigint(&-C_STR.parse::<BigInt>().unwrap()),
            NEG_C
        );

        let mut rng = thread_rng();
        for _ in 0..10 {
            let x =
                (BigInt::from(rng.gen::<i128>().abs()) << 128) + BigInt::from(rng.gen::<u128>());
            let y = &x + (BigInt::from(rng.gen::<u128>()) << 255);
            assert_eq!(I256::from_num_bigint(&y), I256::from_num_bigint(&x));
            assert_eq!(I256::from_num_bigint(&-&y), I256::from_num_bigint(&-x));
            assert_eq!(I256::from_num_bigint(&y), I256::from_num_bigint(&-y).neg());
        }
    }
    #[test]
    fn we_can_convert_i256_from_i32() {
        assert_eq!(I256::from(0), ZERO);
        assert_eq!(I256::from(1), ONE);
        assert_eq!(I256::from(-1), NEG_ONE);
        assert_eq!(I256::from(2), TWO);
        assert_eq!(I256::from(-2), NEG_TWO);
    }
    #[test]
    fn we_can_convert_i256_between_type_compatibly() {
        let mut rng = thread_rng();
        for _ in 0..10 {
            let int32: i32 = rng.gen();
            let neg_int32 = -int32;
            let scalar = TestScalar::from(int32);
            let neg_scalar = -scalar;
            let bigint = BigInt::from(int32);
            let neg_bigint = -&bigint;
            let int256_from_i32 = I256::from(int32);
            let neg_int256_from_i32 = I256::from(neg_int32);
            let int256_from_bigint = I256::from_num_bigint(&bigint);
            let neg_int256_from_bigint = I256::from_num_bigint(&neg_bigint);
            assert_eq!(int256_from_i32, int256_from_bigint);
            assert_eq!(neg_int256_from_i32, neg_int256_from_bigint);
            assert_eq!(neg_int256_from_i32, int256_from_i32.neg());
            assert_eq!(int256_from_i32.into_scalar::<TestScalar>(), scalar);
            assert_eq!(neg_int256_from_i32.into_scalar::<TestScalar>(), neg_scalar);
        }
    }

    #[expect(clippy::cast_sign_loss)]
    #[test]
    fn test_i128_to_i256_conversion() {
        // Test zero
        assert_eq!(I256::from(0i128), ZERO);

        // Test positive values
        assert_eq!(I256::from(1i128), ONE);
        assert_eq!(I256::from(2i128), TWO);

        // Test negative values
        assert_eq!(I256::from(-1i128), NEG_ONE);
        assert_eq!(I256::from(-2i128), NEG_TWO);

        // Test boundary values
        assert_eq!(
            I256::from(i128::MIN),
            I256([0, 0x8000_0000_0000_0000, 0, 0]).neg()
        );
        assert_eq!(
            I256::from(i128::MAX),
            I256([0xFFFF_FFFF_FFFF_FFFF, 0x7FFF_FFFF_FFFF_FFFF, 0, 0])
        );

        // Test some random values
        let test_values = [
            42i128,
            -42i128,
            1_234_567_890i128,
            -1_234_567_890i128,
            i128::from(i64::MAX),
            i128::from(i64::MIN),
        ];

        for &value in &test_values {
            let converted = I256::from(value);
            if value >= 0 {
                assert_eq!(
                    converted.0[0],
                    (value as u128 & 0xFFFF_FFFF_FFFF_FFFF) as u64
                );
                assert_eq!(
                    converted.0[1],
                    ((value as u128 >> 64) & 0xFFFF_FFFF_FFFF_FFFF) as u64
                );
                assert_eq!(converted.0[2], 0);
                assert_eq!(converted.0[3], 0);
            } else {
                let abs_value = value.unsigned_abs();
                let expected = I256([
                    (abs_value & 0xFFFF_FFFF_FFFF_FFFF) as u64,
                    ((abs_value >> 64) & 0xFFFF_FFFF_FFFF_FFFF) as u64,
                    0,
                    0,
                ])
                .neg();
                assert_eq!(converted, expected);
            }
        }
    }
}
