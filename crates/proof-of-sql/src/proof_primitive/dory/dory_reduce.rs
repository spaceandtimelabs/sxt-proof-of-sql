use super::{
    dory_reduce_helper::*, DoryMessages, ProverSetup, ProverState, VerifierSetup, VerifierState,
};
use crate::base::proof::Transcript;

/// This is the prover side of the Dory-Reduce algorithm in section 3.2 of <https://eprint.iacr.org/2020/1274.pdf>.
#[cfg(test)]
pub fn dory_reduce_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: &mut ProverState,
    setup: &ProverSetup,
) {
    assert!(state.nu > 0);
    let half_n = 1usize << (state.nu - 1);
    let (D_1L, D_1R, D_2L, D_2R) = dory_reduce_prove_compute_Ds(state, setup, half_n);
    messages.prover_send_GT_message(transcript, D_1L);
    messages.prover_send_GT_message(transcript, D_1R);
    messages.prover_send_GT_message(transcript, D_2L);
    messages.prover_send_GT_message(transcript, D_2R);
    let betas = messages.verifier_F_message(transcript);
    dory_reduce_prove_mutate_v_vecs(state, setup, betas);
    let (C_plus, C_minus) = dory_reduce_prove_compute_Cs(state, half_n);
    messages.prover_send_GT_message(transcript, C_plus);
    messages.prover_send_GT_message(transcript, C_minus);
    let alphas = messages.verifier_F_message(transcript);
    dory_reduce_prove_fold_v_vecs(state, alphas, half_n);
    state.nu -= 1;
}

/// This is the verifier side of the Dory-Reduce algorithm in section 3.2 of <https://eprint.iacr.org/2020/1274.pdf>.
#[cfg(test)]
pub fn dory_reduce_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: &mut VerifierState,
    setup: &VerifierSetup,
) -> bool {
    assert!(state.nu > 0);
    if messages.GT_messages.len() < 6 {
        return false;
    }
    let D_1L = messages.prover_recieve_GT_message(transcript);
    let D_1R = messages.prover_recieve_GT_message(transcript);
    let D_2L = messages.prover_recieve_GT_message(transcript);
    let D_2R = messages.prover_recieve_GT_message(transcript);
    let betas = messages.verifier_F_message(transcript);
    let C_plus = messages.prover_recieve_GT_message(transcript);
    let C_minus = messages.prover_recieve_GT_message(transcript);
    let alphas = messages.verifier_F_message(transcript);
    dory_reduce_verify_update_C(state, setup, (C_plus, C_minus), alphas, betas);
    dory_reduce_verify_update_Ds(state, setup, (D_1L, D_1R, D_2L, D_2R), alphas, betas);
    state.nu -= 1;
    true
}
