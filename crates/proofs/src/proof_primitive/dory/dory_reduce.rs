use super::{DoryMessages, ProverSetup, ProverState, VerifierSetup, VerifierState, GT};
use ark_ec::pairing::Pairing;
use merlin::Transcript;

/// This is the prover side of the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
pub fn dory_reduce_prove(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: &mut ProverState,
    setup: &ProverSetup,
) {
    // See section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
    //
    // Note:
    // We use nu = m and k = m-i or m-j.
    // This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
    //
    // So,
    // * `Gamma_1[k]` = Γ_1,(m-k) in the paper.
    // * `Gamma_2[k]` = Γ_2,(m-k) in the paper.
    // * `Delta_1L[k]` = Δ_1L,(m-k) in the paper, so `Delta_1L[0]` is unused.
    // * `Delta_1R[k]` = Δ_1R,(m-k) in the paper, so `Delta_1R[0]` is unused.
    // * `Delta_2L[k]` = Δ_2L,(m-k) in the paper, so `Delta_2L[0]` is unused.
    // * `Delta_2R[k]` = Δ_2R,(m-k) in the paper, so `Delta_2R[0]` is unused.
    // * `chi[k]` = χ,(m-k) in the paper.
    // * `Gamma_1_0` is the Γ_1 used in Scalar-Product algorithm.
    // * `Gamma_2_0` is the Γ_2 used in Scalar-Product algorithm.

    assert!(state.nu > 0);
    let half_n = 1usize << (state.nu - 1);
    let (v_1L, v_1R) = state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.v2.split_at(half_n);
    let D_1L: GT = Pairing::multi_pairing(v_1L, setup.Gamma_2[state.nu - 1]);
    let D_1R: GT = Pairing::multi_pairing(v_1R, setup.Gamma_2[state.nu - 1]);
    let D_2L: GT = Pairing::multi_pairing(setup.Gamma_1[state.nu - 1], v_2L);
    let D_2R: GT = Pairing::multi_pairing(setup.Gamma_1[state.nu - 1], v_2R);
    messages.send_prover_GT_message(transcript, D_1L);
    messages.send_prover_GT_message(transcript, D_1R);
    messages.send_prover_GT_message(transcript, D_2L);
    messages.send_prover_GT_message(transcript, D_2R);
    let (beta, beta_inv) = messages.verifier_F_message(transcript);
    state
        .v1
        .iter_mut()
        .zip(setup.Gamma_1[state.nu])
        .for_each(|(v, &g)| *v += g * beta);
    state
        .v2
        .iter_mut()
        .zip(setup.Gamma_2[state.nu])
        .for_each(|(v, &g)| *v += g * beta_inv);
    let (v_1L, v_1R) = state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.v2.split_at(half_n);
    let C_plus = Pairing::multi_pairing(v_1L, v_2R);
    let C_minus = Pairing::multi_pairing(v_1R, v_2L);
    messages.send_prover_GT_message(transcript, C_plus);
    messages.send_prover_GT_message(transcript, C_minus);
    let (alpha, alpha_inv) = messages.verifier_F_message(transcript);
    let (v_1L, v_1R) = state.v1.split_at_mut(half_n);
    let (v_2L, v_2R) = state.v2.split_at_mut(half_n);
    v_1L.iter_mut()
        .zip(v_1R)
        .for_each(|(v_L, v_R)| *v_L = *v_L * alpha + v_R);
    v_2L.iter_mut()
        .zip(v_2R)
        .for_each(|(v_L, v_R)| *v_L = *v_L * alpha_inv + v_R);
    state.v1.truncate(half_n);
    state.v2.truncate(half_n);
    state.nu -= 1;
}

/// This is the verifier side of the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
pub fn dory_reduce_verify(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: &mut VerifierState,
    setup: &VerifierSetup,
) -> bool {
    // See section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
    //
    // Note:
    // We use nu = m and k = m-i or m-j.
    // This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
    //
    // So,
    // * `Gamma_1[k]` = Γ_1,(m-k) in the paper.
    // * `Gamma_2[k]` = Γ_2,(m-k) in the paper.
    // * `Delta_1L[k]` = Δ_1L,(m-k) in the paper, so `Delta_1L[0]` is unused.
    // * `Delta_1R[k]` = Δ_1R,(m-k) in the paper, so `Delta_1R[0]` is unused.
    // * `Delta_2L[k]` = Δ_2L,(m-k) in the paper, so `Delta_2L[0]` is unused.
    // * `Delta_2R[k]` = Δ_2R,(m-k) in the paper, so `Delta_2R[0]` is unused.
    // * `chi[k]` = χ,(m-k) in the paper.
    // * `Gamma_1_0` is the Γ_1 used in Scalar-Product algorithm.
    // * `Gamma_2_0` is the Γ_2 used in Scalar-Product algorithm.

    assert!(state.nu > 0);
    if messages.GT_messages.len() < 6 {
        return false;
    }
    let D_1L = messages.recieve_prover_GT_message(transcript);
    let D_1R = messages.recieve_prover_GT_message(transcript);
    let D_2L = messages.recieve_prover_GT_message(transcript);
    let D_2R = messages.recieve_prover_GT_message(transcript);
    let (beta, beta_inv) = messages.verifier_F_message(transcript);
    let C_plus = messages.recieve_prover_GT_message(transcript);
    let C_minus = messages.recieve_prover_GT_message(transcript);
    let (alpha, alpha_inv) = messages.verifier_F_message(transcript);
    state.C += setup.chi[state.nu]
        + state.D_2 * beta
        + state.D_1 * beta_inv
        + C_plus * alpha
        + C_minus * alpha_inv;
    state.D_1 = D_1L * alpha
        + D_1R
        + setup.Delta_1L[state.nu] * beta * alpha
        + setup.Delta_1R[state.nu] * beta;
    state.D_2 = D_2L * alpha_inv
        + D_2R
        + setup.Delta_2L[state.nu] * beta_inv * alpha_inv
        + setup.Delta_2R[state.nu] * beta_inv;
    state.nu -= 1;
    true
}
