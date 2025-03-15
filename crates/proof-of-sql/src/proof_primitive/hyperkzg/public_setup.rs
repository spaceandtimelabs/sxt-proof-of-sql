use ark_bn254::G1Affine;

/// When borrowed, `PublicSetup` type associated with the `HyperKZG` commitment scheme.
///
/// This "Owned" version is occasionally useful when actively allocating the setup to memory.
/// For example, deserialization, or generation.
///
/// See [`HyperKZGPublicSetup`] for the actual associated public setup type.
pub type HyperKZGPublicSetupOwned = alloc::vec::Vec<G1Affine>;

/// `PublicSetup` type associated with the `HyperKZG` commitment scheme.
pub type HyperKZGPublicSetup<'a> = &'a [G1Affine];
