use super::PublicParameters;

/// The public setup required for the Dory PCS by the prover and the commitment computation.
pub struct DoryProverPublicSetup {
    public_parameters: PublicParameters,
    sigma: usize,
}
impl DoryProverPublicSetup {
    /// Create a new public setup for the Dory PCS.
    /// public_parameters: The public parameters for the Dory protocol.
    /// sigma: A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    pub fn new(public_parameters: PublicParameters, sigma: usize) -> Self {
        Self {
            public_parameters,
            sigma,
        }
    }
    /// Returns sigma. A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    pub fn sigma(&self) -> usize {
        self.sigma
    }
    /// The public parameters for the Dory protocol.
    pub fn public_parameters(&self) -> &PublicParameters {
        &self.public_parameters
    }

    #[cfg(test)]
    /// Create a random public setup for the Dory PCS.
    pub fn rand<R>(max_nu: usize, sigma: usize, rng: &mut R) -> Self
    where
        R: ark_std::rand::Rng + ?Sized,
    {
        Self::new(PublicParameters::rand(max_nu, rng), sigma)
    }
}
