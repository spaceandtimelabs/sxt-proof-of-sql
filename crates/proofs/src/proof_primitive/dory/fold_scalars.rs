use super::{
    extended_state::{ExtendedProverState, ExtendedVerifierState},
    DeferredGT, DoryMessages, ProverSetup, ProverState, VerifierSetup, VerifierState, G1, G2,
};
use ark_ec::pairing::Pairing;
use merlin::Transcript;

/// This is the prover side of the Fold-Scalars algorithm in section 4.1 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Note: this only works for nu = 0.
pub fn fold_scalars_0_prove(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    mut state: ExtendedProverState,
    setup: &ProverSetup,
) -> ProverState {
    assert_eq!(state.base_state.nu, 0);
    let (gamma, gamma_inv) = messages.verifier_F_message(transcript);
    state.base_state.v1[0] += setup.H_1 * state.s1[0] * gamma;
    state.base_state.v2[0] += setup.H_2 * state.s2[0] * gamma_inv;
    state.base_state
}

/// This is the verifier side of the Fold-Scalars algorithm in section 4.1 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Note: this only works for nu = 0.
pub fn fold_scalars_0_verify(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    mut state: ExtendedVerifierState,
    setup: &VerifierSetup,
) -> VerifierState {
    assert_eq!(state.base_state.nu, 0);
    let (gamma, gamma_inv) = messages.verifier_F_message(transcript);
    state.base_state.C += DeferredGT::from(setup.H_T) * state.s1[0] * state.s2[0]
        + DeferredGT::from(Pairing::pairing(setup.H_1, state.E_2.compute::<G2>())) * gamma
        + DeferredGT::from(Pairing::pairing(state.E_1.compute::<G1>(), setup.H_2)) * gamma_inv;
    state.base_state.D_1 += Pairing::pairing(setup.H_1, setup.Gamma_2_0 * state.s1[0] * gamma);
    state.base_state.D_2 += Pairing::pairing(setup.Gamma_1_0 * state.s2[0] * gamma_inv, setup.H_2);
    state.base_state
}
