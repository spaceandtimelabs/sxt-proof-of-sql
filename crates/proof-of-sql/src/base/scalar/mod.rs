//! This module contains the definition of the `Scalar` trait, which is used to represent the scalar field used in Proof of SQL.
mod error;
pub use error::ScalarConversionError;
mod mont_scalar;
#[cfg(test)]
mod mont_scalar_test;
use alloc::string::String;
use core::{cmp::Ordering, ops::Sub};
pub use mont_scalar::Curve25519Scalar;
pub(crate) use mont_scalar::MontScalar;
/// Module for a test Scalar
#[cfg(test)]
pub mod test_scalar;
#[cfg(test)]
mod test_scalar_test;
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
    + core::cmp::Ord
    + core::ops::Neg<Output = Self>
    + num_traits::Zero
    + core::ops::AddAssign
    + ark_serialize::CanonicalSerialize //This enables us to put `Scalar`s on the transcript
    + ark_std::UniformRand //This enables us to get `Scalar`s as challenges from the transcript
    + num_traits::Inv<Output = Option<Self>> // Note: `inv` should return `None` exactly when the element is zero.
    + core::ops::SubAssign
    + for<'a> core::convert::From<&'a String>
    + super::encode::VarInt
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
    /// Compare two `Scalar`s as signed numbers.
    fn signed_cmp(&self, other: &Self) -> Ordering {
        match *self - *other {
            x if x.is_zero() => Ordering::Equal,
            x if x > Self::MAX_SIGNED => Ordering::Less,
            _ => Ordering::Greater,
        }
    }

    fn from_limbs(val: [u64; 4]) -> Self;

    fn to_limbs(&self) -> [u64; 4];

}
