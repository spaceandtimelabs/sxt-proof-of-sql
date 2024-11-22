/// The commitment scheme for hyrax
pub mod hyrax_commitment;
/// The verification scheme for hyrax
pub mod hyrax_commitment_evaluation_proof;
/// Effectively defines the implementation of the scalar, commitment, and evaluation proof by providing the scalar and group.
pub mod hyrax_configuration;
/// Any helper code. Ideally this will be combined with dynamic dory helper code.
pub mod hyrax_helpers;
/// The group generators.
pub mod hyrax_public_setup;
/// A wrapper for the scalar. This might not need to be a wrapper. The purpose of this is to allow the implementation of certain traits on a generic type.
pub mod hyrax_scalar;
