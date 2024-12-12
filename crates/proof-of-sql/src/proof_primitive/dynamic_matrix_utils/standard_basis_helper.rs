use crate::base::scalar::Scalar;
use alloc::{vec, vec::Vec};

/// Compute the hi and lo vectors (or L and R) that are derived from `point`.
/// L and R are the vectors such that LMR is exactly the evaluation of `a` at the point `point`.
/// # Panics
/// This function requires that `point` has length at least as big as the number of rows in `M` that is created by `a`.
pub(crate) fn compute_dynamic_vecs<S: Scalar>(point: &[S]) -> (Vec<S>, Vec<S>) {
    let nu = point.len() / 2 + 1;
    let mut lo_vec = vec![S::ZERO; 1 << nu];
    let mut hi_vec = vec![S::ZERO; 1 << nu];
    lo_vec[0] = point.iter().take(nu).map(|b| S::ONE - *b).product();
    hi_vec[0] = point.iter().skip(nu).map(|b| S::ONE - *b).product();
    let standard_basis_point: Vec<S> = point
        .iter()
        .map(|b| (S::ONE - *b).inv().expect("Values in point cannot be 1.") - S::ONE)
        .collect();
    compute_dynamic_standard_basis_vecs(&standard_basis_point, &mut lo_vec, &mut hi_vec);
    (lo_vec, hi_vec)
}

/// This method produces evaluation vectors from a point. This is a helper method for generating a Vector-Matrix-Vector product in the dynamic dory and hyrax schemes.
///
/// The ith element of the `lo_vec` is essentially the ith monomial basis element (lexicographically).
/// The ith element of the `hi_vec` is essentially the jth monomial basis element where `j = row_start_index(i)`.
///
/// NOTE: the `lo_vec` and `hi_vec` are scaled by `lo_vec[0]` and `hi_vec[0]` respectively.
/// NOTE: `lo_vec` and `hi_vec` should otherwise consist entirely of zeros in order to ensure correct output.
fn compute_dynamic_standard_basis_vecs<S: Scalar>(point: &[S], lo_vec: &mut [S], hi_vec: &mut [S]) {
    let nu = point.len() / 2 + 1;
    debug_assert_eq!(lo_vec.len(), 1 << nu);
    debug_assert_eq!(hi_vec.len(), 1 << nu);
    for i in 1..nu {
        build_partial_second_half_standard_basis_vecs(
            &point[..2 * i - 1],
            &mut lo_vec[..1 << i],
            &mut hi_vec[..1 << i],
            true,
        );
    }
    // Note: if we don't have the "full" point, we shouldn't fill up the last quarter because it should be all zeros.
    build_partial_second_half_standard_basis_vecs(point, lo_vec, hi_vec, point.len() % 2 == 1);
    // Add the most significant variable, which was not included before in order to allow simple copying to work.
    point.iter().skip(1).enumerate().for_each(|(i, v)| {
        let p = i / 2;
        let o = 2 + i % 2;
        (o << p..(o + 1) << p).for_each(|k| hi_vec[k] *= *v);
    });
}

fn build_partial_second_half_standard_basis_vecs<S: Scalar>(
    point: &[S],
    lo_vec: &mut [S],
    hi_vec: &mut [S],
    add_last_quarter: bool,
) {
    let nu = point.len() / 2 + 1;
    debug_assert_eq!(lo_vec.len(), 1 << nu);
    debug_assert_eq!(hi_vec.len(), 1 << nu);
    if nu == 1 {
        lo_vec[1] = if point.is_empty() {
            S::ZERO
        } else {
            lo_vec[0] * point[0]
        };
        hi_vec[1] = hi_vec[0];
    } else {
        let (lo_half0, lo_half1) = lo_vec.split_at_mut(1 << (nu - 1));
        lo_half0
            .iter()
            .zip(lo_half1)
            .for_each(|(l, h)| *h = *l * point[nu - 1]);
        if nu == 2 {
            hi_vec[2] = hi_vec[0];
            if add_last_quarter {
                hi_vec[3] = hi_vec[1];
            }
        } else {
            let (hi_half0, hi_half1) = hi_vec.split_at_mut(1 << (nu - 1));
            let (_, hi_quarter1) = hi_half0.split_at(1 << (nu - 2));
            let (hi_quarter2, hi_quarter3) = hi_half1.split_at_mut(1 << (nu - 2));
            let (_, hi_eighth3) = hi_quarter1.split_at(1 << (nu - 3));
            let (hi_eighth4, hi_eighth5) = hi_quarter2.split_at_mut(1 << (nu - 3));
            let (hi_eighth6, hi_eighth7) = hi_quarter3.split_at_mut(1 << (nu - 3));
            // Fill up quarter #2 (from 2/4..3/4).
            hi_eighth3
                .iter()
                .zip(hi_eighth4.iter_mut().zip(hi_eighth5))
                .for_each(|(&source, (target_lo, target_hi))| {
                    // Copy eighth #3 (from 3/8..4/8) to eighth #4 (4/8..5/8).
                    *target_lo = source;
                    // Copy eighth #3 (from 3/8..4/8) to eighth #5 (5/8..6/8)
                    // and multiply by the third from the last element in point.
                    *target_hi = source * point[2 * nu - 4];
                });
            if add_last_quarter {
                // Fill up quarter #4 (from 3/4..4/4).
                hi_quarter2
                    .iter()
                    .step_by(2)
                    .zip(hi_eighth6.iter_mut().zip(hi_eighth7))
                    .for_each(|(&source, (target_lo, target_hi))| {
                        // Copy every other in quarter #2 (from 2/4..3/4) to eighth #6 (6/8..7/8).
                        *target_lo = source;
                        // Copy every other in quarter #2 (from 2/4..3/4) to eighth #6 (7/8..8/8).
                        // and multiply by the second from the last element in point.
                        *target_hi = source * point[2 * nu - 3];
                    });
            }
        }
    }
}

pub(crate) fn fold_dynamic_standard_basis_tensors<S: Scalar>(
    point: &[S],
    alphas: &[S],
    alpha_invs: &[S],
) -> (S, S) {
    let nu = point.len() / 2 + 1;
    debug_assert_eq!(alphas.len(), nu);
    debug_assert_eq!(alpha_invs.len(), nu);
    let lo_fold = if point.is_empty() {
        alphas[0]
    } else {
        point.iter().zip(alphas).map(|(v, a)| *v + *a).product()
    };
    let hi_fold = point
        .iter()
        .enumerate()
        .fold(
            (alpha_invs[0] + S::ONE, S::ZERO),
            |(acc, prev_partial), (i, &p)| {
                if i == 0 {
                    (acc, S::ZERO)
                } else if i == 1 {
                    (acc * alpha_invs[1] + p * alpha_invs[0], S::ZERO)
                } else if i % 2 == 0 {
                    let partial = (i / 2 + 1..i)
                        .zip(alpha_invs)
                        .map(|(k, a)| point[k] + *a)
                        .product();
                    (acc + p * partial, partial)
                } else {
                    (
                        acc * alpha_invs[i / 2 + 1]
                            + p * alpha_invs[i / 2]
                                * (point[i - 1] + alpha_invs[i / 2 - 1])
                                * prev_partial,
                        S::ZERO,
                    )
                }
            },
        )
        .0;
    (lo_fold, hi_fold)
}

#[cfg(test)]
pub(crate) mod tests {

    use super::*;
    use crate::{
        base::{polynomial::compute_evaluation_vector, scalar::MontScalar},
        proof_primitive::{
            dory::DoryScalar,
            dynamic_matrix_utils::{
                matrix_structure::{row_and_column_from_index, row_start_index},
                standard_basis_helper::{
                    compute_dynamic_standard_basis_vecs, fold_dynamic_standard_basis_tensors,
                },
            },
        },
    };
    use ark_bls12_381::Fr as F;
    use ark_ff::AdditiveGroup;

    pub fn naive_fold<S: Scalar>(mut vec: &mut [S], fold_factors: &[S]) {
        let nu = fold_factors.len();
        assert_eq!(vec.len(), 1 << fold_factors.len());
        for i in (0..nu).rev() {
            let (lo, hi) = vec.split_at_mut(vec.len() / 2);
            lo.iter_mut().zip(hi).for_each(|(l, h)| {
                *l *= fold_factors[i];
                *l += *h;
            });
            vec = lo;
        }
    }

    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_0_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 2];
        let mut hi_vec = vec![DoryScalar::ZERO; 2];
        lo_vec[0] = DoryScalar::from(2);
        hi_vec[0] = DoryScalar::from(3);
        let point = vec![];
        let lo_vec_expected = vec![DoryScalar::from(2), DoryScalar::ZERO];
        let hi_vec_expected = vec![DoryScalar::from(3), DoryScalar::from(3)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_1_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 2];
        let mut hi_vec = vec![DoryScalar::ZERO; 2];
        lo_vec[0] = DoryScalar::from(2);
        hi_vec[0] = DoryScalar::from(3);
        let point = vec![DoryScalar::from(5)];
        let lo_vec_expected = vec![DoryScalar::from(2), DoryScalar::from(2 * 5)];
        let hi_vec_expected = vec![DoryScalar::from(3), DoryScalar::from(3)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_2_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 4];
        let mut hi_vec = vec![DoryScalar::ZERO; 4];
        lo_vec[0] = DoryScalar::from(2);
        hi_vec[0] = DoryScalar::from(3);
        let point = vec![DoryScalar::from(5), DoryScalar::from(7)];
        let lo_vec_expected = vec![
            DoryScalar::from(2),
            DoryScalar::from(2 * 5),
            DoryScalar::from(2 * 7),
            DoryScalar::from(2 * 5 * 7),
        ];
        let hi_vec_expected = vec![
            DoryScalar::from(3),
            DoryScalar::from(3),
            DoryScalar::from(3 * 7),
            DoryScalar::ZERO,
        ];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_3_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 4];
        let mut hi_vec = vec![DoryScalar::ZERO; 4];
        lo_vec[0] = DoryScalar::from(2);
        hi_vec[0] = DoryScalar::from(3);
        let point = vec![
            DoryScalar::from(5),
            DoryScalar::from(7),
            DoryScalar::from(11),
        ];
        let lo_vec_expected = vec![
            DoryScalar::from(2),
            DoryScalar::from(2 * 5),
            DoryScalar::from(2 * 7),
            DoryScalar::from(2 * 5 * 7),
        ];
        let hi_vec_expected = vec![
            DoryScalar::from(3),
            DoryScalar::from(3),
            DoryScalar::from(3 * 7),
            DoryScalar::from(3 * 11),
        ];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_4_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 8];
        let mut hi_vec = vec![DoryScalar::ZERO; 8];
        lo_vec[0] = DoryScalar::from(2);
        hi_vec[0] = DoryScalar::from(3);
        let point = vec![
            DoryScalar::from(5),
            DoryScalar::from(7),
            DoryScalar::from(11),
            DoryScalar::from(13),
        ];
        let lo_vec_expected = vec![
            DoryScalar::from(2),
            DoryScalar::from(2 * 5),
            DoryScalar::from(2 * 7),
            DoryScalar::from(2 * 5 * 7),
            DoryScalar::from(2 * 11),
            DoryScalar::from(2 * 5 * 11),
            DoryScalar::from(2 * 7 * 11),
            DoryScalar::from(2 * 5 * 7 * 11),
        ];
        let hi_vec_expected = vec![
            DoryScalar::from(3),
            DoryScalar::from(3),
            DoryScalar::from(3 * 7),
            DoryScalar::from(3 * 11),
            DoryScalar::from(3 * 13),
            DoryScalar::from(3 * 11 * 13),
            DoryScalar::ZERO,
            DoryScalar::ZERO,
        ];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_5_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 8];
        let mut hi_vec = vec![DoryScalar::ZERO; 8];
        lo_vec[0] = DoryScalar::from(2);
        hi_vec[0] = DoryScalar::from(3);
        let point = vec![
            DoryScalar::from(5),
            DoryScalar::from(7),
            DoryScalar::from(11),
            DoryScalar::from(13),
            DoryScalar::from(17),
        ];
        let lo_vec_expected = vec![
            DoryScalar::from(2),
            DoryScalar::from(2 * 5),
            DoryScalar::from(2 * 7),
            DoryScalar::from(2 * 5 * 7),
            DoryScalar::from(2 * 11),
            DoryScalar::from(2 * 5 * 11),
            DoryScalar::from(2 * 7 * 11),
            DoryScalar::from(2 * 5 * 7 * 11),
        ];
        let hi_vec_expected = vec![
            DoryScalar::from(3),
            DoryScalar::from(3),
            DoryScalar::from(3 * 7),
            DoryScalar::from(3 * 11),
            DoryScalar::from(3 * 13),
            DoryScalar::from(3 * 11 * 13),
            DoryScalar::from(3 * 17),
            DoryScalar::from(3 * 13 * 17),
        ];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }

    /// Computes the evaluation of a basis monomial at the given point.
    ///
    /// In other words, the result is `prod point[i]^(b[i])` where
    /// `index = sum 2^i*b[i]` and `b[i]` is `0` or `1`. (i.e. `b` is the binary representation of `index`.)
    /// Note: `point` is padded with zeros as needed.
    ///
    /// This method is primarily to test the `build_standard_basis_vecs` method.
    fn get_binary_eval(index: usize, point: &[F]) -> F {
        core::iter::successors(Some(index), |&k| match k >> 1 {
            0 => None,
            k => Some(k),
        })
        .enumerate()
        .filter_map(|(i, b)| {
            if b % 2 == 0 {
                None
            } else {
                Some(point.get(i).copied().unwrap_or(F::ZERO))
            }
        })
        .product()
    }

    #[test]
    fn we_can_compute_dynamic_random_standard_basis_vecs() {
        use ark_std::{test_rng, UniformRand};
        use itertools::Itertools;
        let mut rng = test_rng();
        for num_vars in 0..10 {
            let point = core::iter::repeat_with(|| F::rand(&mut rng))
                .take(num_vars)
                .collect_vec();
            let alpha = MontScalar(F::rand(&mut rng));
            let beta = MontScalar(F::rand(&mut rng));
            let nu = point.len() / 2 + 1;
            let mut lo_vec = vec![DoryScalar::ZERO; 1 << nu];
            let mut hi_vec = vec![DoryScalar::ZERO; 1 << nu];
            lo_vec[0] = alpha;
            hi_vec[0] = beta;
            compute_dynamic_standard_basis_vecs(
                bytemuck::TransparentWrapper::wrap_slice(&point) as &[DoryScalar],
                &mut lo_vec,
                &mut hi_vec,
            );
            for i in 0..1 << nu {
                assert_eq!(lo_vec[i], alpha * MontScalar(get_binary_eval(i, &point)));
                assert_eq!(
                    hi_vec[i],
                    beta * MontScalar(get_binary_eval(row_start_index(i), &point))
                );
            }
        }
    }

    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_0_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 2];
        let mut hi_vec = vec![DoryScalar::ZERO; 2];
        lo_vec[0] = DoryScalar::ONE;
        hi_vec[0] = DoryScalar::ONE;
        let point = vec![];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![DoryScalar::from(200)];
        let alpha_invs = vec![DoryScalar::from(300)];
        let lo_fold_expected = lo_vec[0] * DoryScalar::from(200) + lo_vec[1];
        let hi_fold_expected = hi_vec[0] * DoryScalar::from(300) + hi_vec[1];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_1_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 2];
        let mut hi_vec = vec![DoryScalar::ZERO; 2];
        lo_vec[0] = DoryScalar::ONE;
        hi_vec[0] = DoryScalar::ONE;
        let point = vec![DoryScalar::from(5)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![DoryScalar::from(200)];
        let alpha_invs = vec![DoryScalar::from(300)];
        let lo_fold_expected = lo_vec[0] * DoryScalar::from(200) + lo_vec[1];
        let hi_fold_expected = hi_vec[0] * DoryScalar::from(300) + hi_vec[1];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_2_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 4];
        let mut hi_vec = vec![DoryScalar::ZERO; 4];
        lo_vec[0] = DoryScalar::ONE;
        hi_vec[0] = DoryScalar::ONE;
        let point = vec![DoryScalar::from(5), DoryScalar::from(7)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![DoryScalar::from(200), DoryScalar::from(201)];
        let alpha_invs = vec![DoryScalar::from(300), DoryScalar::from(301)];
        let lo_fold_expected = lo_vec[0] * DoryScalar::from(200 * 201)
            + lo_vec[1] * DoryScalar::from(201)
            + lo_vec[2] * DoryScalar::from(200)
            + lo_vec[3];
        let hi_fold_expected = hi_vec[0] * DoryScalar::from(300 * 301)
            + hi_vec[1] * DoryScalar::from(301)
            + hi_vec[2] * DoryScalar::from(300)
            + hi_vec[3];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_3_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 4];
        let mut hi_vec = vec![DoryScalar::ZERO; 4];
        lo_vec[0] = DoryScalar::ONE;
        hi_vec[0] = DoryScalar::ONE;
        let point = vec![
            DoryScalar::from(5),
            DoryScalar::from(7),
            DoryScalar::from(11),
        ];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![DoryScalar::from(200), DoryScalar::from(201)];
        let alpha_invs = vec![DoryScalar::from(300), DoryScalar::from(301)];
        let lo_fold_expected = lo_vec[0] * DoryScalar::from(200 * 201)
            + lo_vec[1] * DoryScalar::from(201)
            + lo_vec[2] * DoryScalar::from(200)
            + lo_vec[3];
        let hi_fold_expected = hi_vec[0] * DoryScalar::from(300 * 301)
            + hi_vec[1] * DoryScalar::from(301)
            + hi_vec[2] * DoryScalar::from(300)
            + hi_vec[3];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_4_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 8];
        let mut hi_vec = vec![DoryScalar::ZERO; 8];
        lo_vec[0] = DoryScalar::ONE;
        hi_vec[0] = DoryScalar::ONE;
        let point = vec![
            DoryScalar::from(5),
            DoryScalar::from(7),
            DoryScalar::from(11),
            DoryScalar::from(13),
        ];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![
            DoryScalar::from(200),
            DoryScalar::from(201),
            DoryScalar::from(202),
        ];
        let alpha_invs = vec![
            DoryScalar::from(300),
            DoryScalar::from(301),
            DoryScalar::from(302),
        ];
        let lo_fold_expected = lo_vec[0] * DoryScalar::from(200 * 201 * 202)
            + lo_vec[1] * DoryScalar::from(201 * 202)
            + lo_vec[2] * DoryScalar::from(200 * 202)
            + lo_vec[3] * DoryScalar::from(202)
            + lo_vec[4] * DoryScalar::from(200 * 201)
            + lo_vec[5] * DoryScalar::from(201)
            + lo_vec[6] * DoryScalar::from(200)
            + lo_vec[7];
        let hi_fold_expected = hi_vec[0] * DoryScalar::from(300 * 301 * 302)
            + hi_vec[1] * DoryScalar::from(301 * 302)
            + hi_vec[2] * DoryScalar::from(300 * 302)
            + hi_vec[3] * DoryScalar::from(302)
            + hi_vec[4] * DoryScalar::from(300 * 301)
            + hi_vec[5] * DoryScalar::from(301)
            + hi_vec[6] * DoryScalar::from(300)
            + hi_vec[7];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_5_point() {
        let mut lo_vec = vec![DoryScalar::ZERO; 8];
        let mut hi_vec = vec![DoryScalar::ZERO; 8];
        lo_vec[0] = DoryScalar::ONE;
        hi_vec[0] = DoryScalar::ONE;
        let point = [5, 7, 11, 13, 17].map(DoryScalar::from);
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![
            DoryScalar::from(200),
            DoryScalar::from(201),
            DoryScalar::from(202),
        ];
        let alpha_invs = vec![
            DoryScalar::from(300),
            DoryScalar::from(301),
            DoryScalar::from(302),
        ];
        let lo_fold_expected = lo_vec[0] * DoryScalar::from(200 * 201 * 202)
            + lo_vec[1] * DoryScalar::from(201 * 202)
            + lo_vec[2] * DoryScalar::from(200 * 202)
            + lo_vec[3] * DoryScalar::from(202)
            + lo_vec[4] * DoryScalar::from(200 * 201)
            + lo_vec[5] * DoryScalar::from(201)
            + lo_vec[6] * DoryScalar::from(200)
            + lo_vec[7];
        let hi_fold_expected = hi_vec[0] * DoryScalar::from(300 * 301 * 302)
            + hi_vec[1] * DoryScalar::from(301 * 302)
            + hi_vec[2] * DoryScalar::from(300 * 302)
            + hi_vec[3] * DoryScalar::from(302)
            + hi_vec[4] * DoryScalar::from(300 * 301)
            + hi_vec[5] * DoryScalar::from(301)
            + hi_vec[6] * DoryScalar::from(300)
            + hi_vec[7];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_naive_fold_length_0_fold_factors() {
        let fold_factors = vec![];
        let mut vec = vec![DoryScalar::from(100)];
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(vec[0], DoryScalar::from(100));
    }
    #[test]
    fn we_can_naive_fold_length_1_fold_factors() {
        let fold_factors = vec![DoryScalar::from(2)];
        let mut vec = vec![DoryScalar::from(100), DoryScalar::from(101)];
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(vec[0], DoryScalar::from(100 * 2 + 101));
    }
    #[test]
    fn we_can_naive_fold_length_2_fold_factors() {
        let fold_factors = vec![DoryScalar::from(2), DoryScalar::from(3)];
        let mut vec = vec![
            DoryScalar::from(100),
            DoryScalar::from(101),
            DoryScalar::from(102),
            DoryScalar::from(103),
        ];
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(
            vec[0],
            DoryScalar::from(100 * 2 * 3 + 101 * 3 + 102 * 2 + 103)
        );
    }
    #[test]
    fn we_can_naive_fold_length_3_fold_factors() {
        let fold_factors = vec![
            DoryScalar::from(2),
            DoryScalar::from(3),
            DoryScalar::from(5),
        ];
        let mut vec = [100, 101, 102, 103, 104, 105, 106, 107].map(DoryScalar::from);
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(
            vec[0],
            DoryScalar::from(
                100 * 2 * 3 * 5
                    + 101 * 3 * 5
                    + 102 * 2 * 5
                    + 103 * 5
                    + 104 * 2 * 3
                    + 105 * 3
                    + 106 * 2
                    + 107
            )
        );
    }

    #[test]
    fn we_can_fold_dynamic_random_standard_basis_tensors() {
        use ark_std::{test_rng, UniformRand};
        use itertools::Itertools;
        let mut rng = test_rng();
        for num_vars in 0..10 {
            let point = core::iter::repeat_with(|| MontScalar(F::rand(&mut rng)))
                .take(num_vars)
                .collect_vec();
            let nu = point.len() / 2 + 1;
            let mut lo_vec = vec![DoryScalar::ZERO; 1 << nu];
            let mut hi_vec = vec![DoryScalar::ZERO; 1 << nu];
            lo_vec[0] = DoryScalar::ONE;
            hi_vec[0] = DoryScalar::ONE;
            compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

            let alphas = core::iter::repeat_with(|| MontScalar(F::rand(&mut rng)))
                .take(nu)
                .collect_vec();
            let alpha_invs = core::iter::repeat_with(|| MontScalar(F::rand(&mut rng)))
                .take(nu)
                .collect_vec();
            let (lo_fold, hi_fold) =
                fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
            naive_fold(&mut lo_vec, &alphas);
            naive_fold(&mut hi_vec, &alpha_invs);
            assert_eq!(lo_fold, lo_vec[0]);
            assert_eq!(hi_fold, hi_vec[0]);
        }
    }

    #[test]
    fn we_can_compute_dynamic_vecs_for_length_0_point() {
        let point = vec![];
        let expected_lo_vec = vec![DoryScalar::from(1), DoryScalar::from(0)];
        let expected_hi_vec = vec![DoryScalar::from(1), DoryScalar::from(1)];
        let (lo_vec, hi_vec) = compute_dynamic_vecs(&point);
        assert_eq!(expected_lo_vec, lo_vec);
        assert_eq!(expected_hi_vec, hi_vec);
    }

    #[test]
    fn we_can_compute_dynamic_vecs_for_length_1_point() {
        let point = vec![DoryScalar::from(2)];
        let expected_lo_vec = vec![DoryScalar::from(1 - 2), DoryScalar::from(2)];
        let expected_hi_vec = vec![DoryScalar::from(1), DoryScalar::from(1)];
        let (lo_vec, hi_vec) = compute_dynamic_vecs(&point);
        assert_eq!(expected_lo_vec, lo_vec);
        assert_eq!(expected_hi_vec, hi_vec);
    }

    #[test]
    fn we_can_compute_dynamic_vecs_for_length_2_point() {
        let point = vec![DoryScalar::from(2), DoryScalar::from(3)];
        let expected_lo_vec = vec![
            DoryScalar::from((1 - 2) * (1 - 3)),
            DoryScalar::from(2 * (1 - 3)),
            DoryScalar::from((1 - 2) * 3),
            DoryScalar::from(2 * 3),
        ];
        let expected_hi_vec = vec![
            DoryScalar::from(1),
            DoryScalar::from(1),
            MontScalar(F::from(3) / F::from(1 - 3)),
            DoryScalar::from(0),
        ];
        let (lo_vec, hi_vec) = compute_dynamic_vecs(&point);
        assert_eq!(expected_lo_vec, lo_vec);
        assert_eq!(expected_hi_vec, hi_vec);
    }

    #[test]
    fn we_can_compute_dynamic_vecs_for_length_3_point() {
        let point = vec![
            DoryScalar::from(2),
            DoryScalar::from(3),
            DoryScalar::from(5),
        ];
        let expected_lo_vec = vec![
            DoryScalar::from((1 - 2) * (1 - 3)),
            DoryScalar::from(2 * (1 - 3)),
            DoryScalar::from((1 - 2) * 3),
            DoryScalar::from(2 * 3),
        ];
        let expected_hi_vec = vec![
            DoryScalar::from(1 - 5),
            DoryScalar::from(1 - 5),
            MontScalar(F::from((1 - 5) * 3) / F::from(1 - 3)),
            DoryScalar::from(5),
        ];
        let (lo_vec, hi_vec) = compute_dynamic_vecs(&point);
        assert_eq!(expected_lo_vec, lo_vec);
        assert_eq!(expected_hi_vec, hi_vec);
    }

    #[test]
    fn we_can_compute_dynamic_vecs_that_matches_evaluation_vec() {
        use ark_std::UniformRand;
        let mut rng = ark_std::test_rng();
        for num_vars in 0..20 {
            let point: Vec<_> = core::iter::repeat_with(|| MontScalar(F::rand(&mut rng)))
                .take(num_vars)
                .collect();
            let (lo_vec, hi_vec) = compute_dynamic_vecs(&point);
            let mut eval_vec = vec![DoryScalar::ZERO; 1 << num_vars];
            compute_evaluation_vector(&mut eval_vec, &point);
            for (i, val) in eval_vec.into_iter().enumerate() {
                let (row, column) = row_and_column_from_index(i);
                assert_eq!(hi_vec[row] * lo_vec[column], val);
            }
        }
    }
}
