use crate::base::{
    math::{log2_up, log2_up_bytes, BitDecompose},
    proof::{Commit, Commitment},
    scalar::IntoScalar,
};
use std::{
    cmp::Ordering,
    ops::{Add, Mul, Neg, Sub},
};

use num_traits::{Bounded, CheckedAdd, CheckedMul, CheckedNeg, CheckedSub, One, Pow, Zero};

use curve25519_dalek::scalar::Scalar;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SafeIntError {
    #[error("value out of range (must be betweeen -2^251 and 2^251")]
    OutOfRange,
    #[error("potential overflow, add a range check upstream")]
    PotentialOverflow,
    #[error("value exceeds provided log_max, which should always overestimate the actual value")]
    ValueExceedsLogMax,
    #[error("inappropriate log_max reduction attempted")]
    InappropriateLogMaxReduction,
}

type Result<T> = std::result::Result<T, SafeIntError>;

/// Integer type for [Scalar] arithmetic without wrapping.
///
/// Based on the "safe integers" defined in [this document](https://github.com/spaceandtimelabs/proofs/blob/5c8c224ad3f76958a7b57d256f2fc183318151fa/protocols/data_types.md#integer-types).
#[derive(Copy, Clone, Default, Debug)]
pub struct SafeInt {
    value: Scalar,
    log_max: u8,
}

impl SafeInt {
    /// The log-base-2 representation of this type's maximum value.
    /// Arithmetic operations check against this value to prevent wrapping.
    pub const LOG_MAX_MAX: u8 = 251;

    /// Constructor that allows setting a custom log_max.
    /// Will error if the log_max is below the log of the value.
    /// Good for constructing a SafeInt that overestimates the log_max.
    pub fn try_new(value: Scalar, log_max: u8) -> Result<SafeInt> {
        let converted = SafeInt::try_from(value)?;

        if converted.log_max <= log_max {
            Ok(SafeInt {
                log_max,
                ..converted
            })
        } else {
            Err(SafeIntError::ValueExceedsLogMax)
        }
    }

    /// Getter for the internal value as a [Scalar].
    /// Public access to this value is prevented to avoid unsafe mutation.
    pub fn value(&self) -> Scalar {
        self.value
    }

    /// Fallible addition method.
    /// Errors if the new log_max exceeds [Self::LOG_MAX_MAX], indicating a potential overflow.
    ///
    /// The normal addition operation will panic in this situation instead of erroring.
    pub fn try_add(self, rhs: Self) -> Result<Self> {
        match self.log_max.max(rhs.log_max).checked_add(1) {
            Some(log_max) if log_max <= Self::LOG_MAX_MAX => Ok(SafeInt {
                value: self.value + rhs.value,
                log_max,
            }),
            _ => Err(SafeIntError::PotentialOverflow),
        }
    }

    /// Fallible multiplication method.
    /// Errors if the new log_max exceeds [Self::LOG_MAX_MAX], indicating a potential overflow.
    ///
    /// The normal multiplication operation will panic in this situation instead of erroring.
    pub fn try_mul(self, rhs: Self) -> Result<Self> {
        match self.log_max.checked_add(rhs.log_max) {
            Some(log_max) if log_max <= Self::LOG_MAX_MAX => Ok(SafeInt {
                value: self.value * rhs.value,
                log_max,
            }),
            _ => Err(SafeIntError::PotentialOverflow),
        }
    }

    /// Fallible subtraction method.
    /// Errors if the new log_max exceeds [Self::LOG_MAX_MAX], indicating a potential overflow.
    ///
    /// The normal subtraction operation will panic in this situation instead of erroring.
    pub fn try_sub(self, rhs: Self) -> Result<Self> {
        self.try_add(-rhs)
    }

    /// Getter for the internal log_max.
    /// The log_max keeps track of the theoretical maximum value for the SafeInt.
    /// This is an overestimate, and is stored logarithmically to save space.
    /// It is used internally during arithmetic operations to prevent wrapping.
    /// Public access to this value is prevented to avoid unsafe mutation.
    pub fn log_max(&self) -> u8 {
        self.log_max
    }

    /// Returns true if the value is greater than 0, otherwise false.
    ///
    /// In the future, we should accomplish this via [num_traits::Signed]
    pub fn geq_zero(&self) -> bool {
        let log_max = log2_up_bytes(self.value.as_bytes());
        log_max <= SafeInt::LOG_MAX_MAX as usize
    }

    /// Fallible exponentiation method.
    /// Errors if the new log_max exceeds [Self::LOG_MAX_MAX], indicating a potential overflow.
    ///
    /// The normal pow method will panic in this situation instead of erroring.
    pub fn try_pow(self, exp: u8) -> Result<Self> {
        let mut result = SafeInt::one();
        for _ in 0..exp {
            result = result.try_mul(self)?;
        }

        Ok(result)
    }

    /// Returns the [SafeInt], but with the `log_max` value increased by `log_max_addend`.
    ///
    /// `log_max` increases do not need to be checked against the [SafeInt]'s value since the type
    /// guarantees that the the value didn't exceed the previous `log_max`.
    pub fn increment_log_max(self, log_max_addend: u8) -> Result<Self> {
        match self.log_max.checked_add(log_max_addend) {
            Some(log_max) if log_max <= Self::LOG_MAX_MAX => Ok(SafeInt { log_max, ..self }),
            _ => Err(SafeIntError::PotentialOverflow),
        }
    }

    /// Returns the [SafeInt], but with the `log_max` value overwritten.
    ///
    /// Currently, this only intended to be used for increasing the `log_max` value.
    /// Attempts to decrease will result in an error.
    pub fn with_log_max(self, log_max: u8) -> Result<Self> {
        if log_max >= self.log_max {
            let log_max_addend = log_max - self.log_max;
            Ok(self.increment_log_max(log_max_addend)?)
        } else {
            Err(SafeIntError::InappropriateLogMaxReduction)
        }
    }
}

impl Add for SafeInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.try_add(rhs).unwrap()
    }
}

impl CheckedAdd for SafeInt {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        self.try_add(*v).ok()
    }
}

impl Mul for SafeInt {
    type Output = Self;

    fn mul(self, rhs: SafeInt) -> Self {
        self.try_mul(rhs).unwrap()
    }
}

impl CheckedMul for SafeInt {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        self.try_mul(*v).ok()
    }
}

impl Neg for SafeInt {
    type Output = Self;

    fn neg(self) -> Self {
        SafeInt {
            value: -self.value,
            log_max: self.log_max,
        }
    }
}

impl CheckedNeg for SafeInt {
    fn checked_neg(&self) -> Option<Self> {
        Some(-*self)
    }
}

impl Sub for SafeInt {
    type Output = Self;

    fn sub(self, rhs: SafeInt) -> Self {
        self.try_sub(rhs).unwrap()
    }
}

impl CheckedSub for SafeInt {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.try_sub(*v).ok()
    }
}

impl Pow<u8> for SafeInt {
    type Output = Self;

    fn pow(self, exp: u8) -> Self {
        self.try_pow(exp).unwrap()
    }
}

impl Bounded for SafeInt {
    fn min_value() -> Self {
        -Self::max_value()
    }

    fn max_value() -> Self {
        SafeInt::try_from(Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 8,
        ]))
        .expect("SafeInt's hardcoded maximum value should be valid")
    }
}

impl Zero for SafeInt {
    fn zero() -> Self {
        SafeInt::from(0)
    }

    fn is_zero(&self) -> bool {
        self.value == Scalar::zero()
    }
}

impl One for SafeInt {
    fn one() -> Self {
        SafeInt::from(1)
    }
}

impl BitDecompose for SafeInt {
    fn bits(&self) -> Vec<bool> {
        self.value
            .to_bytes()
            .iter()
            .flat_map(|b| b.bits().into_iter().chain(std::iter::repeat(false)).take(8))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .skip_while(|b| !b)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}

impl PartialEq for SafeInt {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SafeInt {}

impl Ord for SafeInt {
    fn cmp(&self, other: &SafeInt) -> Ordering {
        match (self.geq_zero(), other.geq_zero()) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => {
                // Since Ord isn't implemented for Scalar, we need to compare the bytes
                // We find the largest byte that differs and then compare the two
                if let Some((lhs, rhs)) = self
                    .value
                    .to_bytes()
                    .iter()
                    .rev()
                    .zip(other.value.to_bytes().iter().rev())
                    .find(|(l, r)| l != r)
                {
                    lhs.cmp(rhs)
                } else {
                    // No bytes differ, so they must be equal
                    Ordering::Equal
                }
            }
        }
    }
}

impl PartialOrd for SafeInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

macro_rules! safe_int_from_int {
    ($int_type:ty) => {
        impl From<$int_type> for SafeInt {
            fn from(data: $int_type) -> SafeInt {
                let value = if data >= 0 {
                    Scalar::from(data as u64)
                } else {
                    -Scalar::from(-data as u64)
                };

                let log_max = if data == 0 {
                    0
                } else {
                    log2_up(data.unsigned_abs()) as u8
                };

                SafeInt { value, log_max }
            }
        }
    };
}

safe_int_from_int!(i8);
safe_int_from_int!(i16);
safe_int_from_int!(i32);
safe_int_from_int!(i64);
safe_int_from_int!(i128);

impl TryFrom<Scalar> for SafeInt {
    type Error = SafeIntError;

    fn try_from(value: Scalar) -> Result<SafeInt> {
        // Try positive
        let log_max = log2_up_bytes(value.as_bytes());
        if log_max <= SafeInt::LOG_MAX_MAX as usize {
            return Ok(SafeInt {
                value,
                log_max: log_max as u8,
            });
        }

        // Try negative
        let log_max = log2_up_bytes((-value).as_bytes());
        if log_max <= SafeInt::LOG_MAX_MAX as usize {
            return Ok(SafeInt {
                value,
                log_max: log_max as u8,
            });
        }

        Err(SafeIntError::OutOfRange)
    }
}

impl From<SafeInt> for Scalar {
    fn from(data: SafeInt) -> Scalar {
        data.value
    }
}

impl IntoScalar for SafeInt {
    fn into_scalar(self) -> Scalar {
        self.into()
    }
}

/// Collection of [SafeInt]s with a shared `log_max`.
///
/// Takes up less memory than storing a simple `Vec<SafeInt>`, with the limitation that their
/// `log_max` value will be unified.
/// While this can lead to overestimating, it may be preferable if you only care about the
/// overall `log_max` of an entire column of `SafeInt`s.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct SafeIntColumn {
    values: Vec<Scalar>,
    log_max: u8,
}

impl SafeIntColumn {
    /// Returns the [SafeInt] at the provided index or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<SafeInt> {
        self.values.get(index).map(|&value| SafeInt {
            value,
            log_max: self.log_max,
        })
    }

    /// Constructor that allows setting a custom log_max.
    /// Will error if the log_max is below the log of the value.
    /// Good for constructing a SafeIntColumn that overestimates the log_max.
    pub fn try_new(values: Vec<Scalar>, log_max: u8) -> Result<SafeIntColumn> {
        let converted: SafeIntColumn = values
            .into_iter()
            .map(SafeInt::try_from)
            .collect::<Result<SafeIntColumn>>()?;

        if converted.log_max <= log_max {
            Ok(SafeIntColumn {
                log_max,
                ..converted
            })
        } else {
            Err(SafeIntError::ValueExceedsLogMax)
        }
    }

    /// Getter for the internal values as a vec of [Scalar]s.
    /// Public access to this value is prevented to avoid unsafe mutation.
    pub fn values(&self) -> &Vec<Scalar> {
        self.values.as_ref()
    }

    /// Getter for the internal log_max.
    /// The log_max keeps track of the theoretical maximum value for the SafeInts in the column.
    /// This is an overestimate, and is stored logarithmically to save space.
    /// It is used internally during arithmetic operations to prevent wrapping.
    /// Public access to this value is prevented to avoid unsafe mutation.
    pub fn log_max(&self) -> u8 {
        self.log_max
    }

    /// Returns the number of elements in the column.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the column contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the [SafeIntColumn], but with the `log_max` value increased by `log_max_addend`.
    ///
    /// `log_max` increases do not need to be checked against the [SafeIntColumn]'s values since
    /// the type guarantees that the the values didn't exceed the previous `log_max`.
    pub fn increment_log_max(self, log_max_addend: u8) -> Result<Self> {
        match self.log_max.checked_add(log_max_addend) {
            Some(log_max) if log_max <= SafeInt::LOG_MAX_MAX => {
                Ok(SafeIntColumn { log_max, ..self })
            }
            _ => Err(SafeIntError::PotentialOverflow),
        }
    }

    /// Returns the [SafeIntColumn], but with the `log_max` value overwritten.
    ///
    /// Currently, this only intended to be used for increasing the `log_max` value.
    /// Attempts to decrease will result in an error.
    pub fn with_log_max(self, log_max: u8) -> Result<Self> {
        if log_max >= self.log_max {
            let log_max_addend = log_max - self.log_max;
            Ok(self.increment_log_max(log_max_addend)?)
        } else {
            Err(SafeIntError::InappropriateLogMaxReduction)
        }
    }
}

/// An iterator that moves out of a [SafeIntColumn].
///
/// Created by the `into_iter` method on [SafeIntColumn].
pub struct IntoIter {
    iter: std::vec::IntoIter<Scalar>,
    log_max: u8,
}

impl Iterator for IntoIter {
    type Item = SafeInt;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|value| SafeInt {
            value,
            log_max: self.log_max,
        })
    }
}

impl IntoIterator for SafeIntColumn {
    type Item = SafeInt;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.values.into_iter(),
            log_max: self.log_max,
        }
    }
}

impl FromIterator<SafeInt> for SafeIntColumn {
    fn from_iter<T: IntoIterator<Item = SafeInt>>(iter: T) -> Self {
        let mut log_max = 0;
        let values = iter
            .into_iter()
            .map(|SafeInt { value, log_max: m }| {
                log_max = log_max.max(m);
                value
            })
            .collect();

        SafeIntColumn { log_max, values }
    }
}

impl<T> From<Vec<T>> for SafeIntColumn
where
    T: Into<SafeInt>,
{
    fn from(data: Vec<T>) -> Self {
        data.into_iter().map(|t| t.into()).collect()
    }
}

impl Commit for SafeIntColumn {
    type Commitment = Commitment;

    fn commit(&self) -> Self::Commitment {
        let log_max = self.log_max;

        let mut commitment: Commitment = self
            .clone()
            .into_iter()
            .map(|s| s.value)
            .collect::<Vec<Scalar>>()
            .as_slice()
            .into();

        commitment.log_max = Some(log_max);
        commitment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This is necessary since SafeInt's Eq implementation doesn't pay attention to log_max
    fn assert_eq_strict(a: SafeInt, b: SafeInt) {
        assert_eq!(a, b);
        assert_eq!(a.log_max, b.log_max);
    }

    fn assert_eq_strict_optional(a: Option<SafeInt>, b: Option<SafeInt>) {
        if let (Some(a), Some(b)) = (a, b) {
            assert_eq_strict(a, b);
        } else {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_addition_zeros_ones() {
        // zeros and ones
        assert_eq_strict(
            SafeInt::from(0) + SafeInt::from(0),
            SafeInt {
                value: Scalar::zero(),
                log_max: 1,
            },
        );
        assert_eq_strict(
            SafeInt::from(1) + SafeInt::from(1),
            SafeInt {
                value: Scalar::from(2u32),
                log_max: 1,
            },
        );
        assert_eq_strict(
            SafeInt::from(0) + SafeInt::from(1),
            SafeInt {
                value: Scalar::from(1u32),
                log_max: 1,
            },
        );
    }

    #[test]
    fn test_addition_log_max_overestimates() {
        assert_eq_strict(
            SafeInt::from(5) + SafeInt::from(1),
            SafeInt {
                value: Scalar::from(6u32),
                log_max: 4,
            },
        );
        assert_eq_strict(
            SafeInt::from(15) + SafeInt::from(17),
            SafeInt {
                value: Scalar::from(32u32),
                log_max: 6,
            },
        );
    }

    #[test]
    fn test_addition_negatives() {
        assert_eq_strict(
            SafeInt::from(15) + SafeInt::from(-17),
            SafeInt {
                value: -Scalar::from(2u32),
                log_max: 6,
            },
        );
        assert_eq_strict(
            SafeInt::from(-2) + SafeInt::from(4),
            SafeInt {
                value: Scalar::from(2u32),
                log_max: 3,
            },
        );
        assert_eq_strict(
            SafeInt::from(-32) + SafeInt::from(-18),
            SafeInt {
                value: -Scalar::from(50u32),
                log_max: 6,
            },
        );
    }

    #[test]
    fn test_addition_maximum_cases() {
        let max_div_2 = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 4,
        ]);
        let max = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 8,
        ]);

        assert_eq_strict(
            SafeInt::try_from(max_div_2).unwrap() + SafeInt::try_from(max_div_2).unwrap(),
            SafeInt {
                value: max,
                log_max: 251,
            },
        );

        assert_eq_strict(
            SafeInt::try_from(-max_div_2).unwrap() + SafeInt::try_from(-max_div_2).unwrap(),
            SafeInt {
                value: -max,
                log_max: 251,
            },
        );
        assert_eq_strict(
            SafeInt::try_from(max_div_2).unwrap() + SafeInt::try_from(-max_div_2).unwrap(),
            SafeInt {
                value: Scalar::zero(),
                log_max: 251,
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_addition_out_of_range() {
        let max = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 8,
        ]);

        // destructuring instead of unwrapping to avoid panicking early
        if let Ok(max) = SafeInt::try_from(max) {
            let _should_panic = max + SafeInt::from(1);
        }
    }

    #[test]
    fn test_mul_zeros_ones() {
        // zeros and ones
        assert_eq_strict(
            SafeInt::from(0) * SafeInt::from(0),
            SafeInt {
                value: Scalar::zero(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::from(1) * SafeInt::from(1),
            SafeInt {
                value: Scalar::from(1u32),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::from(0) * SafeInt::from(1),
            SafeInt {
                value: Scalar::from(0u32),
                log_max: 0,
            },
        );
    }

    #[test]
    fn test_mul_log_max_overestimates() {
        assert_eq_strict(
            SafeInt::from(5) * SafeInt::from(5),
            SafeInt {
                value: Scalar::from(25u32),
                log_max: 6,
            },
        );
        assert_eq_strict(
            SafeInt::from(15) * SafeInt::from(17),
            SafeInt {
                value: Scalar::from(255u32),
                log_max: 9,
            },
        );
    }

    #[test]
    fn test_mul_negatives() {
        assert_eq_strict(
            SafeInt::from(15) * SafeInt::from(-17),
            SafeInt {
                value: -Scalar::from(255u32),
                log_max: 9,
            },
        );
        assert_eq_strict(
            SafeInt::from(-2) * SafeInt::from(4),
            SafeInt {
                value: -Scalar::from(8u32),
                log_max: 3,
            },
        );
        assert_eq_strict(
            SafeInt::from(-32) * SafeInt::from(-18),
            SafeInt {
                value: Scalar::from(576u32),
                log_max: 10,
            },
        );
    }

    #[test]
    fn test_mul_maximum_cases() {
        let max_div_2 = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 4,
        ]);
        let max = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 8,
        ]);

        assert_eq_strict(
            SafeInt::try_from(max_div_2).unwrap() * SafeInt::from(2),
            SafeInt {
                value: max,
                log_max: 251,
            },
        );

        assert_eq_strict(
            SafeInt::try_from(-max_div_2).unwrap() * SafeInt::try_from(-2).unwrap(),
            SafeInt {
                value: max,
                log_max: 251,
            },
        );
        assert_eq_strict(
            SafeInt::try_from(max_div_2).unwrap() * SafeInt::try_from(-2).unwrap(),
            SafeInt {
                value: -max,
                log_max: 251,
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_mul_out_of_range() {
        let max_div_2_plus_1 = Scalar::from_bytes_mod_order([
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 4,
        ]);

        // destructuring instead of unwrapping to avoid panicking early
        if let Ok(max_div_2_plus_1) = SafeInt::try_from(max_div_2_plus_1) {
            let _should_panic = max_div_2_plus_1 * SafeInt::from(2);
        }
    }

    #[test]
    fn test_safe_int_column() {
        let vec_safe_ints = vec![
            SafeInt::from(1),
            SafeInt::from(10),
            SafeInt::from(100),
            SafeInt::from(-100),
            SafeInt::from(-10),
        ];
        let safe_int_col: SafeIntColumn = vec_safe_ints.into_iter().collect();

        // test get
        // log_max should be the maximum log_max of the original SafeInts
        assert_eq_strict_optional(
            safe_int_col.get(1),
            Some(SafeInt {
                value: Scalar::from(10u32),
                log_max: 7,
            }),
        );
        // out of bounds
        assert_eq!(safe_int_col.get(10), None);

        // test iter
        let mut into_iter = safe_int_col.into_iter();

        assert_eq_strict_optional(
            into_iter.next(),
            Some(SafeInt {
                value: Scalar::from(1u32),
                log_max: 7,
            }),
        );
        assert_eq_strict_optional(
            into_iter.next(),
            Some(SafeInt {
                value: Scalar::from(10u32),
                log_max: 7,
            }),
        );
        assert_eq_strict_optional(
            into_iter.next(),
            Some(SafeInt {
                value: Scalar::from(100u32),
                log_max: 7,
            }),
        );
        assert_eq_strict_optional(
            into_iter.next(),
            Some(SafeInt {
                value: -Scalar::from(100u32),
                log_max: 7,
            }),
        );
        assert_eq_strict_optional(
            into_iter.next(),
            Some(SafeInt {
                value: -Scalar::from(10u32),
                log_max: 7,
            }),
        );
        assert_eq!(into_iter.next(), None);
    }

    /// Since Sub just uses Add and Neg internally, testing all the edge cases would be overkill
    #[test]
    fn test_sub() {
        assert_eq_strict(
            SafeInt::from(1) - SafeInt::from(2),
            SafeInt {
                value: -Scalar::from(1u32),
                log_max: 2,
            },
        );
        assert_eq_strict(
            SafeInt::from(100) - SafeInt::from(50),
            SafeInt {
                value: Scalar::from(50u32),
                log_max: 8,
            },
        );
        assert_eq_strict(
            SafeInt::from(-128) - SafeInt::from(256),
            SafeInt {
                value: -Scalar::from(384u32),
                log_max: 9,
            },
        );
    }

    #[test]
    fn test_int_conversion() {
        assert_eq_strict(
            SafeInt::from(0),
            SafeInt {
                value: Scalar::zero(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::from(1),
            SafeInt {
                value: Scalar::one(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::from(100),
            SafeInt {
                value: Scalar::from(100u32),
                log_max: 7,
            },
        );
        assert_eq_strict(
            SafeInt::from(-1000),
            SafeInt {
                value: -Scalar::from(1000u32),
                log_max: 10,
            },
        );
    }

    #[test]
    fn test_scalar_conversion() {
        assert_eq_strict(
            SafeInt::try_from(Scalar::zero()).unwrap(),
            SafeInt {
                value: Scalar::zero(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::try_from(Scalar::one()).unwrap(),
            SafeInt {
                value: Scalar::one(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::try_from(Scalar::from(100u32)).unwrap(),
            SafeInt {
                value: Scalar::from(100u32),
                log_max: 7,
            },
        );
        assert_eq_strict(
            SafeInt::try_from(-Scalar::from(1000u32)).unwrap(),
            SafeInt {
                value: -Scalar::from(1000u32),
                log_max: 10,
            },
        );
    }

    #[test]
    fn test_ord_zeros_ones() {
        assert!(SafeInt::from(-1) < SafeInt::from(0));
        assert!(SafeInt::from(1) > SafeInt::from(0));
        assert!(SafeInt::from(-1) <= SafeInt::from(1));
        assert!(SafeInt::from(1) >= SafeInt::from(1));
        assert!(SafeInt::from(-1) <= SafeInt::from(-1));
    }

    #[test]
    fn test_ord_same_sign() {
        assert!(SafeInt::from(10) < SafeInt::from(100));
        assert!(SafeInt::from(64) > SafeInt::from(32));

        assert!(SafeInt::from(-10) > SafeInt::from(-100));
        assert!(SafeInt::from(-64) < SafeInt::from(-32));
    }

    #[test]
    fn test_ord_maximum() {
        assert!(SafeInt::min_value() < SafeInt::from(0));
        assert!(SafeInt::max_value() > SafeInt::from(0));
        assert!(SafeInt::min_value() <= SafeInt::max_value());
    }

    #[test]
    fn test_pow_zeros_ones() {
        assert_eq_strict(
            SafeInt::zero().pow(4),
            SafeInt {
                value: Scalar::zero(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::one().pow(4),
            SafeInt {
                value: Scalar::one(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::from(-1).pow(6),
            SafeInt {
                value: Scalar::one(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::from(-1).pow(7),
            SafeInt {
                value: -Scalar::one(),
                log_max: 0,
            },
        );
    }

    #[test]
    fn test_pow_powers_of_2() {
        assert_eq_strict(
            SafeInt::from(2).pow(0),
            SafeInt {
                value: Scalar::one(),
                log_max: 0,
            },
        );
        assert_eq_strict(
            SafeInt::from(2).pow(1),
            SafeInt {
                value: Scalar::from(2u32),
                log_max: 1,
            },
        );
        assert_eq_strict(
            SafeInt::from(2).pow(25),
            SafeInt {
                value: Scalar::from(33554432u32),
                log_max: 25,
            },
        )
    }

    #[test]
    fn test_pow_log_max_overestimates() {
        assert_eq_strict(
            SafeInt::from(5).pow(2),
            SafeInt {
                value: Scalar::from(25u32),
                log_max: 6,
            },
        );
        assert_eq_strict(
            SafeInt::from(-17).pow(3),
            SafeInt {
                value: -Scalar::from(4913u32),
                log_max: 15,
            },
        );
    }
}
