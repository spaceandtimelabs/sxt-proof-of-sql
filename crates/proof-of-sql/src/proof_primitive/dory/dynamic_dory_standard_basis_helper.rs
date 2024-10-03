//! This module provides the `build_standard_basis_vecs` method, which is used in converting a point to a
//! vector used in a Vector-Matrix-Vector product in the dynamic dory scheme.

use super::F;
use ark_ff::Field;

#[allow(dead_code)]
/// This method produces evaluation vectors from a point. This is a helper method for generating a Vector-Matrix-Vector product in the dynamic dory scheme.
///
/// The ith element of the `lo_vec` is essentially the ith monomial basis element (lexicographically).
/// The ith element of the `hi_vec` is essentially the jth monomial basis element where j = `row_start_index(i)`.
///
/// NOTE: the `lo_vec` and `hi_vec` are scaled by `lo_vec`[0] and `hi_vec`[0] respectively.
/// NOTE: `lo_vec` and `hi_vec` should otherwise consist entirely of zeros in order to ensure correct output.
pub(super) fn compute_dynamic_standard_basis_vecs(point: &[F], lo_vec: &mut [F], hi_vec: &mut [F]) {
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
        (o << p..(o + 1) << p).for_each(|k| hi_vec[k] *= v);
    });
}

fn build_partial_second_half_standard_basis_vecs(
    point: &[F],
    lo_vec: &mut [F],
    hi_vec: &mut [F],
    add_last_quarter: bool,
) {
    let nu = point.len() / 2 + 1;
    debug_assert_eq!(lo_vec.len(), 1 << nu);
    debug_assert_eq!(hi_vec.len(), 1 << nu);
    if nu == 1 {
        lo_vec[1] = if point.is_empty() {
            F::ZERO
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

#[allow(dead_code)]
pub(super) fn fold_dynamic_standard_basis_tensors(
    point: &[F],
    alphas: &[F],
    alpha_invs: &[F],
) -> (F, F) {
    let nu = point.len() / 2 + 1;
    debug_assert_eq!(alphas.len(), nu);
    debug_assert_eq!(alpha_invs.len(), nu);
    let lo_fold = if point.is_empty() {
        alphas[0]
    } else {
        point.iter().zip(alphas).map(|(v, a)| v + a).product()
    };
    let hi_fold = point
        .iter()
        .enumerate()
        .fold(
            (alpha_invs[0] + F::ONE, F::ZERO),
            |(acc, prev_partial), (i, &p)| {
                if i == 0 {
                    (acc, F::ZERO)
                } else if i == 1 {
                    (acc * alpha_invs[1] + p * alpha_invs[0], F::ZERO)
                } else if i % 2 == 0 {
                    let partial = (i / 2 + 1..i)
                        .zip(alpha_invs)
                        .map(|(k, a)| point[k] + a)
                        .product();
                    (acc + p * partial, partial)
                } else {
                    (
                        acc * alpha_invs[i / 2 + 1]
                            + p * alpha_invs[i / 2]
                                * (point[i - 1] + alpha_invs[i / 2 - 1])
                                * prev_partial,
                        F::ZERO,
                    )
                }
            },
        )
        .0;
    (lo_fold, hi_fold)
}

#[cfg(test)]
pub(super) mod tests {
    use super::{super::dynamic_dory_structure::row_start_index, *};

    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_0_point() {
        let mut lo_vec = vec![F::ZERO; 2];
        let mut hi_vec = vec![F::ZERO; 2];
        lo_vec[0] = F::from(2);
        hi_vec[0] = F::from(3);
        let point = vec![];
        let lo_vec_expected = vec![F::from(2), F::ZERO];
        let hi_vec_expected = vec![F::from(3), F::from(3)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_1_point() {
        let mut lo_vec = vec![F::ZERO; 2];
        let mut hi_vec = vec![F::ZERO; 2];
        lo_vec[0] = F::from(2);
        hi_vec[0] = F::from(3);
        let point = vec![F::from(5)];
        let lo_vec_expected = vec![F::from(2), F::from(2 * 5)];
        let hi_vec_expected = vec![F::from(3), F::from(3)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_2_point() {
        let mut lo_vec = vec![F::ZERO; 4];
        let mut hi_vec = vec![F::ZERO; 4];
        lo_vec[0] = F::from(2);
        hi_vec[0] = F::from(3);
        let point = vec![F::from(5), F::from(7)];
        let lo_vec_expected = vec![
            F::from(2),
            F::from(2 * 5),
            F::from(2 * 7),
            F::from(2 * 5 * 7),
        ];
        let hi_vec_expected = vec![F::from(3), F::from(3), F::from(3 * 7), F::ZERO];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_3_point() {
        let mut lo_vec = vec![F::ZERO; 4];
        let mut hi_vec = vec![F::ZERO; 4];
        lo_vec[0] = F::from(2);
        hi_vec[0] = F::from(3);
        let point = vec![F::from(5), F::from(7), F::from(11)];
        let lo_vec_expected = vec![
            F::from(2),
            F::from(2 * 5),
            F::from(2 * 7),
            F::from(2 * 5 * 7),
        ];
        let hi_vec_expected = vec![F::from(3), F::from(3), F::from(3 * 7), F::from(3 * 11)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_4_point() {
        let mut lo_vec = vec![F::ZERO; 8];
        let mut hi_vec = vec![F::ZERO; 8];
        lo_vec[0] = F::from(2);
        hi_vec[0] = F::from(3);
        let point = vec![F::from(5), F::from(7), F::from(11), F::from(13)];
        let lo_vec_expected = vec![
            F::from(2),
            F::from(2 * 5),
            F::from(2 * 7),
            F::from(2 * 5 * 7),
            F::from(2 * 11),
            F::from(2 * 5 * 11),
            F::from(2 * 7 * 11),
            F::from(2 * 5 * 7 * 11),
        ];
        let hi_vec_expected = vec![
            F::from(3),
            F::from(3),
            F::from(3 * 7),
            F::from(3 * 11),
            F::from(3 * 13),
            F::from(3 * 11 * 13),
            F::ZERO,
            F::ZERO,
        ];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
        assert_eq!(lo_vec, lo_vec_expected);
        assert_eq!(hi_vec, hi_vec_expected);
    }
    #[test]
    fn we_can_compute_dynamic_standard_basis_vecs_from_length_5_point() {
        let mut lo_vec = vec![F::ZERO; 8];
        let mut hi_vec = vec![F::ZERO; 8];
        lo_vec[0] = F::from(2);
        hi_vec[0] = F::from(3);
        let point = vec![
            F::from(5),
            F::from(7),
            F::from(11),
            F::from(13),
            F::from(17),
        ];
        let lo_vec_expected = vec![
            F::from(2),
            F::from(2 * 5),
            F::from(2 * 7),
            F::from(2 * 5 * 7),
            F::from(2 * 11),
            F::from(2 * 5 * 11),
            F::from(2 * 7 * 11),
            F::from(2 * 5 * 7 * 11),
        ];
        let hi_vec_expected = vec![
            F::from(3),
            F::from(3),
            F::from(3 * 7),
            F::from(3 * 11),
            F::from(3 * 13),
            F::from(3 * 11 * 13),
            F::from(3 * 17),
            F::from(3 * 13 * 17),
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
        .filter_map(|(i, b)| match b % 2 == 0 {
            true => None,
            false => Some(point.get(i).copied().unwrap_or(F::ZERO)),
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
            let alpha = F::rand(&mut rng);
            let beta = F::rand(&mut rng);
            let nu = point.len() / 2 + 1;
            let mut lo_vec = vec![F::ZERO; 1 << nu];
            let mut hi_vec = vec![F::ZERO; 1 << nu];
            lo_vec[0] = alpha;
            hi_vec[0] = beta;
            compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);
            for i in 0..1 << nu {
                assert_eq!(lo_vec[i], alpha * get_binary_eval(i, &point));
                assert_eq!(
                    hi_vec[i],
                    beta * get_binary_eval(row_start_index(i), &point)
                );
            }
        }
    }

    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_0_point() {
        let mut lo_vec = vec![F::ZERO; 2];
        let mut hi_vec = vec![F::ZERO; 2];
        lo_vec[0] = F::ONE;
        hi_vec[0] = F::ONE;
        let point = vec![];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![F::from(200)];
        let alpha_invs = vec![F::from(300)];
        let lo_fold_expected = lo_vec[0] * F::from(200) + lo_vec[1];
        let hi_fold_expected = hi_vec[0] * F::from(300) + hi_vec[1];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_1_point() {
        let mut lo_vec = vec![F::ZERO; 2];
        let mut hi_vec = vec![F::ZERO; 2];
        lo_vec[0] = F::ONE;
        hi_vec[0] = F::ONE;
        let point = vec![F::from(5)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![F::from(200)];
        let alpha_invs = vec![F::from(300)];
        let lo_fold_expected = lo_vec[0] * F::from(200) + lo_vec[1];
        let hi_fold_expected = hi_vec[0] * F::from(300) + hi_vec[1];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_2_point() {
        let mut lo_vec = vec![F::ZERO; 4];
        let mut hi_vec = vec![F::ZERO; 4];
        lo_vec[0] = F::ONE;
        hi_vec[0] = F::ONE;
        let point = vec![F::from(5), F::from(7)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![F::from(200), F::from(201)];
        let alpha_invs = vec![F::from(300), F::from(301)];
        let lo_fold_expected = lo_vec[0] * F::from(200 * 201)
            + lo_vec[1] * F::from(201)
            + lo_vec[2] * F::from(200)
            + lo_vec[3];
        let hi_fold_expected = hi_vec[0] * F::from(300 * 301)
            + hi_vec[1] * F::from(301)
            + hi_vec[2] * F::from(300)
            + hi_vec[3];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_3_point() {
        let mut lo_vec = vec![F::ZERO; 4];
        let mut hi_vec = vec![F::ZERO; 4];
        lo_vec[0] = F::ONE;
        hi_vec[0] = F::ONE;
        let point = vec![F::from(5), F::from(7), F::from(11)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![F::from(200), F::from(201)];
        let alpha_invs = vec![F::from(300), F::from(301)];
        let lo_fold_expected = lo_vec[0] * F::from(200 * 201)
            + lo_vec[1] * F::from(201)
            + lo_vec[2] * F::from(200)
            + lo_vec[3];
        let hi_fold_expected = hi_vec[0] * F::from(300 * 301)
            + hi_vec[1] * F::from(301)
            + hi_vec[2] * F::from(300)
            + hi_vec[3];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_4_point() {
        let mut lo_vec = vec![F::ZERO; 8];
        let mut hi_vec = vec![F::ZERO; 8];
        lo_vec[0] = F::ONE;
        hi_vec[0] = F::ONE;
        let point = vec![F::from(5), F::from(7), F::from(11), F::from(13)];
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![F::from(200), F::from(201), F::from(202)];
        let alpha_invs = vec![F::from(300), F::from(301), F::from(302)];
        let lo_fold_expected = lo_vec[0] * F::from(200 * 201 * 202)
            + lo_vec[1] * F::from(201 * 202)
            + lo_vec[2] * F::from(200 * 202)
            + lo_vec[3] * F::from(202)
            + lo_vec[4] * F::from(200 * 201)
            + lo_vec[5] * F::from(201)
            + lo_vec[6] * F::from(200)
            + lo_vec[7];
        let hi_fold_expected = hi_vec[0] * F::from(300 * 301 * 302)
            + hi_vec[1] * F::from(301 * 302)
            + hi_vec[2] * F::from(300 * 302)
            + hi_vec[3] * F::from(302)
            + hi_vec[4] * F::from(300 * 301)
            + hi_vec[5] * F::from(301)
            + hi_vec[6] * F::from(300)
            + hi_vec[7];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }
    #[test]
    fn we_can_fold_dynamic_standard_basis_tensors_of_length_5_point() {
        let mut lo_vec = vec![F::ZERO; 8];
        let mut hi_vec = vec![F::ZERO; 8];
        lo_vec[0] = F::ONE;
        hi_vec[0] = F::ONE;
        let point = [5, 7, 11, 13, 17].map(F::from);
        compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

        let alphas = vec![F::from(200), F::from(201), F::from(202)];
        let alpha_invs = vec![F::from(300), F::from(301), F::from(302)];
        let lo_fold_expected = lo_vec[0] * F::from(200 * 201 * 202)
            + lo_vec[1] * F::from(201 * 202)
            + lo_vec[2] * F::from(200 * 202)
            + lo_vec[3] * F::from(202)
            + lo_vec[4] * F::from(200 * 201)
            + lo_vec[5] * F::from(201)
            + lo_vec[6] * F::from(200)
            + lo_vec[7];
        let hi_fold_expected = hi_vec[0] * F::from(300 * 301 * 302)
            + hi_vec[1] * F::from(301 * 302)
            + hi_vec[2] * F::from(300 * 302)
            + hi_vec[3] * F::from(302)
            + hi_vec[4] * F::from(300 * 301)
            + hi_vec[5] * F::from(301)
            + hi_vec[6] * F::from(300)
            + hi_vec[7];
        let (lo_fold, hi_fold) = fold_dynamic_standard_basis_tensors(&point, &alphas, &alpha_invs);
        assert_eq!(lo_fold, lo_fold_expected);
        assert_eq!(hi_fold, hi_fold_expected);
    }

    pub fn naive_fold(mut vec: &mut [F], fold_factors: &[F]) {
        let nu = fold_factors.len();
        assert_eq!(vec.len(), 1 << fold_factors.len());
        for i in (0..nu).rev() {
            let (lo, hi) = vec.split_at_mut(vec.len() / 2);
            lo.iter_mut().zip(hi).for_each(|(l, h)| {
                *l *= fold_factors[i];
                *l += h;
            });
            vec = lo;
        }
    }
    #[test]
    fn we_can_naive_fold_length_0_fold_factors() {
        let fold_factors = vec![];
        let mut vec = vec![F::from(100)];
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(vec[0], F::from(100));
    }
    #[test]
    fn we_can_naive_fold_length_1_fold_factors() {
        let fold_factors = vec![F::from(2)];
        let mut vec = vec![F::from(100), F::from(101)];
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(vec[0], F::from(100 * 2 + 101));
    }
    #[test]
    fn we_can_naive_fold_length_2_fold_factors() {
        let fold_factors = vec![F::from(2), F::from(3)];
        let mut vec = vec![F::from(100), F::from(101), F::from(102), F::from(103)];
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(vec[0], F::from(100 * 2 * 3 + 101 * 3 + 102 * 2 + 103));
    }
    #[test]
    fn we_can_naive_fold_length_3_fold_factors() {
        let fold_factors = vec![F::from(2), F::from(3), F::from(5)];
        let mut vec = [100, 101, 102, 103, 104, 105, 106, 107].map(F::from);
        naive_fold(&mut vec, &fold_factors);
        assert_eq!(
            vec[0],
            F::from(
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
            let point = core::iter::repeat_with(|| F::rand(&mut rng))
                .take(num_vars)
                .collect_vec();
            let nu = point.len() / 2 + 1;
            let mut lo_vec = vec![F::ZERO; 1 << nu];
            let mut hi_vec = vec![F::ZERO; 1 << nu];
            lo_vec[0] = F::ONE;
            hi_vec[0] = F::ONE;
            compute_dynamic_standard_basis_vecs(&point, &mut lo_vec, &mut hi_vec);

            let alphas = core::iter::repeat_with(|| F::rand(&mut rng))
                .take(nu)
                .collect_vec();
            let alpha_invs = core::iter::repeat_with(|| F::rand(&mut rng))
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
}
