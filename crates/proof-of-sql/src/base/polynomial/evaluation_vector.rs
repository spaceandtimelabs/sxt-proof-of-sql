use core::ops::{Mul, MulAssign, Sub, SubAssign};
use num_traits::One;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

const MIN_PARALLEL_LEN: usize = 16; // The minimum size for which we should actually parallelize the compute.

/// This method manipulates left and right such that
/// right[i] = left[i] * p and left[i] = left[i] * (1 - p)
fn compute_evaluation_vector_impl<F>(left: &mut [F], right: &mut [F], p: F)
where
    F: One + Sub<Output = F> + MulAssign + SubAssign + Mul<Output = F> + Send + Sync + Copy,
{
    let k = std::cmp::min(left.len(), right.len());
    let one_minus_p = F::one() - p;
    left.par_iter_mut()
        .with_min_len(MIN_PARALLEL_LEN)
        .zip(right.par_iter_mut())
        .for_each(|(li, ri)| {
            *ri = *li * p;
            *li -= *ri;
        });
    left[k..]
        .par_iter_mut()
        .with_min_len(MIN_PARALLEL_LEN)
        .for_each(|li| {
            *li *= one_minus_p;
        });
}

/// Given a point of evaluation, computes the vector that allows us
/// to evaluate a multilinear extension as an inner product.
#[tracing::instrument(level = "debug", skip_all)]
pub fn compute_evaluation_vector<F>(v: &mut [F], point: &[F])
where
    F: One + Sub<Output = F> + MulAssign + SubAssign + Mul<Output = F> + Send + Sync + Copy,
{
    assert!(v.len() <= (1 << point.len()));
    if point.is_empty() || v.is_empty() {
        // v is guaranteed to be at most length 1 by the assert!.
        v.fill(F::one());
        return;
    }
    v[0] = F::one() - point[0];
    if v.len() > 1 {
        v[1] = point[0];
    }
    for (level, p) in point[1..].iter().enumerate() {
        let mid = 1 << (level + 1);
        let (left, right): (&mut [F], &mut [F]) = if mid >= v.len() {
            (v, &mut [])
        } else {
            v.split_at_mut(mid)
        };
        compute_evaluation_vector_impl(left, right, *p);
    }
}
