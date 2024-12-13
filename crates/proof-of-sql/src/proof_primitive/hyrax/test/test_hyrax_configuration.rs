use crate::{
    base::scalar::test_scalar::TestScalar,
    proof_primitive::hyrax::base::hyrax_configuration::HyraxConfiguration,
};
use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint};

#[derive(Default, Clone, Eq, Debug, PartialEq)]
pub struct TestHyraxConfiguration {}

impl HyraxConfiguration for TestHyraxConfiguration {
    type OperableGroup = RistrettoPoint;

    type OperableScalar = TestScalar;

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
