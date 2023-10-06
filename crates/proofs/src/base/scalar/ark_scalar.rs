use ark_ff::{Field, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::iter::Sum;
use derive_more::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Product, Sub, SubAssign, Sum,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
/// A wrapper struct around the field element `ark_curve25519::Fr` and should be used in place of `ark_curve25519::Fr`.
///
/// Using traits rather than this struct is encouraged to allow for easier switching of the underlying field.
pub struct ArkScalar(pub ark_curve25519::Fr);

impl ArkScalar {
    pub fn from_bigint(vals: [u64; 4]) -> Self {
        Self(ark_curve25519::Fr::from_bigint(ark_ff::BigInt(vals)).unwrap())
    }
    pub fn from_le_bytes_mod_order(bytes: &[u8]) -> Self {
        Self(ark_curve25519::Fr::from_le_bytes_mod_order(bytes))
    }
    #[cfg(test)]
    pub fn to_bytes_le(&self) -> Vec<u8> {
        use ark_ff::BigInteger;
        self.0.into_bigint().to_bytes_le()
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
}

impl core::ops::Mul<curve25519_dalek::ristretto::RistrettoPoint> for ArkScalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<ArkScalar> for curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: ArkScalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}
impl core::ops::Mul<&curve25519_dalek::ristretto::RistrettoPoint> for ArkScalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: &curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<ArkScalar> for &curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: ArkScalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
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
        let mut bytes = Vec::with_capacity(self.0.compressed_size());
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
impl From<ArkScalar> for curve25519_dalek::scalar::Scalar {
    fn from(value: ArkScalar) -> Self {
        (&value).into()
    }
}

impl From<&ArkScalar> for curve25519_dalek::scalar::Scalar {
    fn from(value: &ArkScalar) -> Self {
        let bytes = ark_ff::BigInteger::to_bytes_le(&value.0.into_bigint());
        curve25519_dalek::scalar::Scalar::from_canonical_bytes(bytes.try_into().unwrap()).unwrap()
    }
}

impl From<ArkScalar> for [u64; 4] {
    fn from(value: ArkScalar) -> Self {
        (&value).into()
    }
}

impl From<&ArkScalar> for [u64; 4] {
    fn from(value: &ArkScalar) -> Self {
        value.0.into_bigint().0
    }
}
