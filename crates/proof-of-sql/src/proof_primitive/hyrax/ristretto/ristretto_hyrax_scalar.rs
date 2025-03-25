use crate::{
    base::scalar::Curve25519Scalar,
    proof_primitive::hyrax::base::hyrax_scalar::{HyraxScalar, HyraxScalarWrapper},
};

pub type RistrettoHyraxScalar = HyraxScalarWrapper<Curve25519Scalar>;

impl HyraxScalar for Curve25519Scalar {}
