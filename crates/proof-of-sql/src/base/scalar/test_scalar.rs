use crate::base::commitment::naive_commitment::NaiveCommitment;

use super::{test_config::TestMontConfig, MontScalar, Scalar};
use ark_ff::PrimeField;

/// A wrapper type around the field element `ark_curve25519::Fr` and should be used in place of `ark_curve25519::Fr`.
///
/// Using the `Scalar` trait rather than this type is encouraged to allow for easier switching of the underlying field.
pub type TestScalar = MontScalar<TestMontConfig>;

impl core::ops::Mul<NaiveCommitment> for TestScalar {
    type Output = NaiveCommitment;
    fn mul(self, rhs: NaiveCommitment) -> Self::Output {
        NaiveCommitment(rhs.0.iter().map(|s| self + *s).collect())
    }
}
impl core::ops::Mul<TestScalar> for NaiveCommitment {
    type Output = NaiveCommitment;
    fn mul(self, rhs: TestScalar) -> Self::Output {
        NaiveCommitment(self.0.iter().map(|s| rhs + *s).collect())
    }
}
impl core::ops::Mul<&NaiveCommitment> for TestScalar {
    type Output = NaiveCommitment;
    fn mul(self, rhs: &NaiveCommitment) -> Self::Output {
        NaiveCommitment(rhs.0.iter().map(|s| self + *s).collect())
    }
}
impl core::ops::Mul<TestScalar> for &NaiveCommitment {
    type Output = NaiveCommitment;
    fn mul(self, rhs: TestScalar) -> Self::Output {
        NaiveCommitment(self.0.iter().map(|s| rhs + *s).collect())
    }
}
impl From<TestScalar> for curve25519_dalek::scalar::Scalar {
    fn from(value: TestScalar) -> Self {
        (&value).into()
    }
}

impl From<&TestScalar> for curve25519_dalek::scalar::Scalar {
    fn from(value: &TestScalar) -> Self {
        let bytes = ark_ff::BigInteger::to_bytes_le(&value.0.into_bigint());
        curve25519_dalek::scalar::Scalar::from_canonical_bytes(bytes.try_into().unwrap()).unwrap()
    }
}

impl Scalar for TestScalar {
    const MAX_SIGNED: Self = Self(ark_ff::MontFp!(
        "3618502788666131106986593281521497120428558179689953803000975469142727125494"
    ));
    const ZERO: Self = Self(ark_ff::MontFp!("0"));
    const ONE: Self = Self(ark_ff::MontFp!("1"));
    const TWO: Self = Self(ark_ff::MontFp!("2"));
}
