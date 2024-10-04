#[cfg(test)]
use super::ProverSetup;
use super::{DeferredGT, G1Affine, G2Affine};
use alloc::vec::Vec;
#[cfg(test)]
use ark_ec::pairing::Pairing;

/// The state of the prover during the Dory proof generation.
/// This is essentially the witness, which only the prover knows.
/// See the beginning of section 3 of https://eprint.iacr.org/2020/1274.pdf for details.
pub struct ProverState {
    /// The vector of G1 elements in the witness. This will be mutated during the proof generation.
    pub(super) v1: Vec<G1Affine>,
    /// The vector of G2 elements in the witness. This will be mutated during the proof generation.
    pub(super) v2: Vec<G2Affine>,
    /// The round number of the proof. The length of `v1` and `v2` should always be 2^nu. This will be mutated during the proof generation.
    pub(super) nu: usize,
}

impl ProverState {
    /// Create a new `ProverState` from the witness.
    /// # Panics
    /// Panics if the length of `v1` is not equal to `2^nu`.
    /// Panics if the length of `v2` is not equal to `2^nu`.
    pub fn new(v1: Vec<G1Affine>, v2: Vec<G2Affine>, nu: usize) -> Self {
        assert_eq!(v1.len(), 1 << nu);
        assert_eq!(v2.len(), 1 << nu);
        ProverState { v1, v2, nu }
    }
    /// Calculate the verifier state from the prover state and setup information.
    /// This is basically the commitment computation of the witness.
    /// See the beginning of section 3 of https://eprint.iacr.org/2020/1274.pdf for details.
    #[cfg(test)]
    pub fn calculate_verifier_state(&self, setup: &ProverSetup) -> VerifierState {
        assert!(setup.max_nu >= self.nu);
        let C = Pairing::multi_pairing(&self.v1, &self.v2).into();
        let D_1 = Pairing::multi_pairing(&self.v1, setup.Gamma_2[self.nu]).into();
        let D_2 = Pairing::multi_pairing(setup.Gamma_1[self.nu], &self.v2).into();
        VerifierState {
            C,
            D_1,
            D_2,
            nu: self.nu,
        }
    }
}

/// The state of the verifier during the Dory proof verification.
/// This is initially created from a type of commitment to the witness, which the prover typically sends to the verifier.
/// This is essentially what the verifier is trying to verify.
/// See the beginning of section 3 of https://eprint.iacr.org/2020/1274.pdf for details.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct VerifierState {
    /// The inner pairing product of the witness. This should be <v1,v2>. This will be mutated during the proof verification.
    pub(super) C: DeferredGT,
    /// The "commitment" to v1. This should be <v1,Γ_2>. This will be mutated during the proof verification.
    pub(super) D_1: DeferredGT,
    /// The "commitment" to v2. This should be <Γ_1,v2>. This will be mutated during the proof verification.
    pub(super) D_2: DeferredGT,
    /// The round number of the proof. The length of `v1` and `v2` should always be 2^nu. This will be mutated during the proof verification.
    pub(super) nu: usize,
}

impl VerifierState {
    /// Create a new `VerifierState` from the commitment to the witness.
    pub fn new(C: DeferredGT, D_1: DeferredGT, D_2: DeferredGT, nu: usize) -> Self {
        VerifierState { C, D_1, D_2, nu }
    }
}
