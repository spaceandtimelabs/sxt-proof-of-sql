//! This module duplicates a lot of code that is in dynamic dory. Ideally this all will be cenralized.

use crate::base::scalar::Scalar;
use alloc::{vec, vec::Vec};
use itertools::Itertools;

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
    let standard_basis_point = point
        .iter()
        .map(|b| (S::ONE - *b).inv().expect("Values in point cannot be 1.") - S::ONE)
        .collect_vec();
    compute_dynamic_standard_basis_vecs(&standard_basis_point, &mut lo_vec, &mut hi_vec);
    (lo_vec, hi_vec)
}

/// Returns the (row, column) in the matrix where the data with the given index belongs.
pub(crate) const fn row_and_column_from_index(index: usize) -> (usize, usize) {
    let width_of_row = 1 << (((2 * index + 1).ilog2() + 1) / 2);
    let row = index / width_of_row + width_of_row / 2;
    let column = index % width_of_row;
    (row, column)
}

pub(crate) const fn full_width_of_row(row: usize) -> usize {
    ((2 * row + 4) / 3).next_power_of_two()
}

pub(crate) const fn matrix_size(data_len: usize, offset: usize) -> (usize, usize) {
    if data_len == 0 && offset == 0 {
        return (0, 0);
    }

    let (last_row, _) = row_and_column_from_index(offset + data_len - 1);
    let width_of_last_row = full_width_of_row(last_row);
    (last_row + 1, width_of_last_row)
}

#[allow(dead_code)]
/// This method produces evaluation vectors from a point. This is a helper method for generating a Vector-Matrix-Vector product in the dynamic dory scheme.
///
/// The ith element of the `lo_vec` is essentially the ith monomial basis element (lexicographically).
/// The ith element of the `hi_vec` is essentially the jth monomial basis element where `j = row_start_index(i)`.
///
/// NOTE: the `lo_vec` and `hi_vec` are scaled by `lo_vec[0]` and `hi_vec[0]` respectively.
/// NOTE: `lo_vec` and `hi_vec` should otherwise consist entirely of zeros in order to ensure correct output.
pub fn compute_dynamic_standard_basis_vecs<S: Scalar>(
    point: &[S],
    lo_vec: &mut [S],
    hi_vec: &mut [S],
) {
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

#[allow(dead_code)]
pub(super) fn fold_dynamic_standard_basis_tensors<S: Scalar>(
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
