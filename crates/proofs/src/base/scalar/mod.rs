#[cfg(any(test, feature = "test"))]
mod commitment_utility;
#[cfg(any(test, feature = "test"))]
pub use commitment_utility::compute_commitment_for_testing;
