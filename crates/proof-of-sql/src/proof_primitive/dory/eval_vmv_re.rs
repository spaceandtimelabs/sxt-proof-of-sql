use super::{
    pairings, DeferredG2, DoryMessages, ExtendedProverState, ExtendedVerifierState, G1Projective,
    ProverSetup, VMVProverState, VMVVerifierState, VerifierSetup,
};
use crate::base::{if_rayon, proof::Transcript};
use alloc::vec::Vec;
use ark_ec::VariableBaseMSM;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// This is the prover side of the Eval-VMV-RE algorithm in section 5 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Note: there are several typos in the paper here.
/// * The paper uses C' and T_vec_prime interchangeably. They are the same thing.
/// * The paper uses s1 = L and s2 = R as the arguments to Dory-Innerproduct. This is backwards.
///     We should have E_1 = s2 * v1 and E_2 = s1 * v2, which is the case if we use s1 = R and s2 = L.
///
/// Note: the paper has the prover send E_2 to the verifier. We opt to simply have the verifier compute E_2 from y, which is known.
#[tracing::instrument(level = "debug", skip_all)]
pub fn eval_vmv_re_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: VMVProverState,
    setup: &ProverSetup,
) -> ExtendedProverState {
    let C = pairings::pairing(
        G1Projective::msm_unchecked(&state.T_vec_prime, &state.v_vec),
        setup.Gamma_2_fin,
    );
    let D_2 = pairings::pairing(
        G1Projective::msm_unchecked(setup.Gamma_1[state.nu], &state.v_vec),
        setup.Gamma_2_fin,
    );
    let E_1 = G1Projective::msm_unchecked(&state.T_vec_prime, &state.L_vec);
    messages.prover_send_GT_message(transcript, C);
    messages.prover_send_GT_message(transcript, D_2);
    messages.prover_send_G1_message(transcript, E_1);
    let Gamma_2_fin = setup.Gamma_2_fin;
    let v2 = if_rayon!(state.v_vec.par_iter(), state.v_vec.iter())
        .map(|c| (Gamma_2_fin * c).into())
        .collect::<Vec<_>>();
    ExtendedProverState::from_vmv_prover_state(state, v2)
}

/// This is the verifier side of the Eval-VMV-RE algorithm in section 5 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Note: there are several typos in the paper here.
/// * The paper uses `C'` and `T_vec_prime` interchangeably. They are the same thing.
/// * The paper uses `s1 = L` and `s2 = R` as the arguments to Dory-Innerproduct. This is backwards.
///     We should have `E_1 = s2 * v1` and `E_2 = s1 * v2`, which is the case if we use `s1 = R` and `s2 = L`.
///
/// Note: the paper has the prover send `E_2` to the verifier. We opt to simply have the verifier compute `E_2` from y, which is known.
pub fn eval_vmv_re_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: VMVVerifierState,
    setup: &VerifierSetup,
) -> Option<ExtendedVerifierState> {
    if messages.GT_messages.len() < 2 || messages.G1_messages.is_empty() {
        return None;
    }
    let C = messages.prover_recieve_GT_message(transcript).into();
    let D_2 = messages.prover_recieve_GT_message(transcript).into();
    let E_1 = messages.prover_recieve_G1_message(transcript).into();
    let D_1 = state.T;
    let E_2 = DeferredG2::from(setup.Gamma_2_fin) * state.y;
    let s1_tensor = state.r_tensor;
    let s2_tensor = state.l_tensor;
    let nu = state.nu;
    Some(ExtendedVerifierState::new_tensor(
        E_1, E_2, s1_tensor, s2_tensor, C, D_1, D_2, nu,
    ))
}
