use super::{Scalar, ScalarConversionError};
use crate::base::math::decimal::MAX_SUPPORTED_PRECISION;
use ark_ff::{BigInteger, Field, Fp, Fp256, MontBackend, MontConfig, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use bytemuck::TransparentWrapper;
use core::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use num_bigint::BigInt;
use num_traits::Signed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(CanonicalSerialize, CanonicalDeserialize, TransparentWrapper)]
/// A wrapper struct around a `Fp256<MontBackend<T, 4>>` that can easily implement the `Scalar` trait.
///
/// Using the `Scalar` trait rather than this type is encouraged to allow for easier switching of the underlying field.
#[repr(transparent)]
pub struct MontScalar<T: MontConfig<4>>(pub Fp256<MontBackend<T, 4>>);

// --------------------------------------------------------------------------------
// replacement for #[derive(Add, Sub, Mul, AddAssign, SubAssign, MulAssign, Neg,
//  Sum, Product, Clone, Copy, PartialOrd, PartialEq, Default, Debug, Eq, Hash, Ord)]
// --------------------------------------------------------------------------------
impl<T: MontConfig<4>> Add for MontScalar<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl<T: MontConfig<4>> Sub for MontScalar<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl<T: MontConfig<4>> Mul for MontScalar<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl<T: MontConfig<4>> AddAssign for MontScalar<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl<T: MontConfig<4>> SubAssign for MontScalar<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl<T: MontConfig<4>> MulAssign for MontScalar<T> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}
impl<T: MontConfig<4>> Neg for MontScalar<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
impl<T: MontConfig<4>> Sum for MontScalar<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).sum())
    }
}
impl<T: MontConfig<4>> Product for MontScalar<T> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).product())
    }
}
impl<T: MontConfig<4>> Clone for MontScalar<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: MontConfig<4>> Copy for MontScalar<T> {}
impl<T: MontConfig<4>> PartialOrd for MontScalar<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: MontConfig<4>> PartialEq for MontScalar<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: MontConfig<4>> Default for MontScalar<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T: MontConfig<4>> Debug for MontScalar<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MontScalar").field(&self.0).finish()
    }
}
impl<T: MontConfig<4>> Eq for MontScalar<T> {}
impl<T: MontConfig<4>> Hash for MontScalar<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
impl<T: MontConfig<4>> Ord for MontScalar<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
// --------------------------------------------------------------------------------
// end replacement for #[derive(...)]
// --------------------------------------------------------------------------------

/// A wrapper type around the field element `ark_curve25519::Fr` and should be used in place of `ark_curve25519::Fr`.
///
/// Using the `Scalar` trait rather than this type is encouraged to allow for easier switching of the underlying field.
pub type Curve25519Scalar = MontScalar<ark_curve25519::FrConfig>;

impl<T: MontConfig<4>> MontScalar<T> {
    /// Convenience function for creating a new `MontScalar<T>` from the underlying `Fp256<MontBackend<T, 4>>`. Should only be used in tests.
    #[cfg(test)]
    pub fn new(value: Fp256<MontBackend<T, 4>>) -> Self {
        Self(value)
    }
    /// Create a new `MontScalar<T>` from a `[u64, 4]`. The array is expected to be in non-montgomery form.
    pub fn from_bigint(vals: [u64; 4]) -> Self {
        Self(Fp::from_bigint(ark_ff::BigInt(vals)).unwrap())
    }
    /// Create a new `MontScalar<T>` from a `[u8]` modulus the field order. The array is expected to be in non-montgomery form.
    pub fn from_le_bytes_mod_order(bytes: &[u8]) -> Self {
        Self(Fp::from_le_bytes_mod_order(bytes))
    }
    /// Create a `Vec<u8>` from a `MontScalar<T>`. The array will be in non-montgomery form.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes_le(&self) -> Vec<u8> {
        self.0.into_bigint().to_bytes_le()
    }
    /// Convenience function for converting a slice of `ark_curve25519::Fr` into a vector of `Curve25519Scalar`. Should not be used outside of tests.
    #[cfg(test)]
    pub fn wrap_slice(slice: &[Fp256<MontBackend<T, 4>>]) -> Vec<Self> {
        slice.iter().copied().map(Self).collect()
    }
    /// Convenience function for converting a slice of `Curve25519Scalar` into a vector of `ark_curve25519::Fr`. Should not be used outside of tests.
    #[cfg(test)]
    pub fn unwrap_slice(slice: &[Self]) -> Vec<Fp256<MontBackend<T, 4>>> {
        slice.iter().map(|x| x.0).collect()
    }
}

impl<T: MontConfig<4>> TryFrom<BigInt> for MontScalar<T> {
    type Error = ScalarConversionError;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        // Obtain the absolute value to ignore the sign when counting digits
        let value_abs = value.abs();

        // Extract digits and check the number of digits directly
        let (_, digits) = value_abs.to_u64_digits();

        // Check if the number of digits exceeds the maximum precision allowed
        if digits.len() > MAX_SUPPORTED_PRECISION.into() {
            return Err(ScalarConversionError::Overflow(format!(
                "Attempted to parse a number with {} digits, which exceeds the max supported precision of {}",
                digits.len(),
                MAX_SUPPORTED_PRECISION
            )));
        }

        // Continue with the previous logic
        assert!(digits.len() <= 4); // This should not happen if the precision check is correct
        let mut data = [0u64; 4];
        data[..digits.len()].copy_from_slice(&digits);
        let result = Self::from_bigint(data);
        match value.sign() {
            // Updated to use value.sign() for clarity
            num_bigint::Sign::Minus => Ok(-result),
            _ => Ok(result),
        }
    }
}
impl<T: MontConfig<4>> From<[u64; 4]> for MontScalar<T> {
    fn from(value: [u64; 4]) -> Self {
        Self(Fp::new(ark_ff::BigInt(value)))
    }
}

impl<T: MontConfig<4>> ark_std::UniformRand for MontScalar<T> {
    fn rand<R: ark_std::rand::Rng + ?Sized>(rng: &mut R) -> Self {
        Self(ark_ff::UniformRand::rand(rng))
    }
}

impl core::ops::Mul<curve25519_dalek::ristretto::RistrettoPoint> for Curve25519Scalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<Curve25519Scalar> for curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}
impl core::ops::Mul<&curve25519_dalek::ristretto::RistrettoPoint> for Curve25519Scalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: &curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<Curve25519Scalar> for &curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}

impl<'a, T: MontConfig<4>> Sum<&'a Self> for MontScalar<T> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).sum())
    }
}
impl<T: MontConfig<4>> num_traits::One for MontScalar<T> {
    fn one() -> Self {
        Self(Fp::one())
    }
}
impl<T: MontConfig<4>> num_traits::Zero for MontScalar<T> {
    fn zero() -> Self {
        Self(Fp::zero())
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
impl<T: MontConfig<4>> num_traits::Inv for MontScalar<T> {
    type Output = Option<Self>;
    fn inv(self) -> Option<Self> {
        self.0.inverse().map(Self)
    }
}
impl<T: MontConfig<4>> Serialize for MontScalar<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut bytes = Vec::with_capacity(self.0.compressed_size());
        self.0
            .serialize_compressed(&mut bytes)
            .map_err(serde::ser::Error::custom)?;
        bytes.serialize(serializer)
    }
}
impl<'de, T: MontConfig<4>> Deserialize<'de> for MontScalar<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        CanonicalDeserialize::deserialize_compressed(Vec::deserialize(deserializer)?.as_slice())
            .map_err(serde::de::Error::custom)
            .map(Self)
    }
}

impl<T: MontConfig<4>> core::ops::Neg for &MontScalar<T> {
    type Output = MontScalar<T>;
    fn neg(self) -> Self::Output {
        MontScalar(-self.0)
    }
}
impl From<Curve25519Scalar> for curve25519_dalek::scalar::Scalar {
    fn from(value: Curve25519Scalar) -> Self {
        (&value).into()
    }
}

impl From<&Curve25519Scalar> for curve25519_dalek::scalar::Scalar {
    fn from(value: &Curve25519Scalar) -> Self {
        let bytes = ark_ff::BigInteger::to_bytes_le(&value.0.into_bigint());
        curve25519_dalek::scalar::Scalar::from_canonical_bytes(bytes.try_into().unwrap()).unwrap()
    }
}

impl<T: MontConfig<4>> From<MontScalar<T>> for [u64; 4] {
    fn from(value: MontScalar<T>) -> Self {
        (&value).into()
    }
}

impl<T: MontConfig<4>> From<&MontScalar<T>> for [u64; 4] {
    fn from(value: &MontScalar<T>) -> Self {
        value.0.into_bigint().0
    }
}

impl<T: MontConfig<4>> Display for MontScalar<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sign = match f.sign_plus() {
            true => {
                let n = -self;
                match self > &n {
                    true => Some(Some(n)),
                    false => Some(None),
                }
            }
            false => None,
        };
        match (f.alternate(), sign) {
            (false, None) => {
                let data = self.0.into_bigint().0;
                write!(
                    f,
                    "{:016X}{:016X}{:016X}{:016X}",
                    data[3], data[2], data[1], data[0],
                )
            }
            (false, Some(None)) => {
                let data = self.0.into_bigint().0;
                write!(
                    f,
                    "+{:016X}{:016X}{:016X}{:016X}",
                    data[3], data[2], data[1], data[0],
                )
            }
            (false, Some(Some(n))) => {
                let data = n.0.into_bigint().0;
                write!(
                    f,
                    "-{:016X}{:016X}{:016X}{:016X}",
                    data[3], data[2], data[1], data[0],
                )
            }
            (true, None) => {
                let data = self.to_bytes_le();
                write!(
                    f,
                    "0x{:02X}{:02X}...{:02X}{:02X}",
                    data[31], data[30], data[1], data[0],
                )
            }
            (true, Some(None)) => {
                let data = self.to_bytes_le();
                write!(
                    f,
                    "+0x{:02X}{:02X}...{:02X}{:02X}",
                    data[31], data[30], data[1], data[0],
                )
            }
            (true, Some(Some(n))) => {
                let data = n.to_bytes_le();
                write!(
                    f,
                    "-0x{:02X}{:02X}...{:02X}{:02X}",
                    data[31], data[30], data[1], data[0],
                )
            }
        }
    }
}

impl super::Scalar for Curve25519Scalar {
    const MAX_SIGNED: Self = Self(ark_ff::MontFp!(
        "3618502788666131106986593281521497120428558179689953803000975469142727125494"
    ));
    const ZERO: Self = Self(ark_ff::MontFp!("0"));
    const ONE: Self = Self(ark_ff::MontFp!("1"));
    const TWO: Self = Self(ark_ff::MontFp!("2"));
}

impl<T> TryFrom<MontScalar<T>> for bool
where
    T: MontConfig<4>,
    MontScalar<T>: Scalar,
{
    type Error = ScalarConversionError;
    fn try_from(value: MontScalar<T>) -> Result<Self, Self::Error> {
        let (sign, abs): (i128, [u64; 4]) = if value > <MontScalar<T>>::MAX_SIGNED {
            (-1, (-value).into())
        } else {
            (1, value.into())
        };
        if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
            return Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in an i8",
                value
            )));
        }
        let val: i128 = sign * abs[0] as i128;
        match val {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in a bool",
                value
            ))),
        }
    }
}

impl<T> TryFrom<MontScalar<T>> for i8
where
    T: MontConfig<4>,
    MontScalar<T>: Scalar,
{
    type Error = ScalarConversionError;
    fn try_from(value: MontScalar<T>) -> Result<Self, Self::Error> {
        let (sign, abs): (i128, [u64; 4]) = if value > <MontScalar<T>>::MAX_SIGNED {
            (-1, (-value).into())
        } else {
            (1, value.into())
        };
        if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
            return Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in an i8",
                value
            )));
        }
        let val: i128 = sign * abs[0] as i128;
        val.try_into().map_err(|_| {
            ScalarConversionError::Overflow(format!("{} is too large to fit in an i8", value))
        })
    }
}

impl<T> TryFrom<MontScalar<T>> for i16
where
    T: MontConfig<4>,
    MontScalar<T>: Scalar,
{
    type Error = ScalarConversionError;
    fn try_from(value: MontScalar<T>) -> Result<Self, Self::Error> {
        let (sign, abs): (i128, [u64; 4]) = if value > <MontScalar<T>>::MAX_SIGNED {
            (-1, (-value).into())
        } else {
            (1, value.into())
        };
        if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
            return Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in an i16",
                value
            )));
        }
        let val: i128 = sign * abs[0] as i128;
        val.try_into().map_err(|_| {
            ScalarConversionError::Overflow(format!("{} is too large to fit in an i16", value))
        })
    }
}

impl<T> TryFrom<MontScalar<T>> for i32
where
    T: MontConfig<4>,
    MontScalar<T>: Scalar,
{
    type Error = ScalarConversionError;
    fn try_from(value: MontScalar<T>) -> Result<Self, Self::Error> {
        let (sign, abs): (i128, [u64; 4]) = if value > <MontScalar<T>>::MAX_SIGNED {
            (-1, (-value).into())
        } else {
            (1, value.into())
        };
        if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
            return Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in an i32",
                value
            )));
        }
        let val: i128 = sign * abs[0] as i128;
        val.try_into().map_err(|_| {
            ScalarConversionError::Overflow(format!("{} is too large to fit in an i32", value))
        })
    }
}

impl<T> TryFrom<MontScalar<T>> for i64
where
    T: MontConfig<4>,
    MontScalar<T>: Scalar,
{
    type Error = ScalarConversionError;
    fn try_from(value: MontScalar<T>) -> Result<Self, Self::Error> {
        let (sign, abs): (i128, [u64; 4]) = if value > <MontScalar<T>>::MAX_SIGNED {
            (-1, (-value).into())
        } else {
            (1, value.into())
        };
        if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
            return Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in an i64",
                value
            )));
        }
        let val: i128 = sign * abs[0] as i128;
        val.try_into().map_err(|_| {
            ScalarConversionError::Overflow(format!("{} is too large to fit in an i64", value))
        })
    }
}

impl<T> TryFrom<MontScalar<T>> for i128
where
    T: MontConfig<4>,
    MontScalar<T>: Scalar,
{
    type Error = ScalarConversionError;
    fn try_from(value: MontScalar<T>) -> Result<Self, Self::Error> {
        let (sign, abs): (i128, [u64; 4]) = if value > <MontScalar<T>>::MAX_SIGNED {
            (-1, (-value).into())
        } else {
            (1, value.into())
        };
        if abs[2] != 0 || abs[3] != 0 {
            return Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in an i128",
                value
            )));
        }
        let val: u128 = (abs[1] as u128) << 64 | (abs[0] as u128);
        match (sign, val) {
            (1, v) if v <= i128::MAX as u128 => Ok(v as i128),
            (-1, v) if v <= i128::MAX as u128 => Ok(-(v as i128)),
            (-1, v) if v == i128::MAX as u128 + 1 => Ok(i128::MIN),
            _ => Err(ScalarConversionError::Overflow(format!(
                "{} is too large to fit in an i128",
                value
            ))),
        }
    }
}

impl<T> From<MontScalar<T>> for BigInt
where
    T: MontConfig<4>,
    MontScalar<T>: Scalar,
{
    fn from(value: MontScalar<T>) -> Self {
        // Since we wrap around in finite fields anything greater than the max signed value is negative
        let is_negative = value > <MontScalar<T>>::MAX_SIGNED;
        let sign = if is_negative {
            num_bigint::Sign::Minus
        } else {
            num_bigint::Sign::Plus
        };
        let value_abs: [u64; 4] = (if is_negative { -value } else { value }).into();
        let bits: &[u8] = bytemuck::cast_slice(&value_abs);
        BigInt::from_bytes_le(sign, bits)
    }
}
