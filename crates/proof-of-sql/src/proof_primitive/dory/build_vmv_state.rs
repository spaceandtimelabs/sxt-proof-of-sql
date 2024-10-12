use super::{
    compute_L_R_vec, compute_l_r_tensors, compute_v_vec, DeferredGT, G1Affine, VMVProverState,
    VMVVerifierState, F,
};
use alloc::vec::Vec;

/// Builds a [`VMVProverState`] from the given parameters.
pub(super) fn build_vmv_prover_state(
    a: &[F],
    b_point: &[F],
    T_vec_prime: Vec<G1Affine>,
    sigma: usize,
    nu: usize,
) -> VMVProverState {
    let (L_vec, R_vec) = compute_L_R_vec(b_point, sigma, nu);
    #[cfg(test)]
    let (l_tensor, r_tensor) = compute_l_r_tensors(b_point, sigma, nu);
    let v_vec = compute_v_vec(a, &L_vec, sigma, nu);
    VMVProverState {
        v_vec,
        T_vec_prime,
        #[cfg(test)]
        l_tensor,
        #[cfg(test)]
        r_tensor,
        L_vec,
        R_vec,
        nu,
    }
}

/// Builds a [`VMVVerifierState`] from the given parameters.
pub(super) fn build_vmv_verifier_state(
    y: F,
    b_point: &[F],
    T: DeferredGT,
    sigma: usize,
    nu: usize,
) -> VMVVerifierState {
    let (l_tensor, r_tensor) = compute_l_r_tensors(b_point, sigma, nu);
    VMVVerifierState {
        y,
        T,
        l_tensor,
        r_tensor,
        nu,
    }
}
