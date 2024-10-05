#[cfg(not(feature = "blitzar"))]
use super::G1Projective;
use super::{transpose, G1Affine, ProverSetup, F};
use crate::base::polynomial::compute_evaluation_vector;
#[cfg(feature = "blitzar")]
use crate::base::slice_ops::slice_cast;
use alloc::{vec, vec::Vec};
#[cfg(not(feature = "blitzar"))]
use ark_ec::{AffineRepr, VariableBaseMSM};
use ark_ff::{BigInt, MontBackend};
#[cfg(feature = "blitzar")]
use blitzar::compute::ElementP2;
#[cfg(feature = "blitzar")]
use core::mem;
use num_traits::{One, Zero};

/// Compute the evaluations of the columns of the matrix M that is derived from `a`.
pub(super) fn compute_v_vec(a: &[F], L_vec: &[F], sigma: usize, nu: usize) -> Vec<F> {
    a.chunks(1 << sigma)
        .zip(L_vec.iter())
        .fold(vec![F::zero(); 1 << nu], |mut v, (row, l)| {
            v.iter_mut().zip(row).for_each(|(v, a)| *v += l * a);
            v
        })
}

/// Converts a bls12-381 scalar to a u64 array.
#[cfg(feature = "blitzar")]
fn convert_scalar_to_array(
    scalars: &[ark_ff::Fp<MontBackend<ark_bls12_381::FrConfig, 4>, 4>],
) -> Vec<[u64; 4]> {
    scalars
        .iter()
        .map(|&element| BigInt::<4>::from(element).0)
        .collect()
}

/// Compute the commitments to the rows of the matrix M that is derived from `a`.
#[tracing::instrument(level = "debug", skip_all)]
#[cfg(feature = "blitzar")]
pub(super) fn compute_T_vec_prime(
    a: &[F],
    sigma: usize,
    nu: usize,
    prover_setup: &ProverSetup,
) -> Vec<G1Affine> {
    let num_columns = 1 << sigma;
    let num_outputs = 1 << nu;
    let data_size = mem::size_of::<F>();

    let a_array = convert_scalar_to_array(a);
    let a_transpose =
        transpose::transpose_for_fixed_msm(&a_array, 0, num_outputs, num_columns, data_size);

    let mut blitzar_commits = vec![ElementP2::<ark_bls12_381::g1::Config>::default(); num_outputs];

    prover_setup.blitzar_msm(
        &mut blitzar_commits,
        data_size as u32,
        a_transpose.as_slice(),
    );

    slice_cast(&blitzar_commits)
}

#[tracing::instrument(level = "debug", skip_all)]
#[cfg(not(feature = "blitzar"))]
pub(super) fn compute_T_vec_prime(
    a: &[F],
    sigma: usize,
    nu: usize,
    prover_setup: &ProverSetup,
) -> Vec<G1Affine> {
    a.chunks(1 << sigma)
        .map(|row| G1Projective::msm_unchecked(prover_setup.Gamma_1[nu], row).into())
        .chain(core::iter::repeat(G1Affine::zero()))
        .take(1 << nu)
        .collect()
}

/// Compute the size of the matrix M that is derived from `a`.
/// More specifically compute `nu`, where 2^nu is the side length the square matrix, M.
/// `num_vars` is the number of variables in the polynomial. In other words, it is the length of `b_points`, which is `ceil(log2(len(a)))`.
pub(super) fn compute_nu(num_vars: usize, sigma: usize) -> usize {
    if num_vars <= sigma * 2 {
        // This is the scenario where we don't need to pad the columns.
        sigma
    } else {
        // This is the scenario where we need to pad the columns.
        num_vars - sigma
    }
}

/// Compute the vectors L and R that are derived from `b_point`.
/// L and R are the vectors such that LMR is exactly the evaluation of `a` at the point `b_point`.
pub(super) fn compute_L_R_vec(b_point: &[F], sigma: usize, nu: usize) -> (Vec<F>, Vec<F>) {
    let mut R_vec = vec![Zero::zero(); 1 << nu];
    let mut L_vec = vec![Zero::zero(); 1 << nu];
    let num_vars = b_point.len();
    if num_vars == 0 {
        // This is the scenario where we only need a single element in the matrix.
        R_vec[0] = One::one();
        L_vec[0] = One::one();
    } else if num_vars <= sigma {
        // This is the scenario where we only need a single row of the matrix.
        compute_evaluation_vector(&mut R_vec[..1 << num_vars], b_point);
        L_vec[0] = One::one();
    } else if num_vars <= sigma * 2 {
        // This is the scenario where we need more than a single row, but we don't need to pad the columns.
        compute_evaluation_vector(&mut R_vec, &b_point[..nu]);
        compute_evaluation_vector(&mut L_vec[..1 << (num_vars - nu)], &b_point[nu..]);
    } else {
        // This is the scenario where we need to pad the columns.
        compute_evaluation_vector(&mut R_vec[..(1 << sigma)], &b_point[..sigma]);
        compute_evaluation_vector(&mut L_vec, &b_point[sigma..]);
    }

    (L_vec, R_vec)
}

/// Compute the l and r tensors that are derived from `b_point`.
/// These match with [`compute_L_R_vec`] but are in tensor form.
pub(super) fn compute_l_r_tensors(b_point: &[F], sigma: usize, nu: usize) -> (Vec<F>, Vec<F>) {
    let mut r_tensor = vec![Zero::zero(); nu];
    let mut l_tensor = vec![Zero::zero(); nu];
    let num_vars = b_point.len();
    if num_vars == 0 {
        // This is the scenario where we only need a single element in the matrix.
    } else if num_vars <= sigma {
        // This is the scenario where we only need a single row of the matrix.
        r_tensor[..num_vars].copy_from_slice(b_point);
    } else if num_vars <= sigma * 2 {
        // This is the scenario where we need more than a single row, but we don't need to pad the columns.
        r_tensor.copy_from_slice(&b_point[..nu]);
        l_tensor[..(num_vars - nu)].copy_from_slice(&b_point[nu..]);
    } else {
        // This is the scenario where we need to pad the columns.
        r_tensor[..sigma].copy_from_slice(&b_point[..sigma]);
        l_tensor.copy_from_slice(&b_point[sigma..]);
    }

    (l_tensor, r_tensor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::UniformRand;
    use core::iter::repeat_with;

    /// This method is simply computing the product LMR where M is the matrix filled, row by row, with `a` but with row length 2^sigma.
    /// Note: because L and R are the same length, and have length at least 2^sigma, we simply pad everything else in the matrix with zeros.
    fn compute_LMR(a: &[F], L: &[F], R: &[F], sigma: usize) -> F {
        assert_eq!(L.len(), R.len());
        assert!(L.len() >= 1 << sigma);
        assert!(R.len() >= 1 << sigma);

        let num_columns = 1 << sigma;
        let M = a.chunks(num_columns);
        M.zip(L)
            .map(|(row, l)| row.iter().zip(R).map(|(a, r)| l * a * r).sum::<F>())
            .sum()
    }
    /// This is the naive inner product. It is used to check the correctness of the `compute_LMR` method.
    fn compute_ab_inner_product(a: &[F], b_point: &[F]) -> F {
        let mut b = vec![Default::default(); a.len()];
        compute_evaluation_vector(&mut b, b_point);
        a.iter().zip(b.iter()).map(|(a, b)| a * b).sum()
    }
    fn check_L_R_with_random_a(b_point: &[F], L: &[F], R: &[F], sigma: usize) {
        let rng = &mut ark_std::test_rng();
        let a: Vec<_> = repeat_with(|| F::rand(rng))
            .take(1 << b_point.len())
            .collect();
        let LMR = compute_LMR(&a, L, R, sigma);
        let ab = compute_ab_inner_product(&a, b_point);
        assert_eq!(LMR, ab);
    }
    fn check_L_R_vecs_with_l_r_tensors(L: &[F], R: &[F], l: &[F], r: &[F]) {
        let mut l_vec = vec![Default::default(); 1 << l.len()];
        let mut r_vec = vec![Default::default(); 1 << r.len()];
        compute_evaluation_vector(&mut l_vec, l);
        compute_evaluation_vector(&mut r_vec, r);
        assert_eq!(l_vec, L);
        assert_eq!(r_vec, R);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_0() {
        let b_point = vec![];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_L_R_vec(&b_point, sigma, nu);
        assert_eq!(L_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);
        assert_eq!(R_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);

        let (l_tensor, r_tensor) = compute_l_r_tensors(&b_point, sigma, nu);
        check_L_R_vecs_with_l_r_tensors(&L_vec, &R_vec, &l_tensor, &r_tensor);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_positive_and_less_than_sigma() {
        let b_point = vec![F::from(10)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_L_R_vec(&b_point, sigma, nu);
        assert_eq!(L_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);
        assert_eq!(
            R_vec,
            vec![F::from(1 - 10), F::from(10), F::from(0), F::from(0)]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);

        let (l_tensor, r_tensor) = compute_l_r_tensors(&b_point, sigma, nu);
        check_L_R_vecs_with_l_r_tensors(&L_vec, &R_vec, &l_tensor, &r_tensor);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_equals_sigma() {
        let b_point = vec![F::from(10), F::from(20)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_L_R_vec(&b_point, sigma, nu);
        assert_eq!(L_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);

        let (l_tensor, r_tensor) = compute_l_r_tensors(&b_point, sigma, nu);
        check_L_R_vecs_with_l_r_tensors(&L_vec, &R_vec, &l_tensor, &r_tensor);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_more_than_sigma_but_less_than_2sigma() {
        let b_point = vec![F::from(10), F::from(20), F::from(30)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_L_R_vec(&b_point, sigma, nu);
        assert_eq!(
            L_vec,
            vec![F::from(1 - 30), F::from(30), F::from(0), F::from(0)]
        );
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);

        let (l_tensor, r_tensor) = compute_l_r_tensors(&b_point, sigma, nu);
        check_L_R_vecs_with_l_r_tensors(&L_vec, &R_vec, &l_tensor, &r_tensor);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_equals_2_sigma() {
        let b_point = vec![F::from(10), F::from(20), F::from(30), F::from(40)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_L_R_vec(&b_point, sigma, nu);
        assert_eq!(
            L_vec,
            vec![
                F::from((1 - 30) * (1 - 40)),
                F::from(30 * (1 - 40)),
                F::from((1 - 30) * 40),
                F::from(30 * 40),
            ]
        );
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);

        let (l_tensor, r_tensor) = compute_l_r_tensors(&b_point, sigma, nu);
        check_L_R_vecs_with_l_r_tensors(&L_vec, &R_vec, &l_tensor, &r_tensor);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_more_than_2_sigma() {
        let b_point = vec![
            F::from(10),
            F::from(20),
            F::from(30),
            F::from(40),
            F::from(50),
        ];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 3);

        let (L_vec, R_vec) = compute_L_R_vec(&b_point, sigma, nu);
        assert_eq!(
            L_vec,
            vec![
                F::from((1 - 30) * (1 - 40) * (1 - 50)),
                F::from(30 * (1 - 40) * (1 - 50)),
                F::from((1 - 30) * 40 * (1 - 50)),
                F::from(30 * 40 * (1 - 50)),
                F::from((1 - 30) * (1 - 40) * 50),
                F::from(30 * (1 - 40) * 50),
                F::from((1 - 30) * 40 * 50),
                F::from(30 * 40 * 50),
            ]
        );
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
                F::from(0),
                F::from(0),
                F::from(0),
                F::from(0)
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);

        let (l_tensor, r_tensor) = compute_l_r_tensors(&b_point, sigma, nu);
        check_L_R_vecs_with_l_r_tensors(&L_vec, &R_vec, &l_tensor, &r_tensor);
    }
}
