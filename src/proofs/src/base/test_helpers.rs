use std::fmt::Debug;
use std::ops::RangeInclusive;

use curve25519_dalek::scalar::Scalar;
use proptest::prelude::*;

use crate::base::proof::Column;

/// Generate a nonzero scalar from a u128
pub fn arbitrary_scalar() -> BoxedStrategy<Scalar> {
    any::<u128>().prop_map(Scalar::from).boxed()
}

/// Generate a known-length column of scalars
pub fn arbitrary_column<X: Debug + 'static>(
    len: usize,
    of_what: BoxedStrategy<X>,
) -> BoxedStrategy<Column<X>> {
    proptest::collection::vec(of_what, len..=len)
        .prop_map(Column::from)
        .boxed()
}

/// Generate any number of columns with the same length, useful for testing
///
/// This returns arrays, so that you can use them in irrefutable patterns,
/// which are very convenient, e.g.:
/// ```ignore
/// fn hadamard_valid_path(
///     p in arb_column_arr(1..=10, arb_scalar())
/// ) {
///     // The length of p is inferred from how you destructure here
///     let [a, b, c] = p;
///     // Now do whatever you want with these columns
///     let ab = &a * &b;
/// }
/// ```
pub fn arbitrary_column_array<const L: usize, X: Debug + 'static>(
    lens: RangeInclusive<usize>,
    of_what: BoxedStrategy<X>,
) -> BoxedStrategy<[Column<X>; L]> {
    lens.prop_flat_map(move |l| {
        (0..L)
            .map(|_| arbitrary_column(l, of_what.clone()))
            .collect::<Vec<_>>()
            .prop_map(|v| v.try_into().unwrap())
    })
    .boxed()
}
