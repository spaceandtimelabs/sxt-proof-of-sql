//! This module defines the zero and one identities for various types.
//! This is useful for writing generic code. We cannot use the `num_traits` crate because
//! Scalar types do not implement the `num_traits` traits. As a result, we define our own
//! traits here and implement them for the types we care about.

/// This trait is used to define the zero identity element of field elements (or other types).
pub trait Zero {
    /// Returns the zero element of the type.
    fn zero() -> Self;
}
/// This trait is used to define the one identity element of field elements (or other types).
pub trait One {
    /// Returns the one element of the type.
    fn one() -> Self;
}

trait IdentityMarker {}
impl<T> One for T
where
    T: num_traits::One + IdentityMarker,
{
    fn one() -> Self {
        num_traits::One::one()
    }
}
impl<T> Zero for T
where
    T: num_traits::Zero + IdentityMarker,
{
    fn zero() -> Self {
        num_traits::Zero::zero()
    }
}
impl IdentityMarker for bool {}
impl IdentityMarker for u8 {}
impl IdentityMarker for u16 {}
impl IdentityMarker for u32 {}
impl IdentityMarker for u64 {}
impl IdentityMarker for u128 {}
impl IdentityMarker for usize {}
impl IdentityMarker for i8 {}
impl IdentityMarker for i16 {}
impl IdentityMarker for i32 {}
impl IdentityMarker for i64 {}
impl IdentityMarker for i128 {}
impl IdentityMarker for isize {}

impl<P: ark_ff::FpConfig<N>, const N: usize> IdentityMarker for ark_ff::Fp<P, N> {}

impl One for curve25519_dalek::scalar::Scalar {
    fn one() -> Self {
        curve25519_dalek::scalar::Scalar::one()
    }
}
impl Zero for curve25519_dalek::scalar::Scalar {
    fn zero() -> Self {
        curve25519_dalek::scalar::Scalar::zero()
    }
}
