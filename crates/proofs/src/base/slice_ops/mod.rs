//! This module provide operations for working on slices. Each operation is done as generically as possible to be interopable.
//! When relevent, slices are assumed to extend indefinitely and be filled with zeros.
//! For example, the inner product will not panic when the two input slices have different lengths.
//! Instead, it will simply truncate the longer one, which is equivalent to multiply each extra element by zero before summing.

pub const MIN_RAYON_LEN: usize = 1 << 8;

mod inner_product;
#[cfg(test)]
mod inner_product_test;
mod mul_add_assign;
#[cfg(test)]
mod mul_add_assign_test;
mod slice_cast;
#[cfg(test)]
mod slice_cast_test;

pub use inner_product::*;
pub use mul_add_assign::*;
pub use slice_cast::*;
