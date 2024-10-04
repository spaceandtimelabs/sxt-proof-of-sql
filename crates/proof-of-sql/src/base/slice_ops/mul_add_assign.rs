use crate::base::if_rayon;
use core::ops::{AddAssign, Mul};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

/// This operation does `result[i] += multiplier * to_mul_add[i]` for `i` in `0..to_mul_add.len()`.
///
/// # Panics
/// Panics if the length of `result` is less than the length of `to_mul_add`.
pub fn mul_add_assign<T, S>(result: &mut [T], multiplier: T, to_mul_add: &[S])
where
    T: Send + Sync + Mul<Output = T> + AddAssign + Copy,
    S: Into<T> + Sync + Copy,
{
    assert!(result.len() >= to_mul_add.len());
    if_rayon!(
        result.par_iter_mut().with_min_len(super::MIN_RAYON_LEN),
        result.iter_mut()
    )
    .zip(to_mul_add)
    .for_each(|(res_i, &data_i)| {
        *res_i += multiplier * data_i.into();
    })
}
