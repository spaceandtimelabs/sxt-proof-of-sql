use super::hyrax_scalar::HyraxScalar;
use core::{
    fmt::Debug,
    iter::Sum,
    ops::{Add, AddAssign, Mul, Neg, Sub},
};
use serde::{Deserialize, Serialize};

pub trait HyraxConfiguration: Debug + Eq + Clone + Default + Send + Sync {
    /// The group that will be used to perform any mathematical operations.
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
    /// The scalar that will be used.
    type OperableScalar: HyraxScalar + for<'a> Deserialize<'a>;
    /// The compressed representation of the operabe group. Note that decompression occurs primarily in verification,
    /// whereas compression occurs mostly in proof and commitment generation. This is an important deatil for example in the sp1 implementation,
    /// where it is most important to emphasize efficient verification.
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

    fn compressed_to_bytes(compressed_element: &Self::CompressedGroup) -> [u8; 32];
}
