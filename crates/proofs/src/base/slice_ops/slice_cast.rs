use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

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
