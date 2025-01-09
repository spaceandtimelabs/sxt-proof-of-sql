/// This module contains the definition of the `Scalar` trait, which is used to represent the scalar field used in Proof of SQL.
mod scalar;
pub use scalar::Scalar;
mod error;
pub use error::ScalarConversionError;
mod mont_scalar;
#[cfg(test)]
mod mont_scalar_test;
pub use mont_scalar::BN254Scalar;
pub use mont_scalar::Curve25519Scalar;
pub(crate) use mont_scalar::MontScalar;
/// Module for a test Scalar
#[cfg(test)]
pub mod test_scalar;
#[cfg(test)]
mod test_scalar_test;

mod scalar_ext;
pub use scalar_ext::ScalarExt;
