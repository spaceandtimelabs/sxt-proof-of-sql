use super::{ProverSetup, VerifierSetup};

/// The public setup required for the Dory PCS by the prover and the commitment computation.
#[derive(Clone, Copy)]
pub struct DoryProverPublicSetup<'a> {
    prover_setup: &'a ProverSetup<'a>,
    sigma: usize,
}
impl<'a> DoryProverPublicSetup<'a> {
    /// Create a new public setup for the Dory PCS.
    /// public_parameters: The public parameters for the Dory protocol.
    /// sigma: A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    #[must_use]
    pub fn new(prover_setup: &'a ProverSetup<'a>, sigma: usize) -> Self {
        Self {
            prover_setup,
            sigma,
        }
    }
    /// Returns sigma. A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    #[must_use]
    pub fn sigma(&self) -> usize {
        self.sigma
    }
    /// The public setup for the Dory protocol.
    #[must_use]
    pub fn prover_setup(&self) -> &ProverSetup {
        self.prover_setup
    }
}

/// The verifier's public setup for the Dory PCS.
#[derive(Clone, Copy)]
pub struct DoryVerifierPublicSetup<'a> {
    verifier_setup: &'a VerifierSetup,
    sigma: usize,
}
impl<'a> DoryVerifierPublicSetup<'a> {
    /// Create a new public setup for the Dory PCS.
    /// verifier_setup: The verifier's setup parameters for the Dory protocol.
    /// sigma: A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    #[must_use]
    pub fn new(verifier_setup: &'a VerifierSetup, sigma: usize) -> Self {
        Self {
            verifier_setup,
            sigma,
        }
    }
    /// Returns sigma. A commitment with this setup is a matrix commitment with `1<<sigma` columns.
    #[must_use]
    pub fn sigma(&self) -> usize {
        self.sigma
    }
    /// The verifier's setup parameters for the Dory protocol.
    #[must_use]
    pub fn verifier_setup(&self) -> &VerifierSetup {
        self.verifier_setup
    }
}
