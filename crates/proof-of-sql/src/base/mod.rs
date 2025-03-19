//! This module contains basic shared functionalities of the library.
/// TODO: add docs
#[cfg(feature = "arrow")]
pub mod arrow;

pub(crate) mod bit;
mod byte;
pub mod commitment;
pub mod database;
/// TODO: add docs
pub(crate) mod encode;
pub mod math;
/// TODO: add docs
pub(crate) mod polynomial;
/// Module for Proof of SQL datetime types.
pub mod posql_time;
pub(crate) mod proof;
pub(crate) mod ref_into;
/// This module contains the `Scalar` trait as well as the main, generic, implementations of it.
pub mod scalar;
mod serialize;
pub(crate) use serialize::{impl_serde_for_ark_serde_checked, impl_serde_for_ark_serde_unchecked};
pub(crate) mod map;
pub(crate) mod slice_ops;

mod rayon_cfg;
pub(crate) use rayon_cfg::if_rayon;
