use super::{ProverSetup, ProverState, VerifierSetup, VerifierState, F, GT};
use ark_ec::pairing::Pairing;

/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * D_1L = <v_1L, Gamma_2'>
/// * D_1R = <v_1R, Gamma_2'>
/// * D_2L = <Gamma_1', v_2L>
/// * D_2R = <Gamma_1', v_2R>
///
/// Returns (D_1L, D_1R, D_2L, D_2R).
pub fn dory_reduce_prove_compute_Ds(
    state: &ProverState,
    setup: &ProverSetup,
    half_n: usize,
) -> (GT, GT, GT, GT) {
    let (v_1L, v_1R) = state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.v2.split_at(half_n);
    let D_1L: GT = Pairing::multi_pairing(v_1L, setup.Gamma_2[state.nu - 1]);
    let D_1R: GT = Pairing::multi_pairing(v_1R, setup.Gamma_2[state.nu - 1]);
    let D_2L: GT = Pairing::multi_pairing(setup.Gamma_1[state.nu - 1], v_2L);
    let D_2R: GT = Pairing::multi_pairing(setup.Gamma_1[state.nu - 1], v_2R);
    (D_1L, D_1R, D_2L, D_2R)
}
/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Mutates v_1 and v_2.
/// * v_1 <- v_1 + beta * Gamma_1
/// * v_2 <- v_2 + beta_inv * Gamma_2
pub fn dory_reduce_prove_mutate_v_vecs(
    state: &mut ProverState,
    setup: &ProverSetup,
    (beta, beta_inv): (F, F),
) {
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
}
/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * C_plus = <v_1L, v_2R>
/// * C_minus = <v_1R, v_2L>
pub fn dory_reduce_prove_compute_Cs(state: &ProverState, half_n: usize) -> (GT, GT) {
    let (v_1L, v_1R) = state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.v2.split_at(half_n);
    let C_plus = Pairing::multi_pairing(v_1L, v_2R);
    let C_minus = Pairing::multi_pairing(v_1R, v_2L);
    (C_plus, C_minus)
}

/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Folds v_1 and v_2.
/// * v_1' <- alpha * v_1L + v_1R
/// * v_2' <- alpha_inv * v_2L + v_2R
pub fn dory_reduce_prove_fold_v_vecs(
    state: &mut ProverState,
    (alpha, alpha_inv): (F, F),
    half_n: usize,
) {
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
}
/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Updates C
/// * C' <- C + chi + beta * D_2 + beta_inv * D_1 + alpha * C_plus + alpha_inv * C_minus
///
/// Note: this should not be used after `dory_reduce_verify_update_Ds` because that function mutates the Ds.
pub fn dory_reduce_verify_update_C(
    state: &mut VerifierState,
    setup: &VerifierSetup,
    (C_plus, C_minus): (GT, GT),
    (alpha, alpha_inv): (F, F),
    (beta, beta_inv): (F, F),
) {
    state.C += setup.chi[state.nu]
        + state.D_2 * beta
        + state.D_1 * beta_inv
        + C_plus * alpha
        + C_minus * alpha_inv;
}
/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Updates D_1 and D_2
/// * D_1' <- alpha * D_1 + D_1R + alpha * beta * Delta_1L + beta * Delta_1R
/// * D_2' <- alpha_inv * D_2 + D_2R + alpha_inv * beta_inv * Delta_2L + beta_inv * Delta_2R
pub fn dory_reduce_verify_update_Ds(
    state: &mut VerifierState,
    setup: &VerifierSetup,
    (D_1L, D_1R, D_2L, D_2R): (GT, GT, GT, GT),
    (alpha, alpha_inv): (F, F),
    (beta, beta_inv): (F, F),
) {
    state.D_1 = D_1L * alpha
        + D_1R
        + setup.Delta_1L[state.nu] * beta * alpha
        + setup.Delta_1R[state.nu] * beta;
    state.D_2 = D_2L * alpha_inv
        + D_2R
        + setup.Delta_2L[state.nu] * beta_inv * alpha_inv
        + setup.Delta_2R[state.nu] * beta_inv;
}
