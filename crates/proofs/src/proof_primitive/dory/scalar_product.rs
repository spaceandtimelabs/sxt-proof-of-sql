#![allow(unused_variables)]
use super::{DoryMessages, ProverState, VerifierSetup, VerifierState};
use ark_ec::pairing::Pairing;
use merlin::Transcript;

/// This is the prover side of the Scalar-Product algorithm in section 3.1 of https://eprint.iacr.org/2020/1274.pdf.
pub fn scalar_product_prove(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: ProverState,
) {
    // See section 3.1 of https://eprint.iacr.org/2020/1274.pdf.
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

    assert_eq!(state.nu, 0);
    // v1 is a single element.
    let E_1 = state.v1[0];
    // v2 is a single element.
    let E_2 = state.v2[0];
    messages.send_prover_G1_message(transcript, E_1);
    messages.send_prover_G2_message(transcript, E_2);
    let (d, d_inv) = messages.verifier_F_message(transcript);
}

/// This is the verifier side of the Scalar-Product algorithm in section 3.1 of https://eprint.iacr.org/2020/1274.pdf.
pub fn scalar_product_verify(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: VerifierState,
    setup: &VerifierSetup,
) -> bool {
    // See section 3.1 of https://eprint.iacr.org/2020/1274.pdf.
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

    assert_eq!(state.nu, 0);
    if messages.G1_messages.len() != 1
        || messages.G2_messages.len() != 1
        || !messages.GT_messages.is_empty()
    {
        return false;
    }
    let E_1 = messages.recieve_prover_G1_message(transcript);
    let E_2 = messages.recieve_prover_G2_message(transcript);
    let (d, d_inv) = messages.verifier_F_message(transcript);
    Pairing::pairing(E_1 + setup.Gamma_1_0 * d, E_2 + setup.Gamma_2_0 * d_inv)
        == setup.chi[0] + state.C + state.D_2 * d + state.D_1 * d_inv
}
