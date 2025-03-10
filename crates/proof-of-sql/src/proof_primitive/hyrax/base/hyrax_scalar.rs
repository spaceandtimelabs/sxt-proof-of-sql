use crate::base::{
    encode::VarInt,
    scalar::{Scalar, ScalarConversionError},
};
use alloc::string::String;
use ark_ff::{One, UniformRand};
use ark_serialize::CanonicalSerialize;
use core::ops::{Mul, MulAssign};
use derive_more::{Add, AddAssign, Display, Neg, Product, Sub, SubAssign, Sum};
use num_bigint::BigInt;
use num_traits::{Inv, Zero};
use serde::{Deserialize, Serialize};

pub trait HyraxScalar:
    for<'a> core::convert::From<&'a [u64; 4]> + Serialize + Scalar + Mul<Self, Output = Self>
{
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    CanonicalSerialize,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Default,
    Neg,
    AddAssign,
    Sub,
    SubAssign,
    Add,
    Display,
    Sum,
    Product,
)]

/// A wrapper type for Scalars that implement `HyraxScalar`.
#[serde(bound = "S: for<'a> Deserialize<'a>")]
pub struct HyraxScalarWrapper<S: HyraxScalar>(pub S);

impl<S: HyraxScalar> Scalar for HyraxScalarWrapper<S> {
    const MAX_SIGNED: Self = Self(S::MAX_SIGNED);

    const ZERO: Self = Self(S::ZERO);

    const ONE: Self = Self(S::ONE);

    const TWO: Self = Self(S::TWO);

    const TEN: Self = Self(S::TEN);
}

impl<S: HyraxScalar> TryFrom<BigInt> for HyraxScalarWrapper<S> {
    type Error = ScalarConversionError;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl<S: HyraxScalar> VarInt for HyraxScalarWrapper<S> {
    fn required_space(self) -> usize {
        self.0.required_space()
    }

    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        S::decode_var(src).map(|(s, u)| (Self(s), u))
    }

    fn encode_var(self, src: &mut [u8]) -> usize {
        S::encode_var(self.0, src)
    }
}

impl<S: HyraxScalar> Inv for HyraxScalarWrapper<S> {
    type Output = Option<Self>;

    fn inv(self) -> Self::Output {
        S::inv(self.0).map(|s| Self(s))
    }
}

impl<S: HyraxScalar> Zero for HyraxScalarWrapper<S> {
    fn zero() -> Self {
        Self(S::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<S: HyraxScalar> One for HyraxScalarWrapper<S> {
    fn one() -> Self {
        Self(S::one())
    }
}

impl<S: HyraxScalar> Mul for HyraxScalarWrapper<S> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl<S: HyraxScalar> MulAssign for HyraxScalarWrapper<S> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl<S: HyraxScalar> UniformRand for HyraxScalarWrapper<S> {
    fn rand<R: ark_std::rand::Rng + ?Sized>(rng: &mut R) -> Self {
        Self(S::rand(rng))
    }
}

impl<S: HyraxScalar> From<&HyraxScalarWrapper<S>> for HyraxScalarWrapper<S> {
    fn from(value: &HyraxScalarWrapper<S>) -> Self {
        Self(value.0)
    }
}

impl<'a, S: HyraxScalar> From<&'a HyraxScalarWrapper<S>> for [u64; 4] {
    fn from(value: &'a HyraxScalarWrapper<S>) -> [u64; 4] {
        value.0.into()
    }
}

macro_rules! impl_from_type {
    ($from_type:ty) => {
        impl<S: HyraxScalar> From<$from_type> for HyraxScalarWrapper<S> {
            fn from(value: $from_type) -> Self {
                Self(value.into())
            }
        }
    };
}

macro_rules! impl_from_wrapper {
    ($from_type:ty) => {
        impl<S: HyraxScalar> From<HyraxScalarWrapper<S>> for $from_type {
            fn from(value: HyraxScalarWrapper<S>) -> $from_type {
                value.0.into()
            }
        }
    };
}

macro_rules! impl_from_with_lifetime {
    ($from_type:ty) => {
        impl<'a, S: HyraxScalar> From<$from_type> for HyraxScalarWrapper<S> {
            fn from(value: $from_type) -> Self {
                Self(value.into())
            }
        }
    };
}

macro_rules! impl_try_into {
    ($from_type:ty) => {
        impl<S: HyraxScalar> TryInto<$from_type> for HyraxScalarWrapper<S> {
            type Error = <S as TryInto<$from_type>>::Error;

            fn try_into(self) -> Result<$from_type, Self::Error> {
                self.0.try_into()
            }
        }
    };
}

impl_from_type!(bool);
impl_from_type!(i8);
impl_from_type!(i16);
impl_from_type!(i32);
impl_from_type!(i64);
impl_from_type!(i128);
impl_from_type!(String);
impl_from_type!(&String);
impl_from_type!([u64; 4]);
impl_from_with_lifetime!(&'a u8);
impl_from_with_lifetime!(&'a i128);
impl_from_with_lifetime!(&'a i64);
impl_from_with_lifetime!(&'a i32);
impl_from_with_lifetime!(&'a i16);
impl_from_with_lifetime!(&'a i8);
impl_from_with_lifetime!(&'a bool);
impl_from_with_lifetime!(&'a str);
impl_from_wrapper!(BigInt);
impl_from_wrapper!([u64; 4]);
impl_try_into!(i128);
impl_try_into!(i64);
impl_try_into!(i32);
impl_try_into!(i16);
impl_try_into!(i8);
impl_try_into!(bool);
