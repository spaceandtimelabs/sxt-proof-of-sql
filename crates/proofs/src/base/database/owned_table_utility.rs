//! Utility functions for creating OwnedTables and OwnedColumns.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
//! let result = owned_table::<Curve25519Scalar>([
//!     bigint("a", [1, 2, 3]),
//!     boolean("b", [true, false, true]),
//!     int128("c", [1, 2, 3]),
//!     scalar("d", [1, 2, 3]),
//!     varchar("e", ["a", "b", "c"]),
//!     decimal75("f", 12, 1, [1, 2, 3]),
//! ]);
//! ```
use super::{OwnedColumn, OwnedTable};
use crate::base::scalar::Scalar;
use core::ops::Deref;
use proofs_sql::Identifier;

/// Creates an OwnedTable from a list of (Identifier, OwnedColumn) pairs.
/// This is a convenience wrapper around OwnedTable::try_from_iter primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [bigint], [boolean], etc).
/// The function will panic under a variety of conditions. See [OwnedTable::try_from_iter] for more details.
///
/// # Example
/// ```
/// use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3]),
///     boolean("b", [true, false, true]),
///     int128("c", [1, 2, 3]),
///     scalar("d", [1, 2, 3]),
///     varchar("e", ["a", "b", "c"]),
///     decimal75("f", 12, 1, [1, 2, 3]),
/// ]);
/// ```
pub fn owned_table<S: Scalar>(
    iter: impl IntoIterator<Item = (Identifier, OwnedColumn<S>)>,
) -> OwnedTable<S> {
    OwnedTable::try_from_iter(iter).unwrap()
}

/// Creates a (Identifier, OwnedColumn) pair for a bigint column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3]),
/// ]);
pub fn bigint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i64>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::BigInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a boolean column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     boolean("a", [true, false, true]),
/// ]);
/// ```
pub fn boolean<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<bool>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Boolean(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a int128 column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     int128("a", [1, 2, 3]),
/// ]);
/// ```
pub fn int128<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i128>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Int128(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a scalar column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     scalar("a", [1, 2, 3]),
/// ]);
/// ```
pub fn scalar<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Scalar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a varchar column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     varchar("a", ["a", "b", "c"]),
/// ]);
/// ```
pub fn varchar<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<String>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::VarChar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proofs::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     decimal75("a", 12, 1, [1, 2, 3]),
/// ]);
/// ```
pub fn decimal75<S: Scalar>(
    name: impl Deref<Target = str>,
    precision: u8,
    scale: i8,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            data.into_iter().map(Into::into).collect(),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{database::OwnedTable, scalar::Curve25519Scalar};
    use core::str::FromStr;

    #[test]
    fn we_can_create_an_owned_table_that_gives_the_same_result_as_the_owned_table_macro() {
        let expected_result: OwnedTable<Curve25519Scalar> = crate::owned_table!(
            "a" => [1i64, 2, 3],
            "b" => [true, false, true],
            "c" => [1i128, 2, 3],
            "d" => ["a", "b", "c"],
        );
        let result = owned_table::<Curve25519Scalar>([
            bigint("a", [1, 2, 3]),
            boolean("b", [true, false, true]),
            int128("c", [1, 2, 3]),
            varchar("d", ["a", "b", "c"]),
        ]);
        assert_eq!(expected_result, result);
    }
    #[test]
    fn we_can_create_an_owned_table_with_comples_types_that_gives_the_same_result_as_the_owned_table_macro(
    ) {
        let id_b = Identifier::from_str("really_long_id_that_is_annoying").unwrap();
        let mut expected_result = crate::owned_table!(
            "a" => [1i64, 2, 3].map(Curve25519Scalar::from),
        );
        expected_result.append_decimal_columns_for_testing(
            id_b.as_str(),
            12,
            1,
            [1, 2, 3].map(Into::into).to_vec(),
        );

        let result = owned_table::<Curve25519Scalar>([
            scalar("a", [1, 2, 3]),
            decimal75(id_b, 12, 1, [1, 2, 3]),
        ]);
        assert_eq!(expected_result, result);
    }
}
