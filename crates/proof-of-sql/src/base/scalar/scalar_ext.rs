use super::Scalar;
use bnum::types::U256;
use core::cmp::Ordering;

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
    use crate::base::scalar::{test_scalar::TestScalar, Curve25519Scalar, MontScalar};
    #[test]
    fn scalar_comparison_works() {
        let zero = Curve25519Scalar::ZERO;
        let one = Curve25519Scalar::ONE;
        let two = Curve25519Scalar::TWO;
        let max = Curve25519Scalar::MAX_SIGNED;
        let min = max + one;
        assert_eq!(max.signed_cmp(&one), Ordering::Greater);
        assert_eq!(one.signed_cmp(&zero), Ordering::Greater);
        assert_eq!(min.signed_cmp(&zero), Ordering::Less);
        assert_eq!((two * max).signed_cmp(&zero), Ordering::Less);
        assert_eq!(two * max + one, zero);
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
}
