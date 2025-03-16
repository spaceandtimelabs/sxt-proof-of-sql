//! Utility functions for creating [`OwnedTable`]s and [`OwnedColumn`]s.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use proof_of_sql::base::{database::owned_table_utility::*};
//! # use proof_of_sql::base::scalar::MontScalar;
//! # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
//! let result = owned_table::<MyScalar>([
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
use alloc::{string::String, vec::Vec};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use sqlparser::ast::Ident;

// Thread-local storage to hold presence information until the OwnedTable is created
thread_local! {
    static NULLABLE_COLUMNS: std::cell::RefCell<std::collections::HashMap<(Ident, usize), Vec<bool>>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Creates an [`OwnedTable`] from a list of `(Ident, OwnedColumn)` pairs.
/// This is a convenience wrapper around [`OwnedTable::try_from_iter`] primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [bigint], [boolean], etc).
/// The function will panic under a variety of conditions. See [`OwnedTable::try_from_iter`] for more details.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///      bigint("a", [1, 2, 3]),
///      boolean("b", [true, false, true]),
///      int128("c", [1, 2, 3]),
///      scalar("d", [1, 2, 3]),
///      varchar("e", ["a", "b", "c"]),
///      decimal75("f", 12, 1, [1, 2, 3]),
/// ]);
/// ```
/// ///
/// # Panics
/// - Panics if converting the iterator into an `OwnedTable<S>` fails.
pub fn owned_table<S: Scalar>(
    iter: impl IntoIterator<Item = (Ident, OwnedColumn<S>)>,
) -> OwnedTable<S> {
    // First, collect all the columns
    let columns: Vec<_> = iter.into_iter().collect();

    // Create the table
    let mut table = OwnedTable::try_from_iter(columns).unwrap();

    // Get all the nullable columns from thread-local storage
    let nullable_columns = NULLABLE_COLUMNS.with(|cell| {
        let mut map = cell.borrow_mut();
        let result = map.clone();
        map.clear();
        result
    });

    // Now add presence information for nullable columns
    for ((name, len), presence) in nullable_columns {
        if let Some(col) = table.inner_table().get(&name) {
            if col.len() == len {
                table.set_presence(name, presence);
            }
        }
    }

    table
}

/// Creates a nullable column with the given name, values, and presence vector.
/// This is primarily intended for use in conjunction with [`owned_table`].
///
/// # Panics
///
/// Panics if the presence vector length does not match the values length.
pub fn nullable_column<S: Scalar>(
    name: impl Into<Ident>,
    values: &OwnedColumn<S>,
    presence: Option<Vec<bool>>,
) -> (Ident, OwnedColumn<S>) {
    let name_ident = name.into();
    let result = (name_ident.clone(), values.clone());

    // If we have presence information, we need to add it to the OwnedTable
    if let Some(presence_vec) = presence {
        NULLABLE_COLUMNS.with(|cell| {
            let mut map = cell.borrow_mut();
            map.insert((name_ident, values.len()), presence_vec);
        });
    }

    result
}

/// Creates a (`Ident`, `OwnedNullableColumn`) pair for a nullable column.
/// This is primarily intended for use with [`owned_table_with_nulls`].
///
/// # Arguments
/// * `name` - The name of the column
/// * `values` - The column values
/// * `presence` - The presence vector (true = value present, false = NULL)
///
/// # Returns
/// A tuple containing the column name and an `OwnedNullableColumn`
///
/// # Panics
/// Panics if the presence vector length does not match the values length.
pub fn nullable_column_pair<S: Scalar>(
    name: impl Into<Ident>,
    values: OwnedColumn<S>,
    presence: Option<Vec<bool>>,
) -> (Ident, super::owned_column::OwnedNullableColumn<S>) {
    let name = name.into();
    let nullable =
        super::owned_column::OwnedNullableColumn::with_presence(values, presence).unwrap();
    (name, nullable)
}

/// Creates an [`OwnedTable`] from a list of nullable column pairs.
/// This function properly preserves nullability information in the created table.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
///
/// // Create presence vectors (true = value present, false = NULL)
/// let presence_a = Some(vec![true, false, true]);
/// let presence_b = Some(vec![false, true, false]);
///
/// let result = owned_table_with_nulls::<MyScalar>([
///     nullable_column("a", bigint_values([1, 2, 3]), presence_a),
///     nullable_column("b", varchar_values(["x", "y", "z"]), presence_b),
/// ]);
/// ```
///
/// # Panics
/// - Panics if converting the iterator into an `OwnedTable<S>` fails.
pub fn owned_table_with_nulls<S: Scalar>(
    iter: impl IntoIterator<Item = (Ident, super::owned_column::OwnedNullableColumn<S>)>,
) -> OwnedTable<S> {
    let columns: crate::base::map::IndexMap<_, _> = iter.into_iter().collect();
    OwnedTable::try_new_from_nullable_columns(columns).unwrap()
}

/// Helper function to create bigint values without creating a column pair
/// Intended for use with `nullable_column` and `owned_table_with_nulls`
pub fn bigint_values<S: Scalar>(data: impl IntoIterator<Item = impl Into<i64>>) -> OwnedColumn<S> {
    OwnedColumn::BigInt(data.into_iter().map(Into::into).collect())
}

/// Helper function to create varchar values without creating a column pair
/// Intended for use with `nullable_column` and `owned_table_with_nulls`
pub fn varchar_values<S: Scalar>(
    data: impl IntoIterator<Item = impl Into<String>>,
) -> OwnedColumn<S> {
    OwnedColumn::VarChar(data.into_iter().map(Into::into).collect())
}

/// Helper function to create boolean values without creating a column pair
/// Intended for use with `nullable_column` and `owned_table_with_nulls`
pub fn boolean_values<S: Scalar>(
    data: impl IntoIterator<Item = impl Into<bool>>,
) -> OwnedColumn<S> {
    OwnedColumn::Boolean(data.into_iter().map(Into::into).collect())
}

/// Creates a (Ident, `OwnedColumn`) pair for a uint8 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     uint8("a", [1_u8, 2, 3]),
/// ]);
///```
pub fn uint8<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<u8>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Uint8(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Ident, `OwnedColumn`) pair for a tinyint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     tinyint("a", [1_i8, 2, 3]),
/// ]);
///```
pub fn tinyint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i8>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::TinyInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a smallint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     smallint("a", [1_i16, 2, 3]),
/// ]);
/// ```
pub fn smallint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i16>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::SmallInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for an int column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     int("a", [1, 2, 3]),
/// ]);
/// ```
pub fn int<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i32>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Int(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a bigint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     bigint("a", [1, 2, 3]),
/// ]);
/// ```
#[allow(clippy::missing_panics_doc)]
pub fn bigint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i64>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::BigInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a boolean column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     boolean("a", [true, false, true]),
/// ]);
/// ```
pub fn boolean<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<bool>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Boolean(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a int128 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     int128("a", [1, 2, 3]),
/// ]);
/// ```
pub fn int128<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i128>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Int128(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a scalar column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     scalar("a", [1, 2, 3]),
/// ]);
/// ```
pub fn scalar<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Scalar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a varchar column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     varchar("a", ["a", "b", "c"]),
/// ]);
/// ```
pub fn varchar<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<String>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::VarChar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a varbinary column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///    varbinary("a", [[1, 2, 3], [4, 5, 6], [7, 8, 9]]),
/// ]);
/// ```
pub fn varbinary<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<Vec<u8>>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::VarBinary(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     decimal75("a", 12, 1, [1, 2, 3]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if creating the `Precision` from the specified precision value fails.
pub fn decimal75<S: Scalar>(
    name: impl Into<Ident>,
    precision: u8,
    scale: i8,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            data.into_iter().map(Into::into).collect(),
        ),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a timestamp column.
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
/// use proof_of_sql::base::{database::owned_table_utility::*, };
/// use proof_of_sql_parser::{
///    posql_time::{PoSQLTimeZone, PoSQLTimeUnit}};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     timestamptz("event_time", PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![1625072400, 1625076000, 1625079600]),
/// ]);
/// ```
pub fn timestamptz<S: Scalar>(
    name: impl Into<Ident>,
    time_unit: PoSQLTimeUnit,
    timezone: PoSQLTimeZone,
    data: impl IntoIterator<Item = i64>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::TimestampTZ(time_unit, timezone, data.into_iter().collect()),
    )
}
