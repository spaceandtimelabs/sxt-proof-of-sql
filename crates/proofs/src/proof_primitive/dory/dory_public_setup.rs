use super::{PublicParameters, VerifierSetup};

/// The public setup required for the Dory PCS by the prover and the commitment computation.
#[derive(Clone)]
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

    #[cfg(any(test, feature = "test"))]
    /// Create a random public setup for the Dory PCS.
    pub fn rand<R>(max_nu: usize, sigma: usize, rng: &mut R) -> Self
    where
        R: ark_std::rand::Rng + ?Sized,
    {
        Self::new(PublicParameters::rand(max_nu, rng), sigma)
    }
}

/// The verifier's public setup for the Dory PCS.
pub struct DoryVerifierPublicSetup {
    verifier_setup: VerifierSetup,
    sigma: usize,
}
impl DoryVerifierPublicSetup {
    /// Create a new public setup for the Dory PCS.
    /// verifier_setup: The verifier's setup parameters for the Dory protocol.
    /// sigma: A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    pub fn new(verifier_setup: VerifierSetup, sigma: usize) -> Self {
        Self {
            verifier_setup,
            sigma,
        }
    }
    /// Returns sigma. A commitment with this setup is a matrix commitment with `1<<sigma` columns.
    pub fn sigma(&self) -> usize {
        self.sigma
    }
    /// The verifier's setup parameters for the Dory protocol.
    pub fn verifier_setup(&self) -> &VerifierSetup {
        &self.verifier_setup
    }
}
impl From<&DoryProverPublicSetup> for DoryVerifierPublicSetup {
    fn from(prover_setup: &DoryProverPublicSetup) -> Self {
        Self {
            verifier_setup: prover_setup.public_parameters().into(),
            sigma: prover_setup.sigma(),
        }
    }
}
