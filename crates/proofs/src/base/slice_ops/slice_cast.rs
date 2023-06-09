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

/// This operation takes an `IndexedParallelIterator` and casts it to an `IndexedParallelIterator` of a different type using the provided function.
pub fn iter_cast_to_iter<F: Sync + Into<T>, T: Send>(
    value: impl IndexedParallelIterator<Item = F>,
) -> impl IndexedParallelIterator<Item = T> {
    value.with_min_len(super::MIN_RAYON_LEN).map(Into::into)
}
/// This operation takes an `IndexedParallelIterator` and casts it to a vector of a different type using the provided function.
pub fn iter_cast<F: Sync + Into<T>, T: Send>(
    value: impl IndexedParallelIterator<Item = F>,
) -> Vec<T> {
    iter_cast_to_iter(value).collect()
}
/// This operation takes a slice and casts it to an `IndexedParallelIterator` of a different type using the provided function.
pub fn slice_cast_to_iter<'a, F: Sync, T: Send + 'a>(
    value: &'a [F],
) -> impl IndexedParallelIterator<Item = T> + 'a
where
    &'a F: Into<T>,
{
    iter_cast_to_iter(value.par_iter())
}
/// This operation takes a slice and casts it to a vector of a different type using the provided function.
pub fn slice_cast<'a, F: Sync, T: Send>(value: &'a [F]) -> Vec<T>
where
    &'a F: Into<T>,
{
    iter_cast(value.par_iter())
}

/// This operation takes a slice and casts it to a mutable slice of a different type using the provided function.
pub fn slice_cast_mut<'a, F, T>(value: &'a [F], result: &mut [T])
where
    F: Sync,
    T: Send + Sync,
    &'a F: Into<T>,
{
    value
        .par_iter()
        .with_min_len(super::MIN_RAYON_LEN)
        .zip(result.par_iter_mut())
        .for_each(|(a, b)| *b = a.into());
}
