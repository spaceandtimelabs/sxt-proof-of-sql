use super::{
    blitzar_metadata_table::create_blitzar_metadata_tables, ExtendedVerifierState, G1Affine,
    ProverSetup, F,
};
use crate::{
    base::{commitment::CommittableColumn, scalar::MontScalar, slice_ops::slice_cast},
    proof_primitive::{
        dory::{
            dynamic_dory_standard_basis_helper::compute_dynamic_standard_basis_vecs, DoryScalar,
        },
        dynamic_matrix_utils::{
            matrix_structure::row_and_column_from_index,
            standard_basis_helper::fold_dynamic_standard_basis_tensors,
        },
    },
};
use alloc::{vec, vec::Vec};
use ark_ff::{AdditiveGroup, Field};
#[cfg(feature = "blitzar")]
use blitzar::compute::ElementP2;
#[cfg(feature = "blitzar")]
use bytemuck::TransparentWrapper;
use itertools::{Itertools, __std_iter::repeat};

/// Compute the evaluations of the columns of the matrix M that is derived from `a`.
///
/// In this context `hi_vec` is the left `L` vector in the vector-matrix-vector product LMR.
///
/// `1 << nu` is the side length of M.
///
/// # Panics
///
/// This function requires that `hi_vec` has length at least as big as the number of rows in `M` that is created by `a`.
/// In practice, `hi_vec` is normally length `1 << nu`.
pub(super) fn compute_dynamic_v_vec(a: &[F], hi_vec: &[F], nu: usize) -> Vec<F> {
    a.iter()
        .enumerate()
        .fold(vec![F::ZERO; 1 << nu], |mut v_vec, (i, v)| {
            let (row, column) = row_and_column_from_index(i);
            v_vec[column] += hi_vec[row] * v;
            v_vec
        })
}

/// Compute the commitments to the rows of the matrix M that is derived from `a`.
#[cfg(not(feature = "blitzar"))]
pub(super) fn compute_dynamic_T_vec_prime(
    a: &[F],
    nu: usize,
    prover_setup: &ProverSetup,
) -> Vec<G1Affine> {
    let mut T_vec_prime = vec![G1Affine::identity(); 1 << nu];
    for (i, v) in a.iter().enumerate() {
        let (row, column) = row_and_column_from_index(i);
        T_vec_prime[row] = (T_vec_prime[row] + prover_setup.Gamma_1[nu][column] * v).into();
    }
    T_vec_prime
}

/// Compute the commitments to the rows of the matrix M that is derived from `a`.
#[cfg(feature = "blitzar")]
pub(super) fn compute_dynamic_T_vec_prime(
    a: &[F],
    nu: usize,
    prover_setup: &ProverSetup,
) -> Vec<G1Affine> {
    let a_col = CommittableColumn::from(TransparentWrapper::wrap_slice(a) as &[DoryScalar]);

    let (blitzar_output_bit_table, blitzar_output_length_table, blitzar_scalars) =
        create_blitzar_metadata_tables(&[a_col], 0);

    let mut blitzar_sub_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); blitzar_output_bit_table.len()];

    prover_setup.blitzar_vlen_msm(
        &mut blitzar_sub_commits,
        &blitzar_output_bit_table,
        &blitzar_output_length_table,
        blitzar_scalars.as_slice(),
    );

    let all_sub_commits: Vec<G1Affine> = slice_cast(&blitzar_sub_commits);

    all_sub_commits
        .iter()
        .step_by(2)
        .chain(repeat(&G1Affine::identity()))
        .take(1 << nu)
        .copied()
        .collect()
}

/// Compute the size of the matrix M that is derived from `a`.
/// More specifically compute `nu`, where 2^nu is the side length the square matrix, M.
/// `num_vars` is the number of variables in the polynomial. In other words, it is the length of `b_points`, which is `ceil(log2(len(a)))`.
pub(super) fn compute_dynamic_nu(num_vars: usize) -> usize {
    num_vars / 2 + 1
}

/// Compute the hi and lo vectors (or L and R) that are derived from `point`.
/// L and R are the vectors such that LMR is exactly the evaluation of `a` at the point `point`.
/// # Panics
/// This function requires that `point` has length at least as big as the number of rows in `M` that is created by `a`.
pub(super) fn compute_dynamic_vecs(point: &[F]) -> (Vec<F>, Vec<F>) {
    let nu = point.len() / 2 + 1;
    let mut lo_vec = vec![F::ZERO; 1 << nu];
    let mut hi_vec = vec![F::ZERO; 1 << nu];
    lo_vec[0] = point.iter().take(nu).map(|b| F::ONE - b).product();
    hi_vec[0] = point.iter().skip(nu).map(|b| F::ONE - b).product();
    let standard_basis_point = point
        .iter()
        .map(|b| {
            (F::ONE - b)
                .inverse()
                .expect("Values in point cannot be 1.")
                - F::ONE
        })
        .collect_vec();
    compute_dynamic_standard_basis_vecs(&standard_basis_point, &mut lo_vec, &mut hi_vec);
    (lo_vec, hi_vec)
}

/// Folds the `s1` and `s2` tensors:
///
/// This is the analogous function of the non-dynamic folding function [`extended_dory_reduce_verify_fold_s_vecs`](super::extended_dory_reduce_helper::extended_dory_reduce_verify_fold_s_vecs).
/// See that method for more details.
/// # Panics
/// This function requires that `point` has length at least as big as the number of rows in `M` that is created by `a`. In practice, `point` is normally length `1 << nu`.      
pub(super) fn fold_dynamic_tensors(state: &ExtendedVerifierState) -> (F, F) {
    let point = &state.s1_tensor;
    let nu = point.len() / 2 + 1;
    let lo_inv_prod: F = point.iter().take(nu).map(|b| F::ONE - b).product();
    let hi_inv_prod: F = point.iter().skip(nu).map(|b| F::ONE - b).product();
    let standard_basis_point = point
        .iter()
        .map(|b| {
            (F::ONE - b)
                .inverse()
                .expect("Values in point cannot be 1.")
                - F::ONE
        })
        .collect_vec();
    let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(
        &standard_basis_point
            .iter()
            .copied()
            .map(MontScalar)
            .collect_vec(),
        &state.alphas.iter().copied().map(MontScalar).collect_vec(),
        &state
            .alpha_invs
            .iter()
            .copied()
            .map(MontScalar)
            .collect_vec(),
    );
    (lo_fold.0 * lo_inv_prod, hi_fold.0 * hi_inv_prod)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_primitive::{
        dory::{deferred_msm::DeferredMSM, test_rng, PublicParameters, VerifierState},
        dynamic_matrix_utils::standard_basis_helper::{compute_dynamic_vecs, tests::naive_fold},
    };

    #[test]
    fn we_can_fold_dynamic_tensors() {
        use ark_std::{test_rng, UniformRand};
        use itertools::Itertools;
        let mut rng = test_rng();
        for num_vars in 0..10 {
            let nu = num_vars / 2 + 1;
            let point: Vec<_> = core::iter::repeat_with(|| F::rand(&mut rng))
                .take(num_vars)
                .collect();
            let alphas = core::iter::repeat_with(|| F::rand(&mut rng))
                .take(nu)
                .collect_vec();
            let alpha_invs = core::iter::repeat_with(|| F::rand(&mut rng))
                .take(nu)
                .collect_vec();

            let (mut lo_vec, mut hi_vec) =
                compute_dynamic_vecs(&point.iter().copied().map(MontScalar).collect_vec());
            naive_fold(
                &mut lo_vec,
                &alphas.iter().copied().map(MontScalar).collect_vec(),
            );
            naive_fold(
                &mut hi_vec,
                &alpha_invs.iter().copied().map(MontScalar).collect_vec(),
            );

            let state = ExtendedVerifierState {
                s1_tensor: point,
                alphas: alphas.clone(),
                alpha_invs: alpha_invs.clone(),
                // Unused values in the struct:
                E_1: DeferredMSM::new([], []),
                E_2: DeferredMSM::new([], []),
                base_state: VerifierState {
                    C: DeferredMSM::new([], []),
                    D_1: DeferredMSM::new([], []),
                    D_2: DeferredMSM::new([], []),
                    nu,
                },
                s2_tensor: Vec::default(),
            };
            let (lo_fold, hi_fold) = fold_dynamic_tensors(&state);

            assert_eq!(lo_fold, lo_vec[0].0);
            assert_eq!(hi_fold, hi_vec[0].0);
        }
    }

    #[test]
    fn we_can_compute_dynamic_v_vec() {
        let a: Vec<F> = (100..109).map(Into::into).collect();
        let hi_vec: Vec<F> = (200..208).map(Into::into).collect();
        let nu = 3;
        let v_vec = compute_dynamic_v_vec(&a, &hi_vec, nu);

        // 100
        //   _, 101
        // 102, 103
        // 104, 105, 106, 107
        // 108

        let expected_v_vec: Vec<F> = [
            100 * 200 + 102 * 202 + 104 * 203 + 108 * 204,
            101 * 201 + 103 * 202 + 105 * 203,
            106 * 203,
            107 * 203,
            0,
            0,
            0,
            0,
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(v_vec, expected_v_vec);
    }

    #[test]
    fn we_can_compute_dynamic_T_vec_prime() {
        let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
        let prover_setup = ProverSetup::from(&public_parameters);

        let a: Vec<F> = (100..109).map(Into::into).collect();
        let nu = 3;
        let T_vec_prime = compute_dynamic_T_vec_prime(&a, nu, &prover_setup);

        // 100
        //   _, 101
        // 102, 103
        // 104, 105, 106, 107
        // 108

        let expected_T_vec_prime = vec![
            (prover_setup.Gamma_1[nu][0] * F::from(100)).into(),
            (prover_setup.Gamma_1[nu][1] * F::from(101)).into(),
            (prover_setup.Gamma_1[nu][0] * F::from(102)
                + prover_setup.Gamma_1[nu][1] * F::from(103))
            .into(),
            (prover_setup.Gamma_1[nu][0] * F::from(104)
                + prover_setup.Gamma_1[nu][1] * F::from(105)
                + prover_setup.Gamma_1[nu][2] * F::from(106)
                + prover_setup.Gamma_1[nu][3] * F::from(107))
            .into(),
            (prover_setup.Gamma_1[nu][0] * F::from(108)).into(),
            G1Affine::identity(),
            G1Affine::identity(),
            G1Affine::identity(),
        ];
        assert_eq!(T_vec_prime, expected_T_vec_prime);
    }
}
