#![allow(clippy::module_inception)]

use crate::base::{encode::VarInt, ref_into::RefInto, scalar::ScalarConversionError};
use alloc::string::String;
use core::ops::Sub;
use num_bigint::BigInt;

/// A trait for the scalar field used in Proof of SQL.
pub trait Scalar:
    Clone
    + core::fmt::Debug
    + core::fmt::Display
    + PartialEq
    + Default
    + for<'a> From<&'a str>
    + Sync
    + Send
    + num_traits::One
    + core::iter::Sum
    + core::iter::Product
    + Sub<Output = Self>
    + Copy
    + core::ops::MulAssign
    + core::ops::AddAssign
    + num_traits::Zero
    + for<'a> core::convert::From<&'a Self> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a bool> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i8> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i16> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i32> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i64> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i128> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a u8> // Required for `Column` to implement `MultilinearExtension`
    + core::convert::TryInto <bool>
    + core::convert::TryInto <i8>
    + core::convert::TryInto <i16>
    + core::convert::TryInto <i32>
    + core::convert::TryInto <i64>
    + core::convert::TryInto <i128>
    + core::convert::Into<[u64; 4]>
    + core::convert::From<[u64; 4]>
    + core::cmp::Ord
    + core::ops::Neg<Output = Self>
    + num_traits::Zero
    + core::ops::AddAssign
    + ark_serialize::CanonicalSerialize //This enables us to put `Scalar`s on the transcript
    + ark_std::UniformRand //This enables us to get `Scalar`s as challenges from the transcript
    + num_traits::Inv<Output = Option<Self>> // Note: `inv` should return `None` exactly when the element is zero.
    + core::ops::SubAssign
    + RefInto<[u64; 4]>
    + for<'a> core::convert::From<&'a String>
    + VarInt
    + core::convert::From<String>
    + core::convert::From<i128>
    + core::convert::From<i64>
    + core::convert::From<i32>
    + core::convert::From<i16>
    + core::convert::From<i8>
    + core::convert::From<bool>
    + core::convert::Into<BigInt>
    + TryFrom<BigInt, Error = ScalarConversionError>
{
    /// The value (p - 1) / 2. This is "mid-point" of the field - the "six" on the clock.
    /// It is the largest signed value that can be represented in the field with the natural embedding.
    const MAX_SIGNED: Self;
    /// The 0 (additive identity) element of the field.
    const ZERO: Self;
    /// The 1 (multiplicative identity) element of the field.
    const ONE: Self;
    /// 1 + 1
    const TWO: Self;
    /// 2 + 2 + 2 + 2 + 2
    const TEN: Self;
    /// The value to mask the challenge with to ensure it is in the field.
    /// This one less than the largest power of 2 that is less than the field modulus.
    const CHALLENGE_MASK: [u64; 4];
}

#[cfg(test)]
pub(crate) fn test_scalar_constants<S: Scalar>() {
    assert_eq!(S::from(0), S::ZERO);
    assert_eq!(S::from(1), S::ONE);
    assert_eq!(S::from(2), S::TWO);
    // -1/2 == least upper bound
    assert_eq!(-S::TWO.inv().unwrap(), S::MAX_SIGNED);
    assert_eq!(S::from(10), S::TEN);

    // Check the challenge mask
    let mid_point_limbs: [u64; 4] = S::MAX_SIGNED.into();
    let modulus_minus_one_limbs: [u64; 4] = (-S::ONE).into();
    assert_eq!(S::CHALLENGE_MASK[0], u64::MAX);
    assert_eq!(S::CHALLENGE_MASK[1], u64::MAX);
    assert_eq!(S::CHALLENGE_MASK[2], u64::MAX);
    assert_eq!(
        S::CHALLENGE_MASK[3],
        u64::MAX >> S::CHALLENGE_MASK[3].leading_zeros()
    );
    assert!(mid_point_limbs[3] < S::CHALLENGE_MASK[3]);
    assert!(modulus_minus_one_limbs[3] > S::CHALLENGE_MASK[3]);
}
