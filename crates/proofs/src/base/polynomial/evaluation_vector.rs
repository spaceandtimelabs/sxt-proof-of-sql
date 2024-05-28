use core::ops::{Mul, MulAssign, Sub};
use num_traits::One;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

const MIN_PARALLEL_LEN: usize = 16; // The minimum size for which we should actually parallelize the compute.

fn compute_evaluation_vector_impl<F>(left: &mut [F], right: &mut [F], p: F)
where
    F: One + Sub<Output = F> + MulAssign + Mul<Output = F> + Send + Sync + Copy,
{
    let k = std::cmp::min(left.len(), right.len());
    let pm1 = F::one() - p;
    left.par_iter_mut()
        .with_min_len(MIN_PARALLEL_LEN)
        .zip(right.par_iter_mut())
        .for_each(|(li, ri)| {
            *ri = *li * p;
            *li *= pm1;
        });
    left[k..]
        .par_iter_mut()
        .with_min_len(MIN_PARALLEL_LEN)
        .for_each(|li| {
            *li *= pm1;
        });
}

/// Given a point of evaluation, computes the vector that allows us
/// to evaluate a multilinear extension as an inner product.
#[tracing::instrument(level = "debug", skip_all)]
pub fn compute_evaluation_vector<F>(v: &mut [F], point: &[F])
where
    F: One + Sub<Output = F> + MulAssign + Mul<Output = F> + Send + Sync + Copy,
{
    let m = point.len();
    assert!(v.len() <= (1 << m));
    if m == 0 {
        // v is guarenteed to be at most length 1.
        v.fill(F::one());
        return;
    }
    assert!(v.len() > (1 << (m - 1)) || v.len() == 1);
    v[0] = F::one() - point[0];
    if v.len() == 1 {
        return;
    }
    v[1] = point[0];
    for (level, p) in point[1..].iter().enumerate() {
        let (left, right) = v.split_at_mut(1 << (level + 1));
        compute_evaluation_vector_impl(left, right, *p);
    }
}
