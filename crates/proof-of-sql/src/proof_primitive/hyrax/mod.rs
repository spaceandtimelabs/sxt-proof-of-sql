/// The hyrax scheme in generic form.
pub(super) mod base;
/// A configuration with ristretto point as the group and curve 25519 scalar. For now, this will only be used for testing purposes.
#[cfg(test)]
pub mod ristretto;
