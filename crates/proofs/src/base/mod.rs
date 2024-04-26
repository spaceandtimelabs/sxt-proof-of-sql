//! TODO: add docs
pub mod bit;
pub mod commitment;
pub mod database;
pub mod encode;
pub mod math;
pub mod polynomial;
pub mod proof;
pub mod ref_into;
pub mod scalar;
mod serialize;
pub(crate) use serialize::impl_serde_for_ark_serde;
pub mod slice_ops;
