use crate::base::if_rayon;
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// This operation takes a slice and casts it to a vector of a different type using the provided function.
pub fn slice_cast_with<'a, F, T>(value: &'a [F], cast: impl Fn(&'a F) -> T + Send + Sync) -> Vec<T>
where
    F: Sync,
    T: Send,
{
    if_rayon!(
        value.par_iter().with_min_len(super::MIN_RAYON_LEN),
        value.iter()
    )
    .map(cast)
    .collect()
}

/// This operation takes a slice and casts it to a mutable slice of a different type using the provided function.
pub fn slice_cast_mut_with<'a, F, T>(
    value: &'a [F],
    result: &mut [T],
    cast: impl Fn(&'a F) -> T + Sync,
) where
    F: Sync,
    T: Send + Sync,
{
    if_rayon!(
        value.par_iter().with_min_len(super::MIN_RAYON_LEN),
        value.iter()
    )
    .zip(result)
    .for_each(|(a, b)| *b = cast(a));
}

/// This operation takes a slice and casts it to a vector of a different type using the provided function.
pub fn slice_cast<'a, F, T>(value: &'a [F]) -> Vec<T>
where
    F: Sync,
    T: Send,
    &'a F: Into<T>,
{
    slice_cast_with(value, Into::into)
}

/// This operation takes a slice and casts it to a mutable slice of a different type using the provided function.
pub fn slice_cast_mut<'a, F, T>(value: &'a [F], result: &mut [T])
where
    F: Sync,
    T: Send + Sync,
    &'a F: Into<T>,
{
    slice_cast_mut_with(value, result, Into::into);
}
