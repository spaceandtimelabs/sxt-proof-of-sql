mod ark_scalar;
#[cfg(test)]
mod ark_scalar_test;
pub use ark_scalar::{ArkScalar, MontScalar};
use core::ops::Sub;
mod ark_scalar_from;
#[cfg(test)]
mod ark_scalar_from_test;

#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
mod commitment_utility;
#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
pub use commitment_utility::compute_commitment_for_testing;

/// A trait for the scalar field used in proofs.
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
    + std::iter::Sum
    + std::iter::Product
    + Sub<Output = Self>
    + Copy
    + std::ops::MulAssign
    + std::ops::AddAssign
    + num_traits::Zero
    + for<'a> std::convert::From<&'a Self> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a i64> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a i128> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a bool>
    + for<'a> std::convert::From<&'a i32>
    + std::convert::Into<[u64; 4]>
    + std::convert::From<[u64; 4]>
    + core::cmp::PartialOrd
    + std::ops::Neg<Output = Self>
    + num_traits::Zero
    + std::ops::AddAssign
    + ark_serialize::CanonicalSerialize //This enables us to put `Scalar`s on the transcript
    + ark_std::UniformRand //This enables us to get `Scalar`s as challenges from the transcript
    + num_traits::Inv<Output = Option<Self>> // Note: `inv` should return `None` exactly when the element is zero.
    + std::ops::SubAssign
    + super::ref_into::RefInto<[u64; 4]>
    + for<'a> std::convert::From<&'a String>
    + super::encode::VarInt
    + std::convert::From<String>
    + std::convert::From<i128>
{
    /// The value (p - 1) / 2. This is "mid-point" of the field - the "six" on the clock.
    /// It is the largest signed value that can be represented in the field with the natural embedding.
    const MAX_SIGNED: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
}
