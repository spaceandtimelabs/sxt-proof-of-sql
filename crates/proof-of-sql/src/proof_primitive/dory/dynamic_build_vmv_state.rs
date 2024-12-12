use super::{
    dynamic_dory_helper::compute_dynamic_v_vec, DeferredGT, DoryScalar, G1Affine, VMVProverState,
    VMVVerifierState, F,
};
use crate::proof_primitive::dynamic_matrix_utils::standard_basis_helper::compute_dynamic_vecs;
use alloc::vec::Vec;

/// Builds a [`VMVProverState`] from the given parameters.
pub(super) fn build_dynamic_vmv_prover_state(
    a: &[F],
    b_point: &[F],
    T_vec_prime: Vec<G1Affine>,
    nu: usize,
) -> VMVProverState {
    let (lo_vec, hi_vec) =
        compute_dynamic_vecs(bytemuck::TransparentWrapper::wrap_slice(b_point) as &[DoryScalar]);
    let (lo_vec, hi_vec) = (
        bytemuck::TransparentWrapper::peel_slice(&lo_vec) as &[F],
        bytemuck::TransparentWrapper::peel_slice(&hi_vec) as &[F],
    );
    let v_vec = compute_dynamic_v_vec(a, hi_vec, nu);
    VMVProverState {
        v_vec,
        T_vec_prime,
        L_vec: hi_vec.to_vec(),
        R_vec: lo_vec.to_vec(),
        #[cfg(test)]
        l_tensor: Vec::with_capacity(0),
        #[cfg(test)]
        r_tensor: b_point.to_vec(),
        nu,
    }
}

/// Builds a [`VMVVerifierState`] from the given parameters.
pub(super) fn build_dynamic_vmv_verifier_state(
    y: F,
    b_point: &[F],
    T: DeferredGT,
    nu: usize,
) -> VMVVerifierState {
    VMVVerifierState {
        y,
        T,
        l_tensor: Vec::with_capacity(0),
        r_tensor: b_point.to_vec(),
        nu,
    }
}
