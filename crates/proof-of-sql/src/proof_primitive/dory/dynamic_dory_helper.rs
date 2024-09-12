/// Compute the evaluations of the columns of the matrix M that is derived from `a`.
pub(super) fn compute_v_vec(a: &[F], L_vec: &[F], sigma: usize, nu: usize) -> Vec<F> {
    todo!()
}

/// Compute the commitments to the rows of the matrix M that is derived from `a`.
pub(super) fn compute_T_vec_prime(
    a: &[F],
    sigma: usize,
    nu: usize,
    prover_setup: &ProverSetup,
) -> Vec<G1Affine> {
    todo!()
}

/// Compute the size of the matrix M that is derived from `a`.
/// More specifically compute `nu`, where 2^nu is the side length the square matrix, M.
/// `num_vars` is the number of variables in the polynomial. In other words, it is the length of `b_points`, which is `ceil(log2(len(a)))`.
pub(super) fn compute_nu(num_vars: usize, sigma: usize) -> usize {
    todo!()
}

/// Compute the vectors L and R that are derived from `b_point`.
/// L and R are the vectors such that LMR is exactly the evaluation of `a` at the point `b_point`.
pub(super) fn compute_L_R_vec(b_point: &[F], sigma: usize, nu: usize) -> (Vec<F>, Vec<F>) {
    todo!()
}

/// Compute the l and r tensors that are derived from `b_point`.
/// These match with [compute_L_R_vec] but are in tensor form.
pub(super) fn compute_l_r_tensors(b_point: &[F], sigma: usize, nu: usize) -> (Vec<F>, Vec<F>) {
    todo!()
}

#[cfg(test)]
mod tests {}
