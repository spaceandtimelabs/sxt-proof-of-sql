//! This module contains the definition of the `Scalar` trait, which is used to represent the scalar field used in Proof of SQL.
mod error;
pub use error::ScalarConversionError;
mod mont_scalar;
#[cfg(test)]
mod mont_scalar_test;
use crate::sql::parse::ConversionError;
use core::{cmp::Ordering, ops::Sub};
pub use mont_scalar::Curve25519Scalar;
pub(crate) use mont_scalar::MontScalar;
mod mont_scalar_from;
#[cfg(test)]
mod mont_scalar_from_test;

#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
mod commitment_utility;
#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
pub use commitment_utility::compute_commitment_for_testing;
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
    + std::iter::Sum
    + std::iter::Product
    + Sub<Output = Self>
    + Copy
    + std::ops::MulAssign
    + std::ops::AddAssign
    + num_traits::Zero
    + for<'a> std::convert::From<&'a Self> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a bool> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a i16> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a i32> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a i64> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> std::convert::From<&'a i128> // Required for `Column` to implement `MultilinearExtension`
    + std::convert::TryInto <i8>
    + std::convert::TryInto <i16>
    + std::convert::TryInto <i32>
    + std::convert::TryInto <i64>
    + std::convert::TryInto <i128>
    + std::convert::Into<[u64; 4]>
    + std::convert::From<[u64; 4]>
    + core::cmp::Ord
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
    + std::convert::From<i64>
    + std::convert::From<i32>
    + std::convert::From<i16>
    + std::convert::From<bool>
    + TryFrom<BigInt, Error = ConversionError>
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
}

macro_rules! scalar_conversion_to_int {
    ($scalar:ty) => {
        impl TryInto<i8> for $scalar {
            type Error = ScalarConversionError;
            fn try_into(self) -> Result<i8, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if self > Self::MAX_SIGNED {
                    (-1, (-self).into())
                } else {
                    (1, self.into())
                };
                if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
                    return Err(ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i8",
                        self
                    )));
                }
                let val: i128 = sign * abs[0] as i128;
                val.try_into().map_err(|_| {
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i8",
                        self
                    ))
                })
            }
        }

        impl TryInto<i16> for $scalar {
            type Error = ScalarConversionError;
            fn try_into(self) -> Result<i16, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if self > Self::MAX_SIGNED {
                    (-1, (-self).into())
                } else {
                    (1, self.into())
                };
                if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
                    return Err(ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i16",
                        self
                    )));
                }
                let val: i128 = sign * abs[0] as i128;
                val.try_into().map_err(|_| {
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i16",
                        self
                    ))
                })
            }
        }

        impl TryInto<i32> for $scalar {
            type Error = ScalarConversionError;
            fn try_into(self) -> Result<i32, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if self > Self::MAX_SIGNED {
                    (-1, (-self).into())
                } else {
                    (1, self.into())
                };
                if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
                    return Err(ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i32",
                        self
                    )));
                }
                let val: i128 = sign * abs[0] as i128;
                val.try_into().map_err(|_| {
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i32",
                        self
                    ))
                })
            }
        }

        impl TryInto<i64> for $scalar {
            type Error = ScalarConversionError;
            fn try_into(self) -> Result<i64, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if self > Self::MAX_SIGNED {
                    (-1, (-self).into())
                } else {
                    (1, self.into())
                };
                if abs[1] != 0 || abs[2] != 0 || abs[3] != 0 {
                    return Err(ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i64",
                        self
                    )));
                }
                let val: i128 = sign * abs[0] as i128;
                val.try_into().map_err(|_| {
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i64",
                        self
                    ))
                })
            }
        }

        impl TryInto<i128> for $scalar {
            type Error = ScalarConversionError;
            fn try_into(self) -> Result<i128, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if self > Self::MAX_SIGNED {
                    (-1, (-self).into())
                } else {
                    (1, self.into())
                };
                if abs[2] != 0 || abs[3] != 0 {
                    return Err(ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i128",
                        self
                    )));
                }
                let val: u128 = (abs[1] as u128) << 64 | (abs[0] as u128);
                match (sign, val) {
                    (1, v) if v <= i128::MAX as u128 => Ok(v as i128),
                    (-1, v) if v <= i128::MAX as u128 => Ok(-(v as i128)),
                    (-1, v) if v == i128::MAX as u128 + 1 => Ok(i128::MIN),
                    _ => Err(ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i128",
                        self
                    ))),
                }
            }
        }
    };
}

pub(crate) use scalar_conversion_to_int;
