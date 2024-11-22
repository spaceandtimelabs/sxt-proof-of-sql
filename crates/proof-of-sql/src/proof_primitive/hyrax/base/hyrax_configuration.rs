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
