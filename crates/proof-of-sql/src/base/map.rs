pub(crate) type IndexMap<K, V> =
    indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<ahash::AHasher>>;
pub(crate) type IndexSet<T> = indexmap::IndexSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

/// Create an [`IndexMap`][self::IndexMap] from a list of key-value pairs
macro_rules! indexmap {
    ($($key:expr => $value:expr,)+) => { indexmap::indexmap_with_default!{ahash::AHasher; $($key => $value),+} };
    ($($key:expr => $value:expr),*) => { indexmap::indexmap_with_default!{ahash::AHasher; $($key => $value),*} };
}

/// Create an [`IndexSet`][self::IndexSet] from a list of values
macro_rules! indexset {
    ($($value:expr,)+) => { indexmap::indexset_with_default!{ahash::AHasher; $($value),+} };
    ($($value:expr),*) => { indexmap::indexset_with_default!{ahash::AHasher; $($value),*} };
}

pub(crate) use indexmap;
pub(crate) use indexset;
