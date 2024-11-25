use super::hyrax_scalar::HyraxScalar;
use core::{
    fmt::Debug,
    iter::Sum,
    ops::{Add, AddAssign, Mul, Neg, Sub},
};
use serde::{Deserialize, Serialize};

pub trait HyraxConfiguration: Debug + Eq + Clone + Default + Send + Sync {
    type OperableGroup: Default
        + Clone
        + Copy
        + Mul<Self::OperableScalar, Output = Self::OperableGroup>
        + AddAssign
        + Sub<Output = Self::OperableGroup>
        + Add<Output = Self::OperableGroup>
        + Eq
        + Debug
        + Neg<Output = Self::OperableGroup>
        + Sum
        + Send
        + Sync;
    type OperableScalar: HyraxScalar + for<'a> Deserialize<'a>;
    type CompressedGroup: Serialize
        + for<'a> Deserialize<'a>
        + Debug
        + Eq
        + Clone
        + Default
        + Send
        + Sync
        + Copy;

    fn from_operable_to_compressed(operable_element: &Self::OperableGroup)
        -> Self::CompressedGroup;

    fn from_compressed_to_operable(
        compressed_element: &Self::CompressedGroup,
    ) -> Self::OperableGroup;
}

#[cfg(test)]
pub(crate) mod generic_hyrax_configuration_tests {
    use super::HyraxConfiguration;

    pub fn we_can_convert_default_values_between_group_representations<C: HyraxConfiguration>() {
        // ARRANGE
        let default_compressed = C::CompressedGroup::default();
        let default_operable = C::OperableGroup::default();

        // ACT
        let operable_from_default_compressed = C::from_compressed_to_operable(&default_compressed);
        let compressed_from_default_operable = C::from_operable_to_compressed(&default_operable);

        // ASSERT
        assert_eq!(default_compressed, compressed_from_default_operable);
        assert_eq!(default_operable, operable_from_default_compressed);
    }

    pub fn from_compressed_to_operable_and_from_operable_to_compressed_are_inverses<
        C: HyraxConfiguration,
    >() {
        // ARRANGE
        let scalar_1 = C::OperableScalar::from(100);
        let scalar_2 = C::OperableScalar::from(314);
        let operable = C::OperableGroup::default() * scalar_1;
        let compressed = C::from_operable_to_compressed(&(C::OperableGroup::default() * scalar_2));

        // ACT
        let operable_from_compressed_from_operable =
            C::from_compressed_to_operable(&C::from_operable_to_compressed(&operable));
        let compressed_from_operable_from_compressed =
            C::from_operable_to_compressed(&C::from_compressed_to_operable(&compressed));

        // ASSERT
        assert_eq!(operable, operable_from_compressed_from_operable);
        assert_eq!(compressed, compressed_from_operable_from_compressed);
    }
}
