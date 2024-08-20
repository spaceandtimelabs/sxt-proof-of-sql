//! Utility functions for creating OwnedTables and OwnedColumns.
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
use ark_std::rand;
use core::ops::Deref;
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    Identifier,
};
use rand::Rng;

/// Creates an OwnedTable from a list of (Identifier, OwnedColumn) pairs.
/// This is a convenience wrapper around OwnedTable::try_from_iter primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [bigint], [boolean], etc).
/// The function will panic under a variety of conditions. See [OwnedTable::try_from_iter] for more details.
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
pub fn owned_table<S: Scalar>(
    iter: impl IntoIterator<Item = (Identifier, OwnedColumn<S>)>,
) -> OwnedTable<S> {
    OwnedTable::try_from_iter(iter).unwrap()
}

/// Creates a (Identifier, OwnedColumn) pair for a smallint column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     smallint("a", [1_i16, 2, 3]),
/// ]);
pub fn smallint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i16>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::SmallInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for an int column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     int("a", [1, 2, 3]),
/// ]);
pub fn int<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i32>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Int(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a bigint column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
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
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
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
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
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
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
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
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
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
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
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

/// Creates a (Identifier, OwnedColumn) pair for a timestamp column.
/// This is primarily intended for use in conjunction with [owned_table].
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
///     timestamptz("event_time", PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, vec![1625072400, 1625076000, 1625079600]),
/// ]);
/// ```
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

/// Generates a random OwnedTable with a specified number of columns
pub fn generate_random_owned_table<S: Scalar>(
    num_columns: usize,
    num_rows: usize,
) -> OwnedTable<S> {
    let mut rng = rand::thread_rng();
    let column_types = [
        "bigint",
        "boolean",
        "int128",
        "scalar",
        "varchar",
        "decimal75",
        "smallint",
        "int",
        "timestamptz",
    ];

    let mut columns = Vec::new();

    for _ in 0..num_columns {
        let column_type = column_types[rng.gen_range(0..column_types.len())];
        let identifier = format!("column_{}", rng.gen::<u32>());

        match column_type {
            "bigint" => columns.push(bigint(identifier.deref(), vec![rng.gen::<i64>(); num_rows])),
            "boolean" => columns.push(boolean(
                identifier.deref(),
                generate_random_boolean_vector(num_rows),
            )),
            "int128" => columns.push(int128(
                identifier.deref(),
                vec![rng.gen::<i128>(); num_rows],
            )),
            "scalar" => columns.push(scalar(
                identifier.deref(),
                vec![generate_random_u64_array(); num_rows],
            )),
            "varchar" => columns.push(varchar(identifier.deref(), gen_rnd_str(num_rows))),
            "decimal75" => columns.push(decimal75(
                identifier.deref(),
                12,
                2,
                vec![generate_random_u64_array(); num_rows],
            )),
            "smallint" => columns.push(smallint(
                identifier.deref(),
                vec![rng.gen::<i16>(); num_rows],
            )),
            "int" => columns.push(int(identifier.deref(), vec![rng.gen::<i32>(); num_rows])),
            "timestamptz" => columns.push(timestamptz(
                identifier.deref(),
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                vec![rng.gen::<i64>(); num_rows],
            )),
            _ => unreachable!(),
        }
    }

    owned_table(columns)
}

/// Generates a random vec of varchar
fn gen_rnd_str(array_size: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();

    // Create a vector to hold the owned Strings
    let mut string_vec: Vec<String> = Vec::with_capacity(array_size);

    for _ in 0..array_size {
        // Generate a random string of a fixed length (e.g., 10 characters)
        let random_string: String = (0..10)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect();

        string_vec.push(random_string);
    }

    string_vec
}

/// Generates a random [u64; 4]
pub fn generate_random_u64_array() -> [u64; 4] {
    let mut rng = rand::thread_rng();
    [rng.gen(), rng.gen(), rng.gen(), rng.gen()]
}

/// Generates a random vec of true/false
pub fn generate_random_boolean_vector(size: usize) -> Vec<bool> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}
