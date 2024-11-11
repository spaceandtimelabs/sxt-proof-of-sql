//! Utility functions for creating [`Table`]s and [`Column`]s.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use bumpalo::Bump;
//! use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
//! let alloc = Bump::new();
//! let result = table::<Curve25519Scalar>([
//!     bigint("a", [1, 2, 3], &alloc),
//!     boolean("b", [true, false, true], &alloc),
//!     int128("c", [1, 2, 3], &alloc),
//!     scalar("d", [1, 2, 3], &alloc),
//!     varchar("e", ["a", "b", "c"], &alloc),
//!     decimal75("f", 12, 1, [1, 2, 3], &alloc),
//! ]);
//! ```
use super::{Column, Table};
use crate::base::scalar::Scalar;
use alloc::{string::String, vec::Vec};
use bumpalo::Bump;
use core::ops::Deref;
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    Identifier,
};

/// Creates an [`Table`] from a list of `(Identifier, Column)` pairs.
/// This is a convenience wrapper around [`Table::try_from_iter`] primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [bigint], [boolean], etc).
/// The function will panic under a variety of conditions. See [`Table::try_from_iter`] for more details.
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3], &alloc),
///     boolean("b", [true, false, true], &alloc),
///     int128("c", [1, 2, 3], &alloc),
///     scalar("d", [1, 2, 3], &alloc),
///     varchar("e", ["a", "b", "c"], &alloc),
///     decimal75("f", 12, 1, [1, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if converting the iterator into an `Table<'a, S>` fails.
pub fn table<'a, S: Scalar>(
    iter: impl IntoIterator<Item = (Identifier, Column<'a, S>)>,
) -> Table<'a, S> {
    Table::try_from_iter(iter).unwrap()
}

/// Creates a (Identifier, `Column`) pair for a tinyint column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     tinyint("a", [1_i8, 2, 3], &alloc),
/// ]);
///```
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn tinyint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i8>>,
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<i8> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.parse().unwrap(), Column::TinyInt(alloc_data))
}

/// Creates a `(Identifier, Column)` pair for a smallint column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     smallint("a", [1_i16, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn smallint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i16>>,
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<i16> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.parse().unwrap(), Column::SmallInt(alloc_data))
}

/// Creates a `(Identifier, Column)` pair for an int column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     int("a", [1, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn int<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i32>>,
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<i32> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.parse().unwrap(), Column::Int(alloc_data))
}

/// Creates a `(Identifier, Column)` pair for a bigint column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
#[allow(clippy::missing_panics_doc)]
pub fn bigint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i64>>,
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<i64> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.parse().unwrap(), Column::BigInt(alloc_data))
}

/// Creates a `(Identifier, Column)` pair for a boolean column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     boolean("a", [true, false, true], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn boolean<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<bool>>,
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<bool> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.parse().unwrap(), Column::Boolean(alloc_data))
}

/// Creates a `(Identifier, Column)` pair for an int128 column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     int128("a", [1, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn int128<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i128>>,
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<i128> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.parse().unwrap(), Column::Int128(alloc_data))
}

/// Creates a `(Identifier, Column)` pair for a scalar column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     scalar("a", [1, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn scalar<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<S>>,
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<S> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.parse().unwrap(), Column::Scalar(alloc_data))
}

/// Creates a `(Identifier, Column)` pair for a varchar column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     varchar("a", ["a", "b", "c"], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if `name.parse()` fails to convert the name into an `Identifier`.
pub fn varchar<'a, S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<String>>,
    alloc: &'a Bump,
) -> (Identifier, Column<'a, S>) {
    let strings: Vec<&'a str> = data
        .into_iter()
        .map(|item| {
            let string = item.into();
            alloc.alloc_str(&string) as &'a str
        })
        .collect();
    let alloc_strings = alloc.alloc_slice_clone(&strings);
    let scalars: Vec<S> = strings.into_iter().map(Into::into).collect();
    let alloc_scalars = alloc.alloc_slice_copy(&scalars);
    (
        name.parse().unwrap(),
        Column::VarChar((alloc_strings, alloc_scalars)),
    )
}

/// Creates a `(Identifier, Column)` pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     decimal75("a", 12, 1, [1, 2, 3], &alloc),
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
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let transformed_data: Vec<S> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (
        name.parse().unwrap(),
        Column::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            alloc_data,
        ),
    )
}

/// Creates a `(Identifier, Column)` pair for a timestamp column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Parameters
/// - `name`: The name of the column.
/// - `time_unit`: The time unit of the timestamps.
/// - `timezone`: The timezone for the timestamps.
/// - `data`: The data for the column, provided as an iterator over `i64` values representing time since the unix epoch.
/// - `alloc`: The bump allocator to use for allocating the column data.
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*,
///     scalar::Curve25519Scalar,
/// };
/// use proof_of_sql_parser::{
///    posql_time::{PoSQLTimeZone, PoSQLTimeUnit}};
///
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     timestamptz("event_time", PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, vec![1625072400, 1625076000, 1625079600], &alloc),
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
    alloc: &Bump,
) -> (Identifier, Column<'_, S>) {
    let vec_data: Vec<i64> = data.into_iter().collect();
    let alloc_data = alloc.alloc_slice_copy(&vec_data);
    (
        name.parse().unwrap(),
        Column::TimestampTZ(time_unit, timezone, alloc_data),
    )
}
