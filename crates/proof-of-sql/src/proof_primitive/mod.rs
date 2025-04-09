//! TODO: add docs
pub mod dory;
/// Central location for any code that requires the use of a dynamic matrix (for now, hyrax and dynamic dory).
pub(super) mod dynamic_matrix_utils;
/// The hyrax evaluation scheme, as outlined here: <https://eprint.iacr.org/2017/1132>
pub mod hyrax;
/// TODO: add docs
pub(crate) mod sumcheck;

pub mod hyperkzg;

/// TODO: Add docs
pub mod inner_product;
