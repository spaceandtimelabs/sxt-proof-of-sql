use super::{
    dory_reduce_helper::*,
    extended_dory_reduce_helper::*,
    extended_state::{ExtendedProverState, ExtendedVerifierState},
    DoryMessages, ProverSetup, VerifierSetup,
};
use merlin::Transcript;

/// This is the prover side of the extended Dory-Reduce algorithm in section 3.2 & 4.2 of https://eprint.iacr.org/2020/1274.pdf.
pub fn extended_dory_reduce_prove(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: &mut ExtendedProverState,
    setup: &ProverSetup,
) {
    assert!(state.base_state.nu > 0);
    let half_n = 1usize << (state.base_state.nu - 1);
    let (D_1L, D_1R, D_2L, D_2R) = dory_reduce_prove_compute_Ds(&state.base_state, setup, half_n);
    let (E_1beta, E_2beta) = extended_dory_reduce_prove_compute_E_betas(state, setup);
    messages.send_prover_GT_message(transcript, D_1L);
    messages.send_prover_GT_message(transcript, D_1R);
    messages.send_prover_GT_message(transcript, D_2L);
    messages.send_prover_GT_message(transcript, D_2R);
    messages.send_prover_G1_message(transcript, E_1beta);
    messages.send_prover_G2_message(transcript, E_2beta);
    let betas = messages.verifier_F_message(transcript);
    dory_reduce_prove_mutate_v_vecs(&mut state.base_state, setup, betas);
    let (C_plus, C_minus) = dory_reduce_prove_compute_Cs(&state.base_state, half_n);
    let (E_1plus, E_1minus, E_2plus, E_2minus) =
        extended_dory_reduce_prove_compute_signed_Es(state, half_n);
    messages.send_prover_GT_message(transcript, C_plus);
    messages.send_prover_GT_message(transcript, C_minus);
    messages.send_prover_G1_message(transcript, E_1plus);
    messages.send_prover_G1_message(transcript, E_1minus);
    messages.send_prover_G2_message(transcript, E_2plus);
    messages.send_prover_G2_message(transcript, E_2minus);
    let alphas = messages.verifier_F_message(transcript);
    dory_reduce_prove_fold_v_vecs(&mut state.base_state, alphas, half_n);
    extended_dory_reduce_prove_fold_s_vecs(state, alphas, half_n);
    state.base_state.nu -= 1;
}

/// This is the verifier side of the extended Dory-Reduce algorithm in section 3.2 & 4.2 of https://eprint.iacr.org/2020/1274.pdf.
pub fn extended_dory_reduce_verify(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: &mut ExtendedVerifierState,
    setup: &VerifierSetup,
) -> bool {
    assert!(state.base_state.nu > 0);
    if messages.GT_messages.len() < 6
        || messages.G1_messages.len() < 3
        || messages.G2_messages.len() < 3
    {
        return false;
    }
    let D_1L = messages.recieve_prover_GT_message(transcript);
    let D_1R = messages.recieve_prover_GT_message(transcript);
    let D_2L = messages.recieve_prover_GT_message(transcript);
    let D_2R = messages.recieve_prover_GT_message(transcript);
    let E_1beta = messages.recieve_prover_G1_message(transcript);
    let E_2beta = messages.recieve_prover_G2_message(transcript);
    let betas = messages.verifier_F_message(transcript);
    let C_plus = messages.recieve_prover_GT_message(transcript);
    let C_minus = messages.recieve_prover_GT_message(transcript);
    let E_1plus = messages.recieve_prover_G1_message(transcript);
    let E_1minus = messages.recieve_prover_G1_message(transcript);
    let E_2plus = messages.recieve_prover_G2_message(transcript);
    let E_2minus = messages.recieve_prover_G2_message(transcript);
    let alphas = messages.verifier_F_message(transcript);
    dory_reduce_verify_update_C(
        &mut state.base_state,
        setup,
        (C_plus, C_minus),
        alphas,
        betas,
    );
    dory_reduce_verify_update_Ds(
        &mut state.base_state,
        setup,
        (D_1L, D_1R, D_2L, D_2R),
        alphas,
        betas,
    );
    extended_dory_reduce_verify_update_Es(
        state,
        (E_1beta, E_2beta),
        (E_1plus, E_1minus, E_2plus, E_2minus),
        alphas,
        betas,
    );
    extended_dory_reduce_verify_fold_s_vecs(state, alphas);
    state.base_state.nu -= 1;
    true
}
