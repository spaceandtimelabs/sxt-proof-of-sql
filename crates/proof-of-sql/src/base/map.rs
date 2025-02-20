pub(crate) type IndexMap<K, V> =
    indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<ahash::AHasher>>;
pub(crate) type IndexSet<T> = indexmap::IndexSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

// Adapted from `indexmap`.

/// Create an [`IndexMap`][self::IndexMap] from a list of key-value pairs
macro_rules! indexmap_macro {
    ($($key:expr_2021 => $value:expr_2021,)+) => { $crate::base::map::indexmap!($($key => $value),+) };
    ($($key:expr_2021 => $value:expr_2021),*) => {
        {
            // Note: `stringify!($key)` is just here to consume the repetition,
            // but we throw away that string literal during constant evaluation.
            const CAP: usize = <[()]>::len(&[$({ stringify!($key); }),*]);
            #[allow(unused_mut)]
            let mut map = $crate::base::map::IndexMap::with_capacity_and_hasher(CAP, <_>::default());
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

/// Create an [`IndexSet`][self::IndexSet] from a list of values
macro_rules! indexset_macro {
    ($($value:expr_2021,)+) => { $crate::base::map::indexset!($($value),+) };
    ($($value:expr_2021),*) => {
        {
            const CAP: usize = <[()]>::len(&[$({ stringify!($value); }),*]);
            #[allow(unused_mut)]
            let mut set = $crate::base::map::IndexSet::with_capacity_and_hasher(CAP, <_>::default());
            $(
                set.insert($value);
            )*
            set
        }
    };
}

pub(crate) use indexmap_macro as indexmap;
pub(crate) use indexset_macro as indexset;
