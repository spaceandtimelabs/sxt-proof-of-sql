mod ark_scalar;
#[cfg(test)]
mod ark_scalar_test;
pub use ark_scalar::ArkScalar;
mod ark_scalar_from;
#[cfg(test)]
mod ark_scalar_from_test;

#[cfg(any(test, feature = "test"))]
mod commitment_utility;
#[cfg(any(test, feature = "test"))]
pub use commitment_utility::compute_commitment_for_testing;
