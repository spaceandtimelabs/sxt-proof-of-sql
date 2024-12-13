use crate::{
    base::scalar::test_scalar::TestScalar,
    proof_primitive::hyrax::base::hyrax_scalar::{HyraxScalar, HyraxScalarWrapper},
};
use ark_ff::PrimeField;
use core::ops::Mul;
use curve25519_dalek::RistrettoPoint;

pub type TestHyraxScalar = HyraxScalarWrapper<TestScalar>;

impl HyraxScalar for TestScalar {}

impl Mul<TestScalar> for RistrettoPoint {
    type Output = RistrettoPoint;

    fn mul(self, rhs: TestScalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from_canonical_bytes(
            ark_ff::BigInteger::to_bytes_le(&rhs.0.into_bigint())
                .try_into()
                .unwrap(),
        )
        .unwrap()
    }
}
