//! TODO: add docs
pub mod dory;
/// Central location for any code that requires the use of a dynamic matrix (for now, hyrax and dynamic dory).
pub(super) mod dynamic_matrix_utils;
/// The hyrax evaluation scheme, as outlined here: <https://eprint.iacr.org/2017/1132>
pub mod hyrax;
/// TODO: add docs
pub(crate) mod sumcheck;

/// An implementation of hyper-kzg PCS. This is a wrapper around nova's hyper-kzg implementation.
#[cfg(feature = "hyperkzg")]
pub mod hyperkzg;
