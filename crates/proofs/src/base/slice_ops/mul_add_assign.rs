use core::ops::{AddAssign, Mul};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

/// This operation does `result[i] += multiplier * to_mul_add[i]` for `i` in `0..to_mul_add.len()`.
///
/// It panics if `result.len() < to_mul_add.len()`.
pub fn mul_add_assign<T>(result: &mut [T], multiplier: T, to_mul_add: &[T])
where
    T: Send + Sync + Mul<Output = T> + AddAssign + Copy,
{
    assert!(result.len() >= to_mul_add.len());
    result
        .par_iter_mut()
        .with_min_len(super::MIN_RAYON_LEN)
        .zip(to_mul_add)
        .for_each(|(res_i, &data_i)| {
            *res_i += multiplier * data_i;
        })
}
