use super::{
    DeferredG1, DeferredG2, DeferredGT, G1Affine, G2Affine, ProverState, VerifierState, F,
};
#[cfg(test)]
use super::{G1Projective, G2Projective, ProverSetup};
#[cfg(test)]
use ark_ec::VariableBaseMSM;

/// The state of the prover during the Dory proof generation with the extended algorithm.
/// `base_state` is the state of the prover during the Dory proof generation with the original algorithm.
/// See the beginning of section 4 of https://eprint.iacr.org/2020/1274.pdf for details.
pub struct ExtendedProverState {
    /// The state of the prover during the Dory proof generation with the original algorithm.
    pub(super) base_state: ProverState,
    /// The first vector of F elements in the witness. This will be mutated during the proof generation.
    pub(super) s1: Vec<F>,
    /// The second vector of F elements in the witness. This will be mutated during the proof generation.
    pub(super) s2: Vec<F>,
}

impl ExtendedProverState {
    /// Create a new `ExtendedProverState` from the witness.
    pub fn new(s1: Vec<F>, s2: Vec<F>, v1: Vec<G1Affine>, v2: Vec<G2Affine>, nu: usize) -> Self {
        assert_eq!(s1.len(), 1 << nu);
        assert_eq!(s2.len(), 1 << nu);
        ExtendedProverState {
            base_state: ProverState::new(v1, v2, nu),
            s1,
            s2,
        }
    }
    /// Calculate the verifier state from the prover state and setup information.
    /// This is basically the commitment computation of the witness.
    /// See the beginning of section 4 of https://eprint.iacr.org/2020/1274.pdf for details.
    #[cfg(test)]
    pub fn calculate_verifier_state(&self, setup: &ProverSetup) -> ExtendedVerifierState {
        let E_1: G1Affine = G1Projective::msm_unchecked(&self.base_state.v1, &self.s2).into();
        let E_2: G2Affine = G2Projective::msm_unchecked(&self.base_state.v2, &self.s1).into();
        ExtendedVerifierState {
            base_state: self.base_state.calculate_verifier_state(setup),
            E_1: E_1.into(),
            E_2: E_2.into(),
            s1: self.s1.clone(),
            s2: self.s2.clone(),
        }
    }
}

/// The state of the verifier during the Dory proof verification with the extended algorithm.
/// `base_state` is the state of the verifier during the Dory proof verification with the original algorithm.
/// See the beginning of section 4 of https://eprint.iacr.org/2020/1274.pdf for details.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct ExtendedVerifierState {
    /// The state of the verifier during the Dory proof verification with the original algorithm.
    pub(super) base_state: super::VerifierState,
    /// The "commitment" to s1. This should be <v1,s2>. This will be mutated during the proof verification.
    pub(super) E_1: DeferredG1,
    /// The "commitment" to s2. This should be <s1,v2>. This will be mutated during the proof verification.
    pub(super) E_2: DeferredG2,
    /// The first vector of F elements in the witness. This will be mutated during the proof verification.
    pub(super) s1: Vec<F>,
    /// The second vector of F elements in the witness. This will be mutated during the proof verification.
    pub(super) s2: Vec<F>,
}

impl ExtendedVerifierState {
    /// Create a new `ExtendedVerifierState` from the commitment to the witness.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        E_1: DeferredG1,
        E_2: DeferredG2,
        s1: Vec<F>,
        s2: Vec<F>,
        C: DeferredGT,
        D_1: DeferredGT,
        D_2: DeferredGT,
        nu: usize,
    ) -> Self {
        ExtendedVerifierState {
            base_state: VerifierState::new(C, D_1, D_2, nu),
            E_1,
            E_2,
            s1,
            s2,
        }
    }
}
