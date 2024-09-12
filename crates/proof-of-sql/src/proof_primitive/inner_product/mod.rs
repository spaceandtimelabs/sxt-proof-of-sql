mod inner_product_proof;
pub use inner_product_proof::InnerProductProof;
/// Provides trait implementations for Curve25519Scalar
pub mod curve_25519_scalar;
#[cfg(test)]
mod curve_25519_scalar_tests;
/// Provides trait implementations for RistrettoPoint
pub mod ristretto_point;

#[cfg(test)]
mod inner_product_proof_tests;
