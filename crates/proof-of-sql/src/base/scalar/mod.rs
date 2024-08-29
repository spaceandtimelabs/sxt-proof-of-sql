//! This module contains the definition of the `Scalar` trait, which is used to represent the scalar field used in Proof of SQL.
mod error;
pub use error::ScalarConversionError;
mod mont_scalar;
#[cfg(test)]
mod mont_scalar_test;
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
    + core::iter::Sum
    + core::iter::Product
    + Sub<Output = Self>
    + Copy
    + core::ops::MulAssign
    + core::ops::AddAssign
    + num_traits::Zero
    + for<'a> core::convert::From<&'a Self> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a bool> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a u8> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i16> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i32> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i64> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i128> // Required for `Column` to implement `MultilinearExtension`
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
    + super::ref_into::RefInto<[u64; 4]>
    + for<'a> core::convert::From<&'a String>
    + super::encode::VarInt
    + core::convert::From<String>
    + core::convert::From<i128>
    + core::convert::From<i64>
    + core::convert::From<i32>
    + core::convert::From<i16>
    + core::convert::From<u8>
    + core::convert::From<u32>
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
}

macro_rules! scalar_conversion_to_int {
    ($scalar:ty) => {
        impl TryFrom<$scalar> for bool {
            type Error = ScalarConversionError;
            fn try_from(value: $scalar) -> Result<Self, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if value > <$scalar>::MAX_SIGNED {
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

        impl TryFrom<$scalar> for i8 {
            type Error = ScalarConversionError;
            fn try_from(value: $scalar) -> Result<Self, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if value > <$scalar>::MAX_SIGNED {
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
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i8",
                        value
                    ))
                })
            }
        }

        impl TryFrom<$scalar> for i16 {
            type Error = ScalarConversionError;
            fn try_from(value: $scalar) -> Result<Self, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if value > <$scalar>::MAX_SIGNED {
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
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i16",
                        value
                    ))
                })
            }
        }

        impl TryFrom<$scalar> for i32 {
            type Error = ScalarConversionError;
            fn try_from(value: $scalar) -> Result<Self, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if value > <$scalar>::MAX_SIGNED {
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
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i32",
                        value
                    ))
                })
            }
        }

        impl TryFrom<$scalar> for i64 {
            type Error = ScalarConversionError;
            fn try_from(value: $scalar) -> Result<Self, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if value > <$scalar>::MAX_SIGNED {
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
                    ScalarConversionError::Overflow(format!(
                        "{} is too large to fit in an i64",
                        value
                    ))
                })
            }
        }

        impl TryFrom<$scalar> for i128 {
            type Error = ScalarConversionError;
            fn try_from(value: $scalar) -> Result<Self, Self::Error> {
                let (sign, abs): (i128, [u64; 4]) = if value > <$scalar>::MAX_SIGNED {
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

        impl From<$scalar> for BigInt {
            fn from(value: $scalar) -> Self {
                // Since we wrap around in finite fields anything greater than the max signed value is negative
                let is_negative = value > <$scalar>::MAX_SIGNED;
                let sign = if is_negative {
                    num_bigint::Sign::Minus
                } else {
                    num_bigint::Sign::Plus
                };
                let value_abs: [u64; 4] = (if is_negative { -value } else { value }).into();
                let bits: &[u8] = bytemuck::cast_slice(&value_abs);
                BigInt::from_bytes_le(sign, &bits)
            }
        }
    };
}

pub(crate) use scalar_conversion_to_int;
