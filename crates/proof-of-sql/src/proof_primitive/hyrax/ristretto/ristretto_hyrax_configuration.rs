use crate::{
    base::scalar::Curve25519Scalar,
    proof_primitive::hyrax::base::hyrax_configuration::HyraxConfiguration,
};
use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint};

#[derive(Default, Clone, Eq, Debug, PartialEq)]
pub struct RistrettoHyraxConfiguration {}

impl HyraxConfiguration for RistrettoHyraxConfiguration {
    type OperableGroup = RistrettoPoint;

    type OperableScalar = Curve25519Scalar;

    type CompressedGroup = CompressedRistretto;

    fn from_operable_to_compressed(
        operable_element: &Self::OperableGroup,
    ) -> Self::CompressedGroup {
        operable_element.compress()
    }

    fn from_compressed_to_operable(
        compressed_element: &Self::CompressedGroup,
    ) -> Self::OperableGroup {
        compressed_element.decompress().unwrap()
    }

    fn compressed_to_bytes(compressed_element: &Self::CompressedGroup) -> [u8; 32] {
        compressed_element.to_bytes()
    }
}
