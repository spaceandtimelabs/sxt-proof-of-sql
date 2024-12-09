use super::{
    pairings::{multi_pairing_2, multi_pairing_4},
    DeferredGT, ProverSetup, ProverState, VerifierSetup, VerifierState, F, GT,
};
use crate::{base::if_rayon, utils::log};
#[cfg(feature = "rayon")]
use rayon::{
    iter::IndexedParallelIterator,
    prelude::{IntoParallelRefMutIterator, ParallelIterator},
};

/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * D_1L = <v_1L, Gamma_2'>
/// * D_1R = <v_1R, Gamma_2'>
/// * D_2L = <Gamma_1', v_2L>
/// * D_2R = <Gamma_1', v_2R>
///
/// Returns (D_1L, D_1R, D_2L, D_2R).
#[tracing::instrument(level = "debug", skip_all)]
pub fn dory_reduce_prove_compute_Ds(
    state: &ProverState,
    setup: &ProverSetup,
    half_n: usize,
) -> (GT, GT, GT, GT) {
    log::log_memory_usage("Start");

    let (v_1L, v_1R) = state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.v2.split_at(half_n);
    let (D_1L, D_1R, D_2L, D_2R) = multi_pairing_4(
        (v_1L, setup.Gamma_2[state.nu - 1]),
        (v_1R, setup.Gamma_2[state.nu - 1]),
        (setup.Gamma_1[state.nu - 1], v_2L),
        (setup.Gamma_1[state.nu - 1], v_2R),
    );

    log::log_memory_usage("End");

    (D_1L, D_1R, D_2L, D_2R)
}
/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Mutates v_1 and v_2.
/// * v_1 <- v_1 + beta * Gamma_1
/// * v_2 <- v_2 + beta_inv * Gamma_2
#[tracing::instrument(level = "debug", skip_all)]
pub fn dory_reduce_prove_mutate_v_vecs(
    state: &mut ProverState,
    setup: &ProverSetup,
    (beta, beta_inv): (F, F),
) {
    log::log_memory_usage("Start");

    if_rayon!(state.v1.par_iter_mut(), state.v1.iter_mut())
        .zip(setup.Gamma_1[state.nu])
        .for_each(|(v, &g)| *v = (*v + g * beta).into());
    if_rayon!(state.v2.par_iter_mut(), state.v2.iter_mut())
        .zip(setup.Gamma_2[state.nu])
        .for_each(|(v, &g)| *v = (*v + g * beta_inv).into());

    log::log_memory_usage("End");
}
/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Computes
/// * C_plus = <v_1L, v_2R>
/// * C_minus = <v_1R, v_2L>
#[tracing::instrument(level = "debug", skip_all)]
pub fn dory_reduce_prove_compute_Cs(state: &ProverState, half_n: usize) -> (GT, GT) {
    log::log_memory_usage("Start");

    let (v_1L, v_1R) = state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.v2.split_at(half_n);
    let (C_plus, C_minus) = multi_pairing_2((v_1L, v_2R), (v_1R, v_2L));

    log::log_memory_usage("End");

    (C_plus, C_minus)
}

/// From the Dory-Reduce algorithm in section 3.2 of https://eprint.iacr.org/2020/1274.pdf.
///
/// Folds v_1 and v_2.
/// * v_1' <- alpha * v_1L + v_1R
/// * v_2' <- alpha_inv * v_2L + v_2R
#[tracing::instrument(level = "debug", skip_all)]
pub fn dory_reduce_prove_fold_v_vecs(
    state: &mut ProverState,
    (alpha, alpha_inv): (F, F),
    half_n: usize,
) {
    log::log_memory_usage("Start");

    let (v_1L, v_1R) = state.v1.split_at_mut(half_n);
    let (v_2L, v_2R) = state.v2.split_at_mut(half_n);
    if_rayon!(v_1L.par_iter_mut(), v_1L.iter_mut())
        .zip(v_1R)
        .for_each(|(v_L, v_R)| *v_L = (*v_L * alpha + v_R).into());
    if_rayon!(v_2L.par_iter_mut(), v_2L.iter_mut())
        .zip(v_2R)
        .for_each(|(v_L, v_R)| *v_L = (*v_L * alpha_inv + v_R).into());
    state.v1.truncate(half_n);
    state.v2.truncate(half_n);

    log::log_memory_usage("End");
}
/// From the Dory-Reduce algorithm in section 3.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Updates C
/// * `C' <- C + chi + beta * D_2 + beta_inv * D_1 + alpha * C_plus + alpha_inv * C_minus`
///
/// Note: this should not be used after `dory_reduce_verify_update_Ds` because that function mutates the Ds.
pub fn dory_reduce_verify_update_C(
    state: &mut VerifierState,
    setup: &VerifierSetup,
    (C_plus, C_minus): (GT, GT),
    (alpha, alpha_inv): (F, F),
    (beta, beta_inv): (F, F),
) {
    state.C += state.D_2.clone() * beta
        + state.D_1.clone() * beta_inv
        + DeferredGT::from(C_plus) * alpha
        + DeferredGT::from(C_minus) * alpha_inv
        + setup.chi[state.nu];
}
/// From the Dory-Reduce algorithm in section 3.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Updates `D_1` and `D_2`
/// * `D_1' <- alpha * D_1 + D_1R + alpha * beta * Delta_1L + beta * Delta_1R`
/// * `D_2' <- alpha_inv * D_2 + D_2R + alpha_inv * beta_inv * Delta_2L + beta_inv * Delta_2R`
pub fn dory_reduce_verify_update_Ds(
    state: &mut VerifierState,
    setup: &VerifierSetup,
    (D_1L, D_1R, D_2L, D_2R): (GT, GT, GT, GT),
    (alpha, alpha_inv): (F, F),
    (beta, beta_inv): (F, F),
) {
    state.D_1 = DeferredGT::from(D_1L) * alpha
        + D_1R
        + DeferredGT::from(setup.Delta_1L[state.nu]) * beta * alpha
        + DeferredGT::from(setup.Delta_1R[state.nu]) * beta;
    state.D_2 = DeferredGT::from(D_2L) * alpha_inv
        + D_2R
        + DeferredGT::from(setup.Delta_2L[state.nu]) * beta_inv * alpha_inv
        + DeferredGT::from(setup.Delta_2R[state.nu]) * beta_inv;
}
