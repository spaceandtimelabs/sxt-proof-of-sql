use crate::{
    base::scalar::Curve25519Scalar,
    proof_primitive::hyrax::base::{
        hyrax_configuration::HyraxConfiguration, hyrax_scalar::HyraxScalar,
    },
};
use curve25519_dalek::{edwards::CompressedEdwardsY, EdwardsPoint};

/// The choice of group and scalar that we deem to be the most effective pairing for implementing the Hyrax scheme for sp1.
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct Sp1HyraxConfiguration;

impl HyraxScalar for Curve25519Scalar {}

impl HyraxConfiguration for Sp1HyraxConfiguration {
    type OperableGroup = EdwardsPoint;

    type OperableScalar = Curve25519Scalar;

    type CompressedGroup = CompressedEdwardsY;

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
}

#[cfg(test)]
mod sp1_hyrax_configuration_tests {
    use super::Sp1HyraxConfiguration;
    use crate::proof_primitive::hyrax::base::hyrax_configuration::generic_hyrax_configuration_tests::{from_compressed_to_operable_and_from_operable_to_compressed_are_inverses, we_can_convert_default_values_between_group_representations};

    #[test]
    fn we_can_convert_default_values_between_edwards_point_and_compressed_edwards_y() {
        we_can_convert_default_values_between_group_representations::<Sp1HyraxConfiguration>();
    }

    #[test]
    fn from_compressed_to_operable_and_from_operable_to_compressed_are_inverses_for_sp1() {
        from_compressed_to_operable_and_from_operable_to_compressed_are_inverses::<
            Sp1HyraxConfiguration,
        >();
    }
}
