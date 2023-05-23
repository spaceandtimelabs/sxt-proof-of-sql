use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

/// This operation takes a slice and casts it to a vector of a different type using the provided function.
pub fn slice_cast_with<F, T>(value: &[F], cast: fn(&F) -> T) -> Vec<T>
where
    F: Sync,
    T: Send,
{
    value
        .par_iter()
        .with_min_len(super::MIN_RAYON_LEN)
        .map(cast)
        .collect()
}

/// This operation takes a slice and casts it to a mutable slice of a different type using the provided function.
pub fn slice_cast_mut_with<F, T>(value: &[F], result: &mut [T], cast: fn(&F) -> T)
where
    F: Sync,
    T: Send + Sync,
{
    value
        .par_iter()
        .with_min_len(super::MIN_RAYON_LEN)
        .zip(result.par_iter_mut())
        .for_each(|(a, b)| *b = cast(a));
}
