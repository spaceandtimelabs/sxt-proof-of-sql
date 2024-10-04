use super::{
    extended_state::{ExtendedProverState, ExtendedVerifierState},
    pairings, DeferredGT, DoryMessages, G1Projective, G2Projective, ProverSetup, ProverState,
    VerifierSetup, VerifierState, F,
};
use crate::base::proof::Transcript;

/// This is the prover side of the Fold-Scalars algorithm in section 4.1 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Note: this only works for nu = 0.
#[allow(clippy::missing_panics_doc)]
pub fn fold_scalars_0_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedProverState,
    setup: &ProverSetup,
) -> ProverState {
    assert_eq!(state.base_state.nu, 0);
    let (gamma, gamma_inv) = messages.verifier_F_message(transcript);
    state.base_state.v1[0] = (state.base_state.v1[0] + setup.H_1 * state.s1[0] * gamma).into();
    state.base_state.v2[0] = (state.base_state.v2[0] + setup.H_2 * state.s2[0] * gamma_inv).into();
    state.base_state
}

/// This is the verifier side of the Fold-Scalars algorithm in section 4.1 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Note: this only works for nu = 0.
///
/// See [extended_dory_reduce_verify_fold_s_vecs](super::extended_dory_reduce_helper::extended_dory_reduce_verify_fold_s_vecs)
/// for an explaination of the `s_folded` values
#[tracing::instrument(level = "debug", skip_all)]
pub fn fold_scalars_0_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedVerifierState,
    setup: &VerifierSetup,
    fold_s_tensors_verify: impl Fn(&ExtendedVerifierState) -> (F, F),
) -> VerifierState {
    assert_eq!(state.base_state.nu, 0);
    let (gamma, gamma_inv) = messages.verifier_F_message(transcript);
    let (s1_folded, s2_folded) = fold_s_tensors_verify(&state);
    state.base_state.C += DeferredGT::from(setup.H_T) * s1_folded * s2_folded
        + DeferredGT::from(pairings::pairing(
            setup.H_1,
            state.E_2.compute::<G2Projective>(),
        )) * gamma
        + DeferredGT::from(pairings::pairing(
            state.E_1.compute::<G1Projective>(),
            setup.H_2,
        )) * gamma_inv;
    state.base_state.D_1 += pairings::pairing(setup.H_1, setup.Gamma_2_0 * s1_folded * gamma);
    state.base_state.D_2 += pairings::pairing(setup.Gamma_1_0 * s2_folded * gamma_inv, setup.H_2);
    state.base_state
}
