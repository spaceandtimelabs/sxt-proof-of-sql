/// This module contains the definition of the `Scalar` trait, which is used to represent the scalar field used in Proof of SQL.
mod scalar_ext;
pub use scalar_ext::ScalarExt;
mod scalar;
pub use scalar::Scalar;
#[cfg(test)]
pub(crate) use scalar::test_scalar_constants;
mod error;
pub use error::ScalarConversionError;
/// TODO add doc
mod mont_scalar;
#[cfg(test)]
mod mont_scalar_test;
pub use mont_scalar::MontScalar;
/// Module for a test Scalar
#[cfg(test)]
pub mod test_scalar;
#[cfg(test)]
mod test_scalar_test;