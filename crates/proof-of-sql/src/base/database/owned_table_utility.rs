//! Utility functions for creating [`OwnedTable`]s and [`OwnedColumn`]s.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
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
use alloc::string::String;
use core::ops::Deref;
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    Identifier,
};

/// Creates an [`OwnedTable`] from a list of `(Identifier, OwnedColumn)` pairs.
/// This is a convenience wrapper around [`OwnedTable::try_from_iter`] primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [bigint], [boolean], etc).
/// The function will panic under a variety of conditions. See [`OwnedTable::try_from_iter`] for more details.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3]),
///     boolean("b", [true, false, true]),
///     int128("c", [1, 2, 3]),
///     scalar("d", [1, 2, 3]),
///     varchar("e", ["a", "b", "c"]),
///     decimal75("f", 12, 1, [1, 2, 3]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if converting the iterator into an `OwnedTable<S>` fails.
pub fn owned_table<S: Scalar>(
    iter: impl IntoIterator<Item = (Identifier, OwnedColumn<S>)>,
) -> OwnedTable<S> {
    OwnedTable::try_from_iter(iter).unwrap()
}

/// Creates a (Identifier, `OwnedColumn`) pair for a tinyint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     tinyint("a", [1_i8, 2, 3]),
/// ]);
///```
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn tinyint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i8>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::TinyInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for a smallint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     smallint("a", [1_i16, 2, 3]),
/// ]);
/// ```
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn smallint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i16>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::SmallInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for an int column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     int("a", [1, 2, 3]),
/// ]);
/// ```
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn int<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i32>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Int(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for a bigint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3]),
/// ]);
/// ```
#[allow(clippy::missing_panics_doc)]
pub fn bigint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i64>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::BigInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for a boolean column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     boolean("a", [true, false, true]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn boolean<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<bool>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Boolean(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for a int128 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     int128("a", [1, 2, 3]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn int128<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i128>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Int128(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for a scalar column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     scalar("a", [1, 2, 3]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn scalar<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Scalar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for a varchar column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     varchar("a", ["a", "b", "c"]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn varchar<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<String>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::VarChar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Identifier, OwnedColumn)` pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     decimal75("a", 12, 1, [1, 2, 3]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
/// - Panics if creating the `Precision` from the specified precision value fails.
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

/// Creates a `(Identifier, OwnedColumn)` pair for a timestamp column.
/// This is primarily intended for use in conjunction with [`owned_table`].
///
/// # Parameters
/// - `name`: The name of the column.
/// - `time_unit`: The time unit of the timestamps.
/// - `timezone`: The timezone for the timestamps.
/// - `data`: The data for the column, provided as an iterator over `i64` values representing time since the unix epoch.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*,
///     scalar::Curve25519Scalar,
/// };
/// use proof_of_sql_parser::{
///    posql_time::{PoSQLTimeZone, PoSQLTimeUnit}};
///
/// let result = owned_table::<Curve25519Scalar>([
///     timestamptz("event_time", PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![1625072400, 1625076000, 1625079600]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn timestamptz<S: Scalar>(
    name: impl Deref<Target = str>,
    time_unit: PoSQLTimeUnit,
    timezone: PoSQLTimeZone,
    data: impl IntoIterator<Item = i64>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::TimestampTZ(time_unit, timezone, data.into_iter().collect()),
    )
}
