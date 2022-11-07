mod byte_slice;
#[cfg(test)]
mod byte_slice_test;
pub use byte_slice::as_byte_slice;

mod commitment_utility;
pub use commitment_utility::compute_commitment_for_testing;

mod inner_product;
#[cfg(test)]
mod inner_product_test;
pub use inner_product::inner_product;

mod into_scalar;
pub use into_scalar::IntoScalar;
