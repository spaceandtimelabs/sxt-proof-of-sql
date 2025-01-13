//! Utility functions for creating [`Table`]s and [`Column`]s.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use bumpalo::Bump;
//! use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
//! let alloc = Bump::new();
//! let result = table::<Curve25519Scalar>([
//!     borrowed_bigint("a", [1, 2, 3], &alloc),
//!     borrowed_boolean("b", [true, false, true], &alloc),
//!     borrowed_int128("c", [1, 2, 3], &alloc),
//!     borrowed_scalar("d", [1, 2, 3], &alloc),
//!     borrowed_varchar("e", ["a", "b", "c"], &alloc),
//!     borrowed_decimal75("f", 12, 1, [1, 2, 3], &alloc),
//! ]);
//! ```
use super::{Column, Table, TableOptions};
use crate::base::scalar::Scalar;
use alloc::{string::String, vec::Vec};
use bumpalo::Bump;
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
use sqlparser::ast::{Ident, TimezoneInfo};

/// Creates an [`Table`] from a list of `(Ident, Column)` pairs.
/// This is a convenience wrapper around [`Table::try_from_iter`] primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [`borrowed_bigint`],
/// [`borrowed_boolean`], etc).
/// The function will panic under a variety of conditions. See [`Table::try_from_iter`] for more details.
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_bigint("a", [1, 2, 3], &alloc),
///     borrowed_boolean("b", [true, false, true], &alloc),
///     borrowed_int128("c", [1, 2, 3], &alloc),
///     borrowed_scalar("d", [1, 2, 3], &alloc),
///     borrowed_varchar("e", ["a", "b", "c"], &alloc),
///     borrowed_decimal75("f", 12, 1, [1, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if converting the iterator into an `Table<'a, S>` fails.
pub fn table<'a, S: Scalar>(
    iter: impl IntoIterator<Item = (Ident, Column<'a, S>)>,
) -> Table<'a, S> {
    Table::try_from_iter(iter).unwrap()
}

/// Creates an [`Table`] from a list of `(Ident, Column)` pairs with a specified row count.
/// The main reason for this function is to allow for creating tables that may potentially have
/// no columns, but still have a specified row count.
///
/// # Panics
/// - Panics if the given row count doesn't match the number of rows in any of the columns.
pub fn table_with_row_count<'a, S: Scalar>(
    iter: impl IntoIterator<Item = (Ident, Column<'a, S>)>,
    row_count: usize,
) -> Table<'a, S> {
    Table::try_from_iter_with_options(iter, TableOptions::new(Some(row_count))).unwrap()
}

/// Creates a (Ident, `Column`) pair for a tinyint column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_tinyint("a", [1_i8, 2, 3], &alloc),
/// ]);
///```
pub fn borrowed_tinyint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i8>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i8> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::TinyInt(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a smallint column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_smallint("a", [1_i16, 2, 3], &alloc),
/// ]);
/// ```
///
pub fn borrowed_smallint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i16>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i16> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::SmallInt(alloc_data))
}

/// Creates a `(Ident, Column)` pair for an int column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_int("a", [1, 2, 3], &alloc),
/// ]);
/// ```
///
pub fn borrowed_int<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i32>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i32> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Int(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a bigint column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_bigint("a", [1, 2, 3], &alloc),
/// ]);
/// ```

pub fn borrowed_bigint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i64>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i64> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::BigInt(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a boolean column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_boolean("a", [true, false, true], &alloc),
/// ]);
/// ```

pub fn borrowed_boolean<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<bool>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<bool> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Boolean(alloc_data))
}

/// Creates a `(Ident, Column)` pair for an int128 column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_int128("a", [1, 2, 3], &alloc),
/// ]);
/// ```

pub fn borrowed_int128<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i128>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i128> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Int128(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a scalar column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_scalar("a", [1, 2, 3], &alloc),
/// ]);
/// ```

pub fn borrowed_scalar<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<S>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<S> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Scalar(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a varchar column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_varchar("a", ["a", "b", "c"], &alloc),
/// ]);
/// ```

pub fn borrowed_varchar<'a, S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<String>>,
    alloc: &'a Bump,
) -> (Ident, Column<'a, S>) {
    let strings: Vec<&'a str> = data
        .into_iter()
        .map(|item| {
            let string = item.into();
            alloc.alloc_str(&string) as &'a str
        })
        .collect();
    let alloc_strings = alloc.alloc_slice_clone(&strings);
    let scalars: Vec<S> = strings.iter().map(|s| (*s).into()).collect();
    let alloc_scalars = alloc.alloc_slice_copy(&scalars);
    (name.into(), Column::VarChar((alloc_strings, alloc_scalars)))
}

/// Creates a `(Ident, Column)` pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, scalar::Curve25519Scalar};
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_decimal75("a", 12, 1, [1, 2, 3], &alloc),
/// ]);
/// ```
/// # Panics
/// - Panics if creating the `Precision` from the specified precision value fails.
pub fn borrowed_decimal75<S: Scalar>(
    name: impl Into<Ident>,
    precision: u8,
    scale: i8,
    data: impl IntoIterator<Item = impl Into<S>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<S> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (
        name.into(),
        Column::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            alloc_data,
        ),
    )
}

/// Creates a `(Ident, Column)` pair for a timestamp column.
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
/// use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
/// use sqlparser::ast::TimezoneInfo;
/// let alloc = Bump::new();
/// let result = table::<Curve25519Scalar>([
///     borrowed_timestamptz("event_time", PoSQLTimeUnit::Second, TimezoneInfo::None,vec![1625072400, 1625076000, 1625079600], &alloc),
/// ]);
/// ```

pub fn borrowed_timestamptz<S: Scalar>(
    name: impl Into<Ident>,
    time_unit: PoSQLTimeUnit,
    timezone: TimezoneInfo,
    data: impl IntoIterator<Item = i64>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let vec_data: Vec<i64> = data.into_iter().collect();
    let alloc_data = alloc.alloc_slice_copy(&vec_data);
    (
        name.into(),
        Column::TimestampTZ(time_unit, timezone, alloc_data),
    )
}
