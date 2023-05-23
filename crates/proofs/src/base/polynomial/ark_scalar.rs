use ark_ff::BigInt;
use ark_ff::Field;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::iter::Sum;
use derive_more::{
    Add, AddAssign, Div, DivAssign, From, Mul, MulAssign, Neg, Product, Sub, SubAssign, Sum,
};
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::base::scalar::ToArkScalar;
use num_traits::Inv;

#[derive(
    Add,
    Mul,
    Div,
    Sub,
    AddAssign,
    MulAssign,
    DivAssign,
    SubAssign,
    Clone,
    Copy,
    Debug,
    Neg,
    From,
    PartialEq,
    Sum,
    Product,
    Default,
    CanonicalSerialize,
    CanonicalDeserialize,
    Eq,
    Hash,
)]
#[mul(forward)]
#[div(forward)]
#[mul_assign(forward)]
#[div_assign(forward)]
#[from(forward)]
/// A wrapper struct around the field element `ark_curve25519::Fr` and should be used in place of `ark_curve25519::Fr`.
///
/// Using traits rather than this struct is encouraged to allow for easier switching of the underlying field.
pub struct ArkScalar(pub ark_curve25519::Fr);

impl ArkScalar {
    pub fn from_bigint(repr: BigInt<4>) -> Option<Self> {
        ark_curve25519::Fr::from_bigint(repr).map(Self)
    }
    pub fn into_bigint(self) -> BigInt<4> {
        self.0.into_bigint()
    }
    pub fn from_le_bytes_mod_order(bytes: &[u8]) -> Self {
        Self(ark_curve25519::Fr::from_le_bytes_mod_order(bytes))
    }
    /// Prefer `into_scalar` unless you are absolutely sure you need dalek.
    pub fn into_dalek_scalar(self) -> curve25519_dalek::scalar::Scalar {
        let x = self.into_bigint();
        let bytes = ark_ff::BigInteger::to_bytes_le(&x);
        curve25519_dalek::scalar::Scalar::from_canonical_bytes(bytes.try_into().unwrap()).unwrap()
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.into_dalek_scalar().as_bytes().to_vec()
    }
    pub fn from_bytes_mod_order(bytes: [u8; 32]) -> Self {
        ToArkScalar::to_ark_scalar(&curve25519_dalek::scalar::Scalar::from_bytes_mod_order(
            bytes,
        ))
    }
    pub fn from_canonical_bytes(bytes: [u8; 32]) -> Option<Self> {
        curve25519_dalek::scalar::Scalar::from_canonical_bytes(bytes)
            .map(|x| ToArkScalar::to_ark_scalar(&x))
    }
    pub fn one() -> Self {
        num_traits::One::one()
    }
    pub fn zero() -> Self {
        num_traits::Zero::zero()
    }
    pub fn into_scalar(self) -> super::Scalar {
        self.into_dalek_scalar()
    }
    pub fn invert(self) -> Self {
        self.inv()
    }
    /// Convenience function for generating random values. Should not be used outside of tests. Instead, use a transcript.
    #[cfg(test)]
    pub fn rand<R: ark_std::rand::Rng + ?Sized>(rng: &mut R) -> Self {
        Self(ark_ff::UniformRand::rand(rng))
    }
    /// Convenience function for converting a slice of `ark_curve25519::Fr` into a vector of `ArkScalar`. Should not be used outside of tests.
    #[cfg(test)]
    pub fn wrap_slice(slice: &[ark_curve25519::Fr]) -> Vec<Self> {
        slice.iter().copied().map(Self).collect()
    }
    /// Convenience function for converting a slice of `ArkScalar` into a vector of `ark_curve25519::Fr`. Should not be used outside of tests.
    #[cfg(test)]
    pub fn unwrap_slice(slice: &[Self]) -> Vec<ark_curve25519::Fr> {
        slice.iter().map(|x| x.0).collect()
    }
    #[cfg(test)]
    pub fn random<R: rand_core::RngCore + rand_core::CryptoRng>(rng: &mut R) -> Self {
        ToArkScalar::to_ark_scalar(&curve25519_dalek::scalar::Scalar::random(rng))
    }
}

impl core::ops::Mul<curve25519_dalek::ristretto::RistrettoPoint> for ArkScalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        self.into_dalek_scalar() * rhs
    }
}
impl core::ops::Mul<ArkScalar> for curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: ArkScalar) -> Self::Output {
        self * rhs.into_dalek_scalar()
    }
}
impl core::ops::Mul<&curve25519_dalek::ristretto::RistrettoPoint> for ArkScalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: &curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        self.into_dalek_scalar() * rhs
    }
}
impl core::ops::Mul<ArkScalar> for &curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: ArkScalar) -> Self::Output {
        self * rhs.into_dalek_scalar()
    }
}

impl<'a> Sum<&'a ArkScalar> for ArkScalar {
    fn sum<I: Iterator<Item = &'a ArkScalar>>(iter: I) -> Self {
        ArkScalar(iter.map(|x| x.0).sum())
    }
}
impl num_traits::One for ArkScalar {
    fn one() -> Self {
        Self(ark_curve25519::Fr::one())
    }
}
impl num_traits::Zero for ArkScalar {
    fn zero() -> Self {
        Self(ark_curve25519::Fr::zero())
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
impl num_traits::Inv for ArkScalar {
    type Output = Self;
    fn inv(self) -> Self {
        Self(self.0.inverse().unwrap())
    }
}
impl Serialize for ArkScalar {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut bytes = vec![0u8; self.0.compressed_size()];
        self.0
            .serialize_compressed(&mut bytes)
            .map_err(serde::ser::Error::custom)?;
        bytes.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ArkScalar {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        ark_curve25519::Fr::deserialize_compressed(Vec::deserialize(deserializer)?.as_slice())
            .map_err(serde::de::Error::custom)
            .map(Self)
    }
}
impl core::ops::Neg for &ArkScalar {
    type Output = ArkScalar;
    fn neg(self) -> Self::Output {
        ArkScalar(-self.0)
    }
}
