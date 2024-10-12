use super::{
    extended_state::{ExtendedProverState, ExtendedVerifierState},
    DeferredG1, DeferredG2, G1Affine, G1Projective, G2Affine, G2Projective, ProverSetup, F,
};
use ark_ec::VariableBaseMSM;
use ark_ff::Field;

/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * E_1beta = <Gamma_1, s_2>
/// * E_2beta = <s_1, Gamma_2>
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_reduce_prove_compute_E_betas(
    state: &ExtendedProverState,
    setup: &ProverSetup,
) -> (G1Affine, G2Affine) {
    let E_1beta: G1Affine =
        G1Projective::msm_unchecked(setup.Gamma_1[state.base_state.nu], &state.s2).into();
    let E_2beta: G2Affine =
        G2Projective::msm_unchecked(setup.Gamma_2[state.base_state.nu], &state.s1).into();
    (E_1beta, E_2beta)
}
/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * E_1plus = <v_1L, s_2R>
/// * E_1minus = <v_1R, s_2L>
/// * E_2plus = <s_1L, v_2R>
/// * E_2minus = <s_1R, v_2L>
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_reduce_prove_compute_signed_Es(
    state: &ExtendedProverState,
    half_n: usize,
) -> (G1Affine, G1Affine, G2Affine, G2Affine) {
    let (v_1L, v_1R) = state.base_state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.base_state.v2.split_at(half_n);
    let (s_1L, s_1R) = state.s1.split_at(half_n);
    let (s_2L, s_2R) = state.s2.split_at(half_n);
    let E_1plus = G1Projective::msm_unchecked(v_1L, s_2R).into();
    let E_1minus = G1Projective::msm_unchecked(v_1R, s_2L).into();
    let E_2plus = G2Projective::msm_unchecked(v_2R, s_1L).into();
    let E_2minus = G2Projective::msm_unchecked(v_2L, s_1R).into();
    (E_1plus, E_1minus, E_2plus, E_2minus)
}
/// From the extended Dory-Reduce algorithm in section 4.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Folds s1 and s2.
/// * s_1' <- alpha * s_1L + s_1R
/// * s_2' <- alpha_inv * s_2L + s_2R
#[tracing::instrument(level = "debug", skip_all)]
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
/// From the extended Dory-Reduce algorithm in section 4.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Updates `E_1` and `E_2`
/// * `E_1' <- E_1 + beta * E_1beta + alpha * E_1plus + alpha_inv * E_1minus`
/// * `E_2' <- E_2 + beta_inv * E_2beta + alpha * E_2plus + alpha_inv * E_2minus`
pub fn extended_dory_reduce_verify_update_Es(
    state: &mut ExtendedVerifierState,
    (E_1beta, E_2beta): (G1Affine, G2Affine),
    (E_1plus, E_1minus, E_2plus, E_2minus): (G1Affine, G1Affine, G2Affine, G2Affine),
    (alpha, alpha_inv): (F, F),
    (beta, beta_inv): (F, F),
) {
    state.E_1 += DeferredG1::from(E_1beta) * beta
        + DeferredG1::from(E_1plus) * alpha
        + DeferredG1::from(E_1minus) * alpha_inv;
    state.E_2 += DeferredG2::from(E_2beta) * beta_inv
        + DeferredG2::from(E_2plus) * alpha
        + DeferredG2::from(E_2minus) * alpha_inv;
}

/// From the extended Dory-Reduce algorithm in section 4.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Folds s1 and s2.
/// * `s_1' <- alpha * s_1L + s_1R`
/// * `s_2' <- alpha_inv * s_2L + s_2R`
///
/// NOTE: this logically is identical to `extended_dory_reduce_prove_fold_s_vecs`. However, the actual values
/// of the s vectors not needed.
///
/// Instead, only the final, completely folded value is used, in [`fold_scalars_0_verify`](super::fold_scalars_0_verify).
/// This implementation works because the final value of the s vectors is:
///
/// `product (1-s1_tensor[i]) * alpha[i] + s1_tensor[i] over all i`
///
/// So, instead of folding the s vectors, we can directly compute the final value by mutating
///
/// `s1_tensor[nu-1] <- s1_tensor[nu-1] * (1- alpha) + alpha`
///
/// and taking the product in [`fold_scalars_0_verify`](super::fold_scalars_0_verify).
pub fn extended_dory_reduce_verify_fold_s_vecs(state: &ExtendedVerifierState) -> (F, F) {
    (
        state
            .s1_tensor
            .iter()
            .zip(state.alphas.iter())
            .map(|(s, a)| (F::ONE - s) * a + s)
            .product(),
        state
            .s2_tensor
            .iter()
            .zip(state.alpha_invs.iter())
            .map(|(s, a)| (F::ONE - s) * a + s)
            .product(),
    )
}
