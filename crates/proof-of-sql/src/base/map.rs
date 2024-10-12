pub(crate) type IndexMap<K, V> =
    indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<ahash::AHasher>>;
pub(crate) type IndexSet<T> = indexmap::IndexSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

// Adapted from `indexmap`.

/// Create an [`IndexMap`][self::IndexMap] from a list of key-value pairs
macro_rules! indexmap_macro {
    ($($key:expr => $value:expr,)+) => { $crate::base::map::indexmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            // Note: `stringify!($key)` is just here to consume the repetition,
            // but we throw away that string literal during constant evaluation.
            const CAP: usize = <[()]>::len(&[$({ stringify!($key); }),*]);
            let mut map = $crate::base::map::IndexMap::with_capacity_and_hasher(CAP, <_>::default());
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

pub(crate) use indexmap_macro as indexmap;
