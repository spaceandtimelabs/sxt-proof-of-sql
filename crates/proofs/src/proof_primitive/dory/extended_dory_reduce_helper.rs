use super::{
    extended_state::{ExtendedProverState, ExtendedVerifierState},
    ProverSetup, F, G1, G2,
};
use ark_ec::{ScalarMul, VariableBaseMSM};

/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * E_1beta = <Gamma_1, s_2>
/// * E_2beta = <s_1, Gamma_2>
pub fn extended_dory_reduce_prove_compute_E_betas(
    state: &ExtendedProverState,
    setup: &ProverSetup,
) -> (G1, G2) {
    let E_1beta: G1 = G1::msm(
        &ScalarMul::batch_convert_to_mul_base(setup.Gamma_1[state.base_state.nu]),
        &state.s2,
    )
    .unwrap();
    let E_2beta: G2 = G2::msm(
        &ScalarMul::batch_convert_to_mul_base(setup.Gamma_2[state.base_state.nu]),
        &state.s1,
    )
    .unwrap();
    (E_1beta, E_2beta)
}
/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * E_1plus = <v_1L, s_2R>
/// * E_1minus = <v_1R, s_2L>
/// * E_2plus = <s_1L, v_2R>
/// * E_2minus = <s_1R, v_2L>
pub fn extended_dory_reduce_prove_compute_signed_Es(
    state: &ExtendedProverState,
    half_n: usize,
) -> (G1, G1, G2, G2) {
    let (v_1L, v_1R) = state.base_state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.base_state.v2.split_at(half_n);
    let (s_1L, s_1R) = state.s1.split_at(half_n);
    let (s_2L, s_2R) = state.s2.split_at(half_n);
    let E_1plus = G1::msm(&ScalarMul::batch_convert_to_mul_base(v_1L), s_2R).unwrap();
    let E_1minus = G1::msm(&ScalarMul::batch_convert_to_mul_base(v_1R), s_2L).unwrap();
    let E_2plus = G2::msm(&ScalarMul::batch_convert_to_mul_base(v_2R), s_1L).unwrap();
    let E_2minus = G2::msm(&ScalarMul::batch_convert_to_mul_base(v_2L), s_1R).unwrap();
    (E_1plus, E_1minus, E_2plus, E_2minus)
}
/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Folds s1 and s2.
/// * s_1' <- alpha * s_1L + s_1R
/// * s_2' <- alpha_inv * s_2L + s_2R
pub fn extended_dory_reduce_prove_fold_s_vecs(
    state: &mut ExtendedProverState,
    (alpha, alpha_inv): (F, F),
    half_n: usize,
) {
    let (s_1L, s_1R) = state.s1.split_at_mut(half_n);
    let (s_2L, s_2R) = state.s2.split_at_mut(half_n);
    s_1L.iter_mut()
        .zip(s_1R)
        .for_each(|(s_L, s_R)| *s_L = *s_L * alpha + s_R);
    s_2L.iter_mut()
        .zip(s_2R)
        .for_each(|(s_L, s_R)| *s_L = *s_L * alpha_inv + s_R);
    state.s1.truncate(half_n);
    state.s2.truncate(half_n);
}
/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Updates E_1 and E_2
/// * E_1' <- E_1 + beta * E_1beta + alpha * E_1plus + alpha_inv * E_1minus
/// * E_2' <- E_2 + beta_inv * E_2beta + alpha * E_2plus + alpha_inv * E_2minus
pub fn extended_dory_reduce_verify_update_Es(
    state: &mut ExtendedVerifierState,
    (E_1beta, E_2beta): (G1, G2),
    (E_1plus, E_1minus, E_2plus, E_2minus): (G1, G1, G2, G2),
    (alpha, alpha_inv): (F, F),
    (beta, beta_inv): (F, F),
) {
    state.E_1 += E_1beta * beta + E_1plus * alpha + E_1minus * alpha_inv;
    state.E_2 += E_2beta * beta_inv + E_2plus * alpha + E_2minus * alpha_inv;
}

/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Folds s1 and s2.
/// * s_1' <- alpha * s_1L + s_1R
/// * s_2' <- alpha_inv * s_2L + s_2R
pub fn extended_dory_reduce_verify_fold_s_vecs(
    state: &mut ExtendedVerifierState,
    (alpha, alpha_inv): (F, F),
) {
    let half_n = 1usize << (state.base_state.nu - 1);
    let (s_1L, s_1R) = state.s1.split_at_mut(half_n);
    let (s_2L, s_2R) = state.s2.split_at_mut(half_n);
    s_1L.iter_mut()
        .zip(s_1R)
        .for_each(|(s_L, s_R)| *s_L = *s_L * alpha + s_R);
    s_2L.iter_mut()
        .zip(s_2R)
        .for_each(|(s_L, s_R)| *s_L = *s_L * alpha_inv + s_R);
    state.s1.truncate(half_n);
    state.s2.truncate(half_n);
}
