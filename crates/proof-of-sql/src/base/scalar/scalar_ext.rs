use super::Scalar;
use bnum::types::U256;
use core::cmp::Ordering;
use tiny_keccak::Hasher;

/// Extension trait for blanket implementations for `Scalar` types.
/// This trait is primarily to avoid cluttering the core `Scalar` implementation with default implementations
/// and provides helper methods for `Scalar`.
pub trait ScalarExt: Scalar {
    /// Compute 10^exponent for the Scalar. Note that we do not check for overflow.
    fn pow10(exponent: u8) -> Self {
        itertools::repeat_n(Self::TEN, exponent as usize).product()
    }
    /// Compare two `Scalar`s as signed numbers.
    fn signed_cmp(&self, other: &Self) -> Ordering {
        match *self - *other {
            x if x.is_zero() => Ordering::Equal,
            x if x > Self::MAX_SIGNED => Ordering::Less,
            _ => Ordering::Greater,
        }
    }

    #[must_use]
    /// Converts a U256 to Scalar, wrapping as needed
    fn from_wrapping(value: U256) -> Self {
        let value_as_limbs: [u64; 4] = value.into();
        Self::from(value_as_limbs)
    }

    /// Converts a Scalar to U256. Note that any values above `MAX_SIGNED` shall remain positive, even if they are representative of negative values.
    fn into_u256_wrapping(self) -> U256 {
        U256::from(Into::<[u64; 4]>::into(self))
    }

    /// Converts a byte slice to a Scalar using a hash function, preventing collisions.
    /// WARNING: Only up to 31 bytes (2^248 bits) are supported by `PoSQL` cryptographic
    /// objects. This function masks off the last byte of the hash to ensure the result
    /// fits in this range.
    #[must_use]
    fn from_byte_slice_via_hash(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self::zero();
        }

        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(bytes);
        let mut hashed_bytes = [0u8; 32];
        hasher.finalize(&mut hashed_bytes);
        let hashed_val =
            U256::from_le_slice(&hashed_bytes).expect("32 bytes => guaranteed to parse as U256");
        let masked_val = hashed_val & Self::CHALLENGE_MASK;
        Self::from_wrapping(masked_val)
    }
}

impl<S: Scalar> ScalarExt for S {}

#[cfg(test)]
pub(crate) fn test_scalar_constants<S: Scalar>() {
    assert_eq!(S::from(0), S::ZERO);
    assert_eq!(S::from(1), S::ONE);
    assert_eq!(S::from(2), S::TWO);
    // -1/2 == least upper bound
    assert_eq!(-S::TWO.inv().unwrap(), S::MAX_SIGNED);
    assert_eq!(S::from(10), S::TEN);

    // Check the challenge mask
    assert_eq!(
        S::CHALLENGE_MASK,
        U256::MAX >> S::CHALLENGE_MASK.leading_zeros()
    );
    assert!(S::MAX_SIGNED.into_u256_wrapping() < S::CHALLENGE_MASK);
    assert!((-S::ONE).into_u256_wrapping() > S::CHALLENGE_MASK);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::{test_scalar::TestScalar, MontScalar};
    use bytemuck::cast;

    #[test]
    fn we_can_get_zero_from_zero_bytes() {
        assert_eq!(TestScalar::from_byte_slice_via_hash(&[]), TestScalar::ZERO);
    }

    #[test]
    fn we_can_get_scalar_from_hashed_bytes() {
        // Raw bytes of test string "abc" with 31st byte zeroed out:
        let expected: [u8; 32] = [
            0x4e, 0x03, 0x65, 0x7a, 0xea, 0x45, 0xa9, 0x4f, 0xc7, 0xd4, 0x7b, 0xa8, 0x26, 0xc8,
            0xd6, 0x67, 0xc0, 0xd1, 0xe6, 0xe3, 0x3a, 0x64, 0xa0, 0x36, 0xec, 0x44, 0xf5, 0x8f,
            0xa1, 0x2d, 0x6c, 0x05,
        ];

        let scalar_from_bytes: TestScalar = TestScalar::from_byte_slice_via_hash(b"abc");

        let limbs_native: [u64; 4] = cast(expected);
        let limbs_le = [
            u64::from_le_bytes(limbs_native[0].to_le_bytes()),
            u64::from_le_bytes(limbs_native[1].to_le_bytes()),
            u64::from_le_bytes(limbs_native[2].to_le_bytes()),
            u64::from_le_bytes(limbs_native[3].to_le_bytes()),
        ];
        let scalar_from_ref = TestScalar::from(limbs_le);

        assert_eq!(
            scalar_from_bytes, scalar_from_ref,
            "The masked keccak v256 of 'abc' must match"
        );
    }

    #[test]
    fn we_can_compute_powers_of_10() {
        for i in 0..=u128::MAX.ilog10() {
            assert_eq!(
                TestScalar::pow10(u8::try_from(i).unwrap()),
                TestScalar::from(u128::pow(10, i))
            );
        }
        assert_eq!(
            TestScalar::pow10(76),
            MontScalar(ark_ff::MontFp!(
                "10000000000000000000000000000000000000000000000000000000000000000000000000000"
            ))
        );
    }

    #[test]
    fn scalar_comparison_works() {
        let zero = TestScalar::ZERO;
        let one = TestScalar::ONE;
        let two = TestScalar::TWO;
        let max = TestScalar::MAX_SIGNED;
        let min = max + one;
        assert_eq!(max.signed_cmp(&one), Ordering::Greater);
        assert_eq!(one.signed_cmp(&zero), Ordering::Greater);
        assert_eq!(min.signed_cmp(&zero), Ordering::Less);
        assert_eq!((two * max).signed_cmp(&zero), Ordering::Less);
        assert_eq!(two * max + one, zero);
    }
}
