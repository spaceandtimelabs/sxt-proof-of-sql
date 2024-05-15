use super::{
    DeferredG2, DoryMessages, ExtendedProverState, ExtendedVerifierState, ProverSetup,
    VMVProverState, VMVVerifierState, VerifierSetup, G1,
};
use ark_ec::{pairing::Pairing, ScalarMul, VariableBaseMSM};
use merlin::Transcript;

/// This is the prover side of the Eval-VMV-RE algorithm in section 5 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Note: there are several typos in the paper here.
/// * The paper uses C' and T_vec_prime interchangeably. They are the same thing.
/// * The paper uses s1 = L and s2 = R as the arguments to Dory-Innerproduct. This is backwards.
///     We should have E_1 = s2 * v1 and E_2 = s1 * v2, which is the case if we use s1 = R and s2 = L.
///
/// Note: the paper has the prover send E_2 to the verifier. We opt to simply have the verifier compute E_2 from y, which is known.
pub fn eval_vmv_re_prove(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: VMVProverState,
    setup: &ProverSetup,
) -> ExtendedProverState {
    let C = Pairing::pairing(
        G1::msm_unchecked(
            &ScalarMul::batch_convert_to_mul_base(&state.T_vec_prime),
            &state.v_vec,
        ),
        setup.Gamma_2_fin,
    );
    let D_2 = Pairing::pairing(
        G1::msm_unchecked(
            &ScalarMul::batch_convert_to_mul_base(setup.Gamma_1[state.nu]),
            &state.v_vec,
        ),
        setup.Gamma_2_fin,
    );
    let E_1 = G1::msm_unchecked(
        &ScalarMul::batch_convert_to_mul_base(&state.T_vec_prime),
        &state.L_vec,
    );
    messages.prover_send_GT_message(transcript, C);
    messages.prover_send_GT_message(transcript, D_2);
    messages.prover_send_G1_message(transcript, E_1);
    let s1 = state.R_vec;
    let s2 = state.L_vec;
    let v1 = state.T_vec_prime;
    let v2 = Vec::from_iter(state.v_vec.iter().map(|c| setup.Gamma_2_fin * c));
    let nu = state.nu;
    ExtendedProverState::new(s1, s2, v1, v2, nu)
}

/// This is the verifier side of the Eval-VMV-RE algorithm in section 5 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Note: there are several typos in the paper here.
/// * The paper uses C' and T_vec_prime interchangeably. They are the same thing.
/// * The paper uses s1 = L and s2 = R as the arguments to Dory-Innerproduct. This is backwards.
///     We should have E_1 = s2 * v1 and E_2 = s1 * v2, which is the case if we use s1 = R and s2 = L.
///
/// Note: the paper has the prover send E_2 to the verifier. We opt to simply have the verifier compute E_2 from y, which is known.
pub fn eval_vmv_re_verify(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    state: VMVVerifierState,
    setup: &VerifierSetup,
) -> Option<ExtendedVerifierState> {
    if messages.GT_messages.len() < 2 || messages.G1_messages.is_empty() {
        return None;
    }
    let C = messages.prover_recieve_GT_message(transcript).into();
    let D_2 = messages.prover_recieve_GT_message(transcript).into();
    let E_1 = messages.prover_recieve_G1_message(transcript).into();
    let D_1 = state.T.into();
    let E_2 = DeferredG2::from(setup.Gamma_2_fin) * state.y;
    let s1 = state.R_vec;
    let s2 = state.L_vec;
    let nu = state.nu;
    Some(ExtendedVerifierState::new(
        E_1, E_2, s1, s2, C, D_1, D_2, nu,
    ))
}
