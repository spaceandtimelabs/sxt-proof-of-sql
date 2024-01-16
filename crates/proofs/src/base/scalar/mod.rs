mod ark_scalar;
#[cfg(test)]
mod ark_scalar_test;
pub use ark_scalar::ArkScalar;
use core::ops::Sub;
mod ark_scalar_from;
#[cfg(test)]
mod ark_scalar_from_test;

#[cfg(any(test, feature = "test"))]
mod commitment_utility;
#[cfg(any(test, feature = "test"))]
pub use commitment_utility::compute_commitment_for_testing;

pub trait Scalar:
    Clone
    + core::fmt::Debug
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
{
}
impl Scalar for ArkScalar {}
