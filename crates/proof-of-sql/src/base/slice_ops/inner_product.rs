use core::{iter::Sum, ops::Mul};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// This operation takes the inner product of two slices. In other words, it does `a[0] * b[0] + a[1] * b[1] + ... + a[n] * b[n]`.
/// If one of the slices is longer than the other, the extra elements are ignored/considered to be 0.
pub fn inner_product<F>(a: &[F], b: &[F]) -> F
where
    F: Sync + Send + Mul<Output = F> + Sum + Copy,
{
    a.par_iter()
        .with_min_len(super::MIN_RAYON_LEN)
        .zip(b.par_iter())
        .map(|(&a, &b)| a * b)
        .sum()
}
