use super::{owned_column::OwnedNullableColumn, LiteralValue, TableRef};
use crate::base::{
    database::owned_column::OwnedColumn,
    database::table::TableError,
    math::decimal::Precision,
    scalar::{Scalar, ScalarExt},
    slice_ops::slice_cast_with,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::{
    fmt,
    fmt::{Display, Formatter},
    mem::size_of,
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum Column<'a, S: Scalar> {
    /// Boolean columns
    Boolean(&'a [bool]),
    /// u8 columns
    Uint8(&'a [u8]),
    /// i8 columns
    TinyInt(&'a [i8]),
    /// i16 columns
    SmallInt(&'a [i16]),
    /// i32 columns
    Int(&'a [i32]),
    /// i64 columns
    BigInt(&'a [i64]),
    /// i128 columns
    Int128(&'a [i128]),
    /// Decimal columns with a max width of 252 bits
    ///  - the backing store maps to the type `S`
    Decimal75(Precision, i8, &'a [S]),
    /// Scalar columns
    Scalar(&'a [S]),
    /// String columns
    ///  - the first element maps to the str values.
    ///  - the second element maps to the str hashes (see [`crate::base::scalar::Scalar`]).
    VarChar((&'a [&'a str], &'a [S])),
    /// Timestamp columns with timezone
    /// - the first element maps to the stored `TimeUnit`
    /// - the second element maps to a timezone
    /// - the third element maps to columns of timeunits since unix epoch
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone, &'a [i64]),
    /// Variable length binary columns
    VarBinary((&'a [&'a [u8]], &'a [S])),
}

/// Represents a nullable column that contains a values Column,
/// and an optional boolean presence slice.
///
/// When `presence` is `None`, the column is not nullable (all values are present).
/// When `presence` contains a boolean slice, its length must match the length of the values column,
/// and a `true` value indicates the presence of a value at the corresponding index,
/// while a `false` value indicates NULL.
///
/// This implementation follows the `PostgreSQL` approach to NULL values by using
/// a separate boolean array to track presence.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct NullableColumn<'a, S: Scalar> {
    /// The actual values in the column
    pub values: Column<'a, S>,
    /// Optional presence slice. `true` means value is present, `false` means NULL
    /// If `None`, all values are present (non-NULL)
    pub presence: Option<&'a [bool]>,
}

impl<'a, S: Scalar> NullableColumn<'a, S> {
    /// Creates a new `NullableColumn` without any NULL values
    /// (all values are present)
    #[must_use]
    pub fn new(values: Column<'a, S>) -> Self {
        Self {
            values,
            presence: None,
        }
    }

    /// Creates a new `NullableColumn` with the given values and presence slice
    ///
    /// Returns an error if the presence slice is `Some` and its length does not match the values length
    pub fn with_presence(
        values: Column<'a, S>,
        presence: Option<&'a [bool]>,
    ) -> Result<Self, TableError> {
        if let Some(presence_slice) = presence {
            // Use a more efficient length comparison that avoids potential performance issues with very large datasets
            // This check is O(1) regardless of the size of the slices
            let values_len = values.len();
            let presence_len = presence_slice.len();

            if values_len != presence_len {
                return Err(TableError::PresenceLengthMismatch);
            }
        }
        Ok(Self { values, presence })
    }

    /// Returns the length of the column
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the column has no elements
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns `true` if the column is nullable
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.presence.is_some()
    }

    /// Returns the column type
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.values.column_type()
    }

    /// Checks if the value at the given index is NULL
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    #[must_use]
    pub fn is_null(&self, index: usize) -> bool {
        // Perform a single length check to avoid multiple bounds checks in large datasets
        let column_len = self.len();
        assert!(index < column_len, "Index out of bounds");

        // Use a direct access pattern that's more efficient for large datasets
        match self.presence {
            Some(presence) => !presence[index],
            None => false, // When presence is None, no values are NULL
        }
    }

    /// Returns the scalar at the given index, or None if the value is NULL
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    #[must_use]
    pub fn scalar_at(&self, index: usize) -> Option<Option<S>> {
        // Perform a single length check to avoid multiple bounds checks in large datasets
        let column_len = self.len();
        assert!(index < column_len, "Index out of bounds");

        // Optimize the NULL check for large datasets by avoiding unnecessary operations
        // Check if the value is NULL first to avoid unnecessary scalar conversion
        if let Some(presence) = self.presence {
            if !presence[index] {
                return Some(None); // This position contains a NULL
            }
        }

        // Get the non-NULL value only if needed
        self.values.scalar_at(index).map(Some)
    }

    /// Create a `NullableColumn` from an `OwnedNullableColumn`
    pub fn from_owned_nullable_column(
        owned_column: &'a OwnedNullableColumn<S>,
        alloc: &'a Bump,
    ) -> Self {
        let values = Column::from_owned_column(&owned_column.values, alloc);

        // Create the presence slice with the correct lifetime and immutability
        let presence = if let Some(p) = &owned_column.presence {
            // First copy the data into the bump allocator
            let bool_vec = p.as_slice();
            // Then explicitly create it as an immutable slice with the correct lifetime
            let slice_ref: &'a [bool] = alloc.alloc_slice_copy(bool_vec);
            Some(slice_ref)
        } else {
            None
        };

        Self { values, presence }
    }
}

impl<'a, S: Scalar> Column<'a, S> {
    /// Provides the column type associated with the column
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Boolean(_) => ColumnType::Boolean,
            Self::Uint8(_) => ColumnType::Uint8,
            Self::TinyInt(_) => ColumnType::TinyInt,
            Self::SmallInt(_) => ColumnType::SmallInt,
            Self::Int(_) => ColumnType::Int,
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::Int128(_col) => ColumnType::Int128,
            Self::Scalar(_col) => ColumnType::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnType::Decimal75(*precision, *scale),
            Self::TimestampTZ(time_unit, timezone, _) => {
                ColumnType::TimestampTZ(*time_unit, *timezone)
            }
            Self::VarBinary(..) => ColumnType::VarBinary,
        }
    }
    /// Returns the length of the column.
    /// # Panics
    /// this function requires that `col` and `scals` have the same length.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Boolean(col) => col.len(),
            Self::Uint8(col) => col.len(),
            Self::TinyInt(col) => col.len(),
            Self::SmallInt(col) => col.len(),
            Self::Int(col) => col.len(),
            Self::BigInt(col) | Self::TimestampTZ(_, _, col) => col.len(),
            Self::VarChar((col, scals)) => {
                assert_eq!(col.len(), scals.len());
                col.len()
            }
            Self::VarBinary((col, scals)) => {
                assert_eq!(col.len(), scals.len());
                col.len()
            }
            Self::Int128(col) => col.len(),
            Self::Scalar(col) | Self::Decimal75(_, _, col) => col.len(),
        }
    }
    /// Returns `true` if the column has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Generate a constant column from a literal value with a given length
    pub fn from_literal_with_length(
        literal: &LiteralValue,
        length: usize,
        alloc: &'a Bump,
    ) -> Self {
        match literal {
            LiteralValue::Boolean(value) => {
                Column::Boolean(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Uint8(value) => {
                Column::Uint8(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::TinyInt(value) => {
                Column::TinyInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::SmallInt(value) => {
                Column::SmallInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Int(value) => Column::Int(alloc.alloc_slice_fill_copy(length, *value)),
            LiteralValue::BigInt(value) => {
                Column::BigInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Int128(value) => {
                Column::Int128(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Scalar(value) => {
                Column::Scalar(alloc.alloc_slice_fill_copy(length, (*value).into()))
            }
            LiteralValue::Decimal75(precision, scale, value) => Column::Decimal75(
                *precision,
                *scale,
                alloc.alloc_slice_fill_copy(length, value.into_scalar()),
            ),
            LiteralValue::TimeStampTZ(tu, tz, value) => {
                Column::TimestampTZ(*tu, *tz, alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::VarChar(string) => Column::VarChar((
                alloc.alloc_slice_fill_with(length, |_| alloc.alloc_str(string) as &str),
                alloc.alloc_slice_fill_copy(length, S::from(string)),
            )),
            LiteralValue::VarBinary(bytes) => {
                // Convert the bytes to a slice of bytes references
                let bytes_slice = alloc
                    .alloc_slice_fill_with(length, |_| alloc.alloc_slice_copy(bytes) as &[_]);

                // Convert the bytes to scalars using from_byte_slice_via_hash
                let scalars =
                    alloc.alloc_slice_fill_copy(length, S::from_byte_slice_via_hash(bytes));

                Column::VarBinary((bytes_slice, scalars))
            }
        }
    }

    /// Generate a `Int128` `rho` column [0, 1, 2, ..., length - 1]
    pub fn rho(length: usize, alloc: &'a Bump) -> Self {
        let raw_rho = (0..length as i128).collect::<Vec<_>>();
        let rho = alloc.alloc_slice_copy(raw_rho.as_slice());
        Column::<S>::Int128(rho as &[_])
    }

    /// Convert an `OwnedColumn` to a `Column`
    pub fn from_owned_column(owned_column: &'a OwnedColumn<S>, alloc: &'a Bump) -> Self {
        match owned_column {
            OwnedColumn::Boolean(col) => Column::Boolean(col.as_slice()),
            OwnedColumn::Uint8(col) => Column::Uint8(col.as_slice()),
            OwnedColumn::TinyInt(col) => Column::TinyInt(col.as_slice()),
            OwnedColumn::SmallInt(col) => Column::SmallInt(col.as_slice()),
            OwnedColumn::Int(col) => Column::Int(col.as_slice()),
            OwnedColumn::BigInt(col) => Column::BigInt(col.as_slice()),
            OwnedColumn::Int128(col) => Column::Int128(col.as_slice()),
            OwnedColumn::Decimal75(precision, scale, col) => {
                Column::Decimal75(*precision, *scale, col.as_slice())
            }
            OwnedColumn::Scalar(col) => Column::Scalar(col.as_slice()),
            OwnedColumn::VarChar(col) => {
                let scalars = col.iter().map(S::from).collect::<Vec<_>>();
                let strs = col
                    .iter()
                    .map(|s| s.as_str() as &'a str)
                    .collect::<Vec<_>>();
                Column::VarChar((
                    alloc.alloc_slice_clone(strs.as_slice()),
                    alloc.alloc_slice_copy(scalars.as_slice()),
                ))
            }
            OwnedColumn::VarBinary(col) => {
                let scalars = col
                    .iter()
                    .map(|b| S::from_byte_slice_via_hash(b))
                    .collect::<Vec<_>>();
                let bytes = col.iter().map(|s| s as &'a [u8]).collect::<Vec<_>>();
                Column::VarBinary((
                    alloc.alloc_slice_clone(&bytes),
                    alloc.alloc_slice_copy(scalars.as_slice()),
                ))
            }
            OwnedColumn::TimestampTZ(tu, tz, col) => Column::TimestampTZ(*tu, *tz, col.as_slice()),
        }
    }

    /// Returns the column as a slice of booleans if it is a boolean column. Otherwise, returns None.
    pub(crate) fn as_boolean(&self) -> Option<&'a [bool]> {
        match self {
            Self::Boolean(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of u8 if it is a uint8 column. Otherwise, returns None.
    pub(crate) fn as_uint8(&self) -> Option<&'a [u8]> {
        match self {
            Self::Uint8(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i8 if it is a tinyint column. Otherwise, returns None.
    pub(crate) fn as_tinyint(&self) -> Option<&'a [i8]> {
        match self {
            Self::TinyInt(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i16 if it is a smallint column. Otherwise, returns None.
    pub(crate) fn as_smallint(&self) -> Option<&'a [i16]> {
        match self {
            Self::SmallInt(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i32 if it is an int column. Otherwise, returns None.
    pub(crate) fn as_int(&self) -> Option<&'a [i32]> {
        match self {
            Self::Int(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i64 if it is a bigint column. Otherwise, returns None.
    pub(crate) fn as_bigint(&self) -> Option<&'a [i64]> {
        match self {
            Self::BigInt(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i128 if it is an int128 column. Otherwise, returns None.
    pub(crate) fn as_int128(&self) -> Option<&'a [i128]> {
        match self {
            Self::Int128(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of scalars if it is a scalar column. Otherwise, returns None.
    pub(crate) fn as_scalar(&self) -> Option<&'a [S]> {
        match self {
            Self::Scalar(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of scalars if it is a decimal75 column. Otherwise, returns None.
    pub(crate) fn as_decimal75(&self) -> Option<&'a [S]> {
        match self {
            Self::Decimal75(_, _, col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of strings and a slice of scalars if it is a varchar column. Otherwise, returns None.
    pub(crate) fn as_varchar(&self) -> Option<(&'a [&'a str], &'a [S])> {
        match self {
            Self::VarChar((col, scals)) => Some((col, scals)),
            _ => None,
        }
    }

    /// Returns the column as a slice of strings and a slice of scalars if it is a varchar column. Otherwise, returns None.
    pub(crate) fn as_varbinary(&self) -> Option<(&'a [&'a [u8]], &'a [S])> {
        match self {
            Self::VarBinary((col, scals)) => Some((col, scals)),
            _ => None,
        }
    }

    /// Returns the column as a slice of i64 if it is a timestamp column. Otherwise, returns None.
    pub(crate) fn as_timestamptz(&self) -> Option<&'a [i64]> {
        match self {
            Self::TimestampTZ(_, _, col) => Some(col),
            _ => None,
        }
    }

    /// Returns element at index as scalar
    ///
    /// Note that if index is out of bounds, this function will return None
    pub(crate) fn scalar_at(&self, index: usize) -> Option<S> {
        (index < self.len()).then_some(match self {
            Self::Boolean(col) => S::from(col[index]),
            Self::Uint8(col) => S::from(col[index]),
            Self::TinyInt(col) => S::from(col[index]),
            Self::SmallInt(col) => S::from(col[index]),
            Self::Int(col) => S::from(col[index]),
            Self::BigInt(col) | Self::TimestampTZ(_, _, col) => S::from(col[index]),
            Self::Int128(col) => S::from(col[index]),
            Self::Scalar(col) | Self::Decimal75(_, _, col) => col[index],
            Self::VarChar((_, scals)) | Self::VarBinary((_, scals)) => scals[index],
        })
    }

    /// Convert a column to a vector of Scalar values with scaling
    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn to_scalar_with_scaling(self, scale: i8) -> Vec<S> {
        let scale_factor = S::pow10(u8::try_from(scale).expect("Upscale factor is nonnegative"));
        match self {
            Self::Boolean(col) => slice_cast_with(col, |b| S::from(b) * scale_factor),
            Self::Decimal75(_, _, col) => slice_cast_with(col, |s| *s * scale_factor),
            Self::VarChar((_, values)) => slice_cast_with(values, |s| *s * scale_factor),
            Self::VarBinary((_, values)) => slice_cast_with(values, |s| *s * scale_factor),
            Self::Uint8(col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
            Self::TinyInt(col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
            Self::SmallInt(col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
            Self::Int(col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
            Self::BigInt(col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
            Self::Int128(col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
            Self::Scalar(col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
            Self::TimestampTZ(_, _, col) => slice_cast_with(col, |i| S::from(i) * scale_factor),
        }
    }
}

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum ColumnType {
    /// Mapped to bool
    #[serde(alias = "BOOLEAN", alias = "boolean")]
    Boolean,
    /// Mapped to u8
    #[serde(alias = "UINT8", alias = "uint8")]
    Uint8,
    /// Mapped to i8
    #[serde(alias = "TINYINT", alias = "tinyint")]
    TinyInt,
    /// Mapped to i16
    #[serde(alias = "SMALLINT", alias = "smallint")]
    SmallInt,
    /// Mapped to i32
    #[serde(alias = "INT", alias = "int")]
    Int,
    /// Mapped to i64
    #[serde(alias = "BIGINT", alias = "bigint")]
    BigInt,
    /// Mapped to i128
    #[serde(rename = "Decimal", alias = "DECIMAL", alias = "decimal")]
    Int128,
    /// Mapped to String
    #[serde(alias = "VARCHAR", alias = "varchar")]
    VarChar,
    /// Mapped to i256
    #[serde(rename = "Decimal75", alias = "DECIMAL75", alias = "decimal75")]
    Decimal75(Precision, i8),
    /// Mapped to i64
    #[serde(alias = "TIMESTAMP", alias = "timestamp")]
    #[cfg_attr(test, proptest(skip))]
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone),
    /// Mapped to `S`
    #[serde(alias = "SCALAR", alias = "scalar")]
    #[cfg_attr(test, proptest(skip))]
    Scalar,
    /// Mapped to [u8]
    #[serde(alias = "BINARY", alias = "BINARY")]
    VarBinary,
}

impl ColumnType {
    /// Returns true if this column is numeric and false otherwise
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ColumnType::Uint8
                | ColumnType::TinyInt
                | ColumnType::SmallInt
                | ColumnType::Int
                | ColumnType::BigInt
                | ColumnType::Int128
                | ColumnType::Scalar
                | ColumnType::Decimal75(_, _)
        )
    }

    /// Returns true if this column is an integer and false otherwise
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            ColumnType::Uint8
                | ColumnType::TinyInt
                | ColumnType::SmallInt
                | ColumnType::Int
                | ColumnType::BigInt
                | ColumnType::Int128
        )
    }

    /// Returns the number of bits in the integer type if it is an integer type. Otherwise, return None.
    fn to_integer_bits(self) -> Option<usize> {
        match self {
            ColumnType::Uint8 | ColumnType::TinyInt => Some(8),
            ColumnType::SmallInt => Some(16),
            ColumnType::Int => Some(32),
            ColumnType::BigInt => Some(64),
            ColumnType::Int128 => Some(128),
            _ => None,
        }
    }

    /// Returns the [`ColumnType`] of the signed integer type with the given number of bits if it is a valid integer type.
    ///
    /// Otherwise, return None.
    fn from_signed_integer_bits(bits: usize) -> Option<Self> {
        match bits {
            8 => Some(ColumnType::TinyInt),
            16 => Some(ColumnType::SmallInt),
            32 => Some(ColumnType::Int),
            64 => Some(ColumnType::BigInt),
            128 => Some(ColumnType::Int128),
            _ => None,
        }
    }

    /// Returns the [`ColumnType`] of the unsigned integer type with the given number of bits if it is a valid integer type.
    ///
    /// Otherwise, return None.
    fn from_unsigned_integer_bits(bits: usize) -> Option<Self> {
        match bits {
            8 => Some(ColumnType::Uint8),
            _ => None,
        }
    }

    /// Returns the larger integer type of two [`ColumnType`]s if they are both integers.
    ///
    /// If either of the columns is not an integer, return None.
    #[must_use]
    pub fn max_integer_type(&self, other: &Self) -> Option<Self> {
        // If either of the columns is not an integer, return None
        if !self.is_integer() || !other.is_integer() {
            return None;
        }
        self.to_integer_bits().and_then(|self_bits| {
            other
                .to_integer_bits()
                .and_then(|other_bits| Self::from_signed_integer_bits(self_bits.max(other_bits)))
        })
    }

    /// Returns the larger integer type of two [`ColumnType`]s if they are both integers.
    ///
    /// If either of the columns is not an integer, return None.
    #[must_use]
    pub fn max_unsigned_integer_type(&self, other: &Self) -> Option<Self> {
        // If either of the columns is not an integer, return None
        if !self.is_integer() || !other.is_integer() {
            return None;
        }
        self.to_integer_bits().and_then(|self_bits| {
            other
                .to_integer_bits()
                .and_then(|other_bits| Self::from_unsigned_integer_bits(self_bits.max(other_bits)))
        })
    }

    /// Returns the precision of a [`ColumnType`] if it is converted to a decimal wrapped in `Some()`. If it can not be converted to a decimal, return None.
    #[must_use]
    pub fn precision_value(&self) -> Option<u8> {
        match self {
            Self::Uint8 | Self::TinyInt => Some(3_u8),
            Self::SmallInt => Some(5_u8),
            Self::Int => Some(10_u8),
            Self::BigInt | Self::TimestampTZ(_, _) => Some(19_u8),
            Self::Int128 => Some(39_u8),
            Self::Decimal75(precision, _) => Some(precision.value()),
            // Scalars are not in database & are only used for typeless comparisons for testing so we return 0
            // so that they do not cause errors when used in comparisons.
            Self::Scalar => Some(0_u8),
            Self::Boolean | Self::VarChar | Self::VarBinary => None,
        }
    }
    /// Returns scale of a [`ColumnType`] if it is convertible to a decimal wrapped in `Some()`. Otherwise return None.
    #[must_use]
    pub fn scale(&self) -> Option<i8> {
        match self {
            Self::Decimal75(_, scale) => Some(*scale),
            Self::TinyInt
            | Self::Uint8
            | Self::SmallInt
            | Self::Int
            | Self::BigInt
            | Self::Int128
            | Self::Scalar => Some(0),
            Self::Boolean | Self::VarBinary | Self::VarChar => None,
            Self::TimestampTZ(tu, _) => match tu {
                PoSQLTimeUnit::Second => Some(0),
                PoSQLTimeUnit::Millisecond => Some(3),
                PoSQLTimeUnit::Microsecond => Some(6),
                PoSQLTimeUnit::Nanosecond => Some(9),
            },
        }
    }

    /// Returns the byte size of the column type.
    #[must_use]
    pub fn byte_size(&self) -> usize {
        match self {
            Self::Boolean => size_of::<bool>(),
            Self::Uint8 => size_of::<u8>(),
            Self::TinyInt => size_of::<i8>(),
            Self::SmallInt => size_of::<i16>(),
            Self::Int => size_of::<i32>(),
            Self::BigInt | Self::TimestampTZ(_, _) => size_of::<i64>(),
            Self::Int128 => size_of::<i128>(),
            Self::Scalar | Self::Decimal75(_, _) | Self::VarBinary | Self::VarChar => {
                size_of::<[u64; 4]>()
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    /// Returns the bit size of the column type.
    #[must_use]
    pub fn bit_size(&self) -> u32 {
        self.byte_size() as u32 * 8
    }

    /// Returns if the column type supports signed values.
    #[must_use]
    pub const fn is_signed(&self) -> bool {
        match self {
            Self::TinyInt
            | Self::SmallInt
            | Self::Int
            | Self::BigInt
            | Self::Int128
            | Self::TimestampTZ(_, _) => true,
            Self::Decimal75(_, _)
            | Self::Scalar
            | Self::VarBinary
            | Self::VarChar
            | Self::Boolean
            | Self::Uint8 => false,
        }
    }

    /// Returns if the column type supports signed values.
    #[must_use]
    pub fn min_scalar<S: Scalar>(&self) -> Option<S> {
        match self {
            ColumnType::TinyInt => Some(S::from(i8::MIN)),
            ColumnType::SmallInt => Some(S::from(i16::MIN)),
            ColumnType::Int => Some(S::from(i32::MIN)),
            ColumnType::BigInt => Some(S::from(i64::MIN)),
            ColumnType::Int128 => Some(S::from(i128::MIN)),
            _ => None,
        }
    }
}

/// Display the column type as a str name (in all caps)
impl Display for ColumnType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Boolean => write!(f, "BOOLEAN"),
            ColumnType::Uint8 => write!(f, "UINT8"),
            ColumnType::TinyInt => write!(f, "TINYINT"),
            ColumnType::SmallInt => write!(f, "SMALLINT"),
            ColumnType::Int => write!(f, "INT"),
            ColumnType::BigInt => write!(f, "BIGINT"),
            ColumnType::Int128 => write!(f, "DECIMAL"),
            ColumnType::Decimal75(precision, scale) => {
                write!(
                    f,
                    "DECIMAL75(PRECISION: {:?}, SCALE: {scale})",
                    precision.value()
                )
            }
            ColumnType::VarChar => write!(f, "VARCHAR"),
            ColumnType::VarBinary => write!(f, "BINARY"),
            ColumnType::Scalar => write!(f, "SCALAR"),
            ColumnType::TimestampTZ(timeunit, timezone) => {
                write!(f, "TIMESTAMP(TIMEUNIT: {timeunit}, TIMEZONE: {timezone})")
            }
        }
    }
}

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnRef {
    column_id: Ident,
    table_ref: TableRef,
    column_type: ColumnType,
}

impl ColumnRef {
    /// Create a new `ColumnRef` from a table, column identifier and column type
    #[must_use]
    pub fn new(table_ref: TableRef, column_id: Ident, column_type: ColumnType) -> Self {
        Self {
            column_id,
            table_ref,
            column_type,
        }
    }

    /// Returns the table reference of this column
    #[must_use]
    pub fn table_ref(&self) -> TableRef {
        self.table_ref.clone()
    }

    /// Returns the column identifier of this column
    #[must_use]
    pub fn column_id(&self) -> Ident {
        self.column_id.clone()
    }

    /// Returns the column type of this column
    #[must_use]
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }
}

/// This type is used to represent the metadata
/// of a column in a table. Namely: it's name and type.
///
/// This is the analog of a `Field` in Apache Arrow.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnField {
    name: Ident,
    data_type: ColumnType,
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    #[must_use]
    pub fn new(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
    }

    /// Returns the name of the column
    #[must_use]
    pub fn name(&self) -> Ident {
        self.name.clone()
    }

    /// Returns the type of the column
    #[must_use]
    pub fn data_type(&self) -> ColumnType {
        self.data_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::scalar::test_scalar::TestScalar, proof_primitive::dory::DoryScalar};
    use alloc::{string::String, vec};

    #[test]
    fn column_type_serializes_to_string() {
        let column_type = ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc());
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"TimestampTZ":["Second",{"offset":0}]}"#);

        let column_type = ColumnType::Boolean;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Boolean""#);

        let column_type = ColumnType::TinyInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""TinyInt""#);

        let column_type = ColumnType::SmallInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""SmallInt""#);

        let column_type = ColumnType::Int;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Int""#);

        let column_type = ColumnType::BigInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""BigInt""#);

        let column_type = ColumnType::Int128;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Decimal""#);

        let column_type = ColumnType::VarChar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""VarChar""#);

        let column_type = ColumnType::Scalar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Scalar""#);

        let column_type = ColumnType::Decimal75(Precision::new(1).unwrap(), 0);
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"Decimal75":[1,0]}"#);
    }

    #[test]
    fn we_can_deserialize_columns_from_valid_strings() {
        let expected_column_type =
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc());
        let deserialized: ColumnType =
            serde_json::from_str(r#"{"TimestampTZ":["Second",{"offset":0}]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Boolean;
        let deserialized: ColumnType = serde_json::from_str(r#""Boolean""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::TinyInt;
        let deserialized: ColumnType = serde_json::from_str(r#""TinyInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::SmallInt;
        let deserialized: ColumnType = serde_json::from_str(r#""SmallInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int;
        let deserialized: ColumnType = serde_json::from_str(r#""Int""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::BigInt;
        let deserialized: ColumnType = serde_json::from_str(r#""BigInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::TinyInt;
        let deserialized: ColumnType = serde_json::from_str(r#""TINYINT""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::SmallInt;
        let deserialized: ColumnType = serde_json::from_str(r#""SMALLINT""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int128;
        let deserialized: ColumnType = serde_json::from_str(r#""DECIMAL""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int128;
        let deserialized: ColumnType = serde_json::from_str(r#""Decimal""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::VarChar;
        let deserialized: ColumnType = serde_json::from_str(r#""VarChar""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Scalar;
        let deserialized: ColumnType = serde_json::from_str(r#""SCALAR""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Decimal75(Precision::new(75).unwrap(), i8::MAX);
        let deserialized: ColumnType = serde_json::from_str(r#"{"Decimal75":[75, 127]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type =
            ColumnType::Decimal75(Precision::new(u8::MIN + 1).unwrap(), i8::MIN);
        let deserialized: ColumnType = serde_json::from_str(r#"{"Decimal75":[1, -128]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Decimal75(Precision::new(1).unwrap(), 0);
        let deserialized: ColumnType = serde_json::from_str(r#"{"Decimal75":[1, 0]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);
    }

    #[test]
    fn we_can_deserialize_columns_from_lowercase_or_uppercase_strings() {
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""boolean""#).unwrap(),
            ColumnType::Boolean
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""BOOLEAN""#).unwrap(),
            ColumnType::Boolean
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""bigint""#).unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""BIGINT""#).unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""TINYINT""#).unwrap(),
            ColumnType::TinyInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""tinyint""#).unwrap(),
            ColumnType::TinyInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""SMALLINT""#).unwrap(),
            ColumnType::SmallInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""smallint""#).unwrap(),
            ColumnType::SmallInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""int""#).unwrap(),
            ColumnType::Int
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""INT""#).unwrap(),
            ColumnType::Int
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""decimal""#).unwrap(),
            ColumnType::Int128
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""DECIMAL""#).unwrap(),
            ColumnType::Int128
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""VARCHAR""#).unwrap(),
            ColumnType::VarChar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""varchar""#).unwrap(),
            ColumnType::VarChar
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""SCALAR""#).unwrap(),
            ColumnType::Scalar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""scalar""#).unwrap(),
            ColumnType::Scalar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"decimal75":[1,0]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0)
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"DECIMAL75":[1,0]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0)
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"decimal75":[10,5]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(10).unwrap(), 5)
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"DECIMAL75":[1,-128]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), -128)
        );
    }

    #[test]
    fn we_cannot_deserialize_columns_from_invalid_strings() {
        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""BooLean""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Tinyint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Smallint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""iNt""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Bigint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""DecImal""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""DecImal75""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> =
            serde_json::from_str(r#"{"TimestampTZ":["Utc","Second"]}"#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Varchar""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""ScaLar""#);
        assert!(deserialized.is_err());
    }

    #[test]
    fn we_can_convert_columntype_to_json_string_and_back() {
        let boolean = ColumnType::Boolean;
        let boolean_json = serde_json::to_string(&boolean).unwrap();
        assert_eq!(boolean_json, "\"Boolean\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&boolean_json).unwrap(),
            boolean
        );

        let tinyint = ColumnType::TinyInt;
        let tinyint_json = serde_json::to_string(&tinyint).unwrap();
        assert_eq!(tinyint_json, "\"TinyInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&tinyint_json).unwrap(),
            tinyint
        );

        let smallint = ColumnType::SmallInt;
        let smallint_json = serde_json::to_string(&smallint).unwrap();
        assert_eq!(smallint_json, "\"SmallInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&smallint_json).unwrap(),
            smallint
        );

        let int = ColumnType::Int;
        let int_json = serde_json::to_string(&int).unwrap();
        assert_eq!(int_json, "\"Int\"");
        assert_eq!(serde_json::from_str::<ColumnType>(&int_json).unwrap(), int);

        let bigint = ColumnType::BigInt;
        let bigint_json = serde_json::to_string(&bigint).unwrap();
        assert_eq!(bigint_json, "\"BigInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&bigint_json).unwrap(),
            bigint
        );

        let int128 = ColumnType::Int128;
        let int128_json = serde_json::to_string(&int128).unwrap();
        assert_eq!(int128_json, "\"Decimal\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&int128_json).unwrap(),
            int128
        );

        let varchar = ColumnType::VarChar;
        let varchar_json = serde_json::to_string(&varchar).unwrap();
        assert_eq!(varchar_json, "\"VarChar\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&varchar_json).unwrap(),
            varchar
        );

        let scalar = ColumnType::Scalar;
        let scalar_json = serde_json::to_string(&scalar).unwrap();
        assert_eq!(scalar_json, "\"Scalar\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&scalar_json).unwrap(),
            scalar
        );

        let decimal75 = ColumnType::Decimal75(Precision::new(75).unwrap(), 0);
        let decimal75_json = serde_json::to_string(&decimal75).unwrap();
        assert_eq!(decimal75_json, r#"{"Decimal75":[75,0]}"#);
        assert_eq!(
            serde_json::from_str::<ColumnType>(&decimal75_json).unwrap(),
            decimal75
        );
    }

    #[test]
    fn we_can_get_the_len_of_a_column() {
        let precision = 10;
        let scale = 2;

        let scalar_values = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        // Test non-empty columns
        let column = Column::<DoryScalar>::Boolean(&[true, false, true]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<DoryScalar>::TinyInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<TestScalar>::SmallInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<TestScalar>::Int(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<TestScalar>::BigInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::VarChar((&["a", "b", "c"], &scalar_values));
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<DoryScalar>::Int128(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<TestScalar>::Scalar(&scalar_values);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let decimal_data = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        let precision = Precision::new(precision).unwrap();
        let column = Column::Decimal75(precision, scale, &decimal_data);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        // Test empty columns
        let column = Column::<DoryScalar>::Boolean(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::TinyInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::SmallInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::Int(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::BigInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::VarChar((&[], &[]));
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::Int128(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::Scalar(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column: Column<'_, TestScalar> = Column::Decimal75(precision, scale, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());
    }

    #[test]
    fn we_can_convert_owned_columns_to_columns_round_trip() {
        let alloc = Bump::new();
        // Integers
        let owned_col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::Int128(&[1, 2, 3, 4, 5]));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Booleans
        let owned_col: OwnedColumn<TestScalar> =
            OwnedColumn::Boolean(vec![true, false, true, false, true]);
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::Boolean(&[true, false, true, false, true]));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Strings
        let strs = [
            "Space and Time",
            "Tér és Idő",
            "Пространство и время",
            "Spațiu și Timp",
            "Spazju u Ħin",
        ];
        let scalars = strs.iter().map(TestScalar::from).collect::<Vec<_>>();
        let owned_col = OwnedColumn::VarChar(
            strs.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::VarChar((&strs, &scalars)));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Decimals
        let scalars: Vec<TestScalar> = [1, 2, 3, 4, 5].iter().map(TestScalar::from).collect();
        let owned_col: OwnedColumn<TestScalar> =
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), 127, scalars.clone());
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(
            col,
            Column::Decimal75(Precision::new(75).unwrap(), 127, &scalars)
        );
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);
    }

    #[test]
    fn we_can_get_the_data_size_of_a_column() {
        let column = Column::<DoryScalar>::Boolean(&[true, false, true]);
        assert_eq!(column.column_type().byte_size(), 1);
        assert_eq!(column.column_type().bit_size(), 8);

        let column = Column::<TestScalar>::TinyInt(&[1, 2, 3, 4]);
        assert_eq!(column.column_type().byte_size(), 1);
        assert_eq!(column.column_type().bit_size(), 8);

        let column = Column::<TestScalar>::SmallInt(&[1, 2, 3, 4]);
        assert_eq!(column.column_type().byte_size(), 2);
        assert_eq!(column.column_type().bit_size(), 16);

        let column = Column::<TestScalar>::Int(&[1, 2, 3]);
        assert_eq!(column.column_type().byte_size(), 4);
        assert_eq!(column.column_type().bit_size(), 32);

        let column = Column::<TestScalar>::BigInt(&[1]);
        assert_eq!(column.column_type().byte_size(), 8);
        assert_eq!(column.column_type().bit_size(), 64);

        let column = Column::<DoryScalar>::Int128(&[1, 2]);
        assert_eq!(column.column_type().byte_size(), 16);
        assert_eq!(column.column_type().bit_size(), 128);

        let scalar_values = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        let column = Column::VarChar((&["a", "b", "c", "d", "e"], &scalar_values));
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let column = Column::<TestScalar>::Scalar(&scalar_values);
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let precision = 10;
        let scale = 2;
        let decimal_data = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        let precision = Precision::new(precision).unwrap();
        let column = Column::Decimal75(precision, scale, &decimal_data);
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let column: Column<'_, DoryScalar> =
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[1, 2, 3]);
        assert_eq!(column.column_type().byte_size(), 8);
        assert_eq!(column.column_type().bit_size(), 64);
    }

    #[test]
    fn we_can_get_length_of_varbinary_column() {
        let raw_bytes: &[&[u8]] = &[b"foo", b"bar", b""];
        let scalars: Vec<TestScalar> = raw_bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();

        let column = Column::VarBinary((raw_bytes, &scalars));
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());
        assert_eq!(column.column_type(), ColumnType::VarBinary);
    }

    #[test]
    fn we_can_convert_varbinary_owned_column_to_column_and_back() {
        use bumpalo::Bump;
        let alloc = Bump::new();

        let owned_varbinary = OwnedColumn::VarBinary(vec![b"abc".to_vec(), b"xyz".to_vec()]);

        let column = Column::<TestScalar>::from_owned_column(&owned_varbinary, &alloc);
        match column {
            Column::VarBinary((bytes, scalars)) => {
                assert_eq!(bytes.len(), 2);
                assert_eq!(scalars.len(), 2);
                assert_eq!(bytes[0], b"abc");
                assert_eq!(bytes[1], b"xyz");
            }
            _ => panic!("Expected VarBinary column"),
        }

        let round_trip_owned: OwnedColumn<TestScalar> = (&column).into();
        assert_eq!(owned_varbinary, round_trip_owned);
    }

    #[test]
    fn we_can_get_min_scalar() {
        assert_eq!(
            ColumnType::TinyInt.min_scalar(),
            Some(TestScalar::from(i8::MIN))
        );
        assert_eq!(
            ColumnType::SmallInt.min_scalar(),
            Some(TestScalar::from(i16::MIN))
        );
        assert_eq!(
            ColumnType::Int.min_scalar(),
            Some(TestScalar::from(i32::MIN))
        );
        assert_eq!(
            ColumnType::BigInt.min_scalar(),
            Some(TestScalar::from(i64::MIN))
        );
        assert_eq!(
            ColumnType::Int128.min_scalar(),
            Some(TestScalar::from(i128::MIN))
        );
        assert_eq!(ColumnType::Uint8.min_scalar::<TestScalar>(), None);
        assert_eq!(ColumnType::Scalar.min_scalar::<TestScalar>(), None);
        assert_eq!(ColumnType::Boolean.min_scalar::<TestScalar>(), None);
        assert_eq!(ColumnType::VarBinary.min_scalar::<TestScalar>(), None);
        assert_eq!(
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::new(0))
                .min_scalar::<TestScalar>(),
            None
        );
        assert_eq!(
            ColumnType::Decimal75(Precision::new(1).unwrap(), 1).min_scalar::<TestScalar>(),
            None
        );
        assert_eq!(ColumnType::VarChar.min_scalar::<TestScalar>(), None);
    }

    #[test]
    fn we_can_create_nullable_column() {
        let bool_values = &[true, false, true];
        let column: Column<'_, TestScalar> = Column::Boolean(bool_values);

        let nullable_column = NullableColumn::new(column);
        assert_eq!(nullable_column.len(), 3);
        assert!(!nullable_column.is_empty());
        assert!(!nullable_column.is_nullable());

        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }

        let presence = &[true, true, true];
        let nullable_column = NullableColumn::with_presence(column, Some(presence)).unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }

        let presence = &[true, false, true];
        let nullable_column = NullableColumn::with_presence(column, Some(presence)).unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));
    }

    #[test]
    fn nullable_column_returns_error_if_presence_length_mismatch() {
        let bool_values = &[true, false, true];
        let column: Column<'_, TestScalar> = Column::Boolean(bool_values);

        let presence = &[true, false];
        let result = NullableColumn::with_presence(column, Some(presence));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TableError::PresenceLengthMismatch
        ));
    }

    #[test]
    fn nullable_column_scalar_at_works_correctly() {
        let alloc = Bump::new();
        let scalar_values = [
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];
        let column = Column::Scalar(&scalar_values);

        let nullable_column = NullableColumn::new(column);

        assert_eq!(
            nullable_column.scalar_at(0),
            Some(Some(TestScalar::from(10)))
        );
        assert_eq!(
            nullable_column.scalar_at(1),
            Some(Some(TestScalar::from(20)))
        );
        assert_eq!(
            nullable_column.scalar_at(2),
            Some(Some(TestScalar::from(30)))
        );

        let presence = alloc.alloc_slice_copy(&[true, false, true]);
        let nullable_column = NullableColumn::with_presence(column, Some(presence)).unwrap();

        assert_eq!(
            nullable_column.scalar_at(0),
            Some(Some(TestScalar::from(10)))
        );
        assert_eq!(nullable_column.scalar_at(1), Some(None));
        assert_eq!(
            nullable_column.scalar_at(2),
            Some(Some(TestScalar::from(30)))
        );
    }

    #[test]
    fn we_can_convert_owned_nullable_columns_to_nullable_columns() {
        let alloc = Bump::new();

        // Test with Boolean column
        let bool_values = vec![true, false, true];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Boolean(bool_values);
        let presence = Some(vec![true, false, true]);
        let owned_nullable_column =
            OwnedNullableColumn::with_presence(owned_column.clone(), presence).unwrap();

        let nullable_column =
            NullableColumn::from_owned_nullable_column(&owned_nullable_column, &alloc);

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));
    }

    #[test]
    fn we_can_create_owned_nullable_column() {
        let bool_values = vec![true, false, true];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Boolean(bool_values);

        let presence = Some(vec![true, true, true]);
        let nullable_column =
            OwnedNullableColumn::with_presence(owned_column.clone(), presence).unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(!nullable_column.is_empty());
        assert!(nullable_column.is_nullable());

        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }

        let presence = Some(vec![true, false, true]);
        let nullable_column = OwnedNullableColumn::with_presence(owned_column, presence).unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));
    }

    #[test]
    fn nullable_column_column_type_works_correctly() {
        let bool_values = &[true, false, true];
        let bool_column: Column<'_, TestScalar> = Column::Boolean(bool_values);
        let nullable_bool_column = NullableColumn::new(bool_column);
        assert_eq!(nullable_bool_column.column_type(), ColumnType::Boolean);

        let int_values = &[10, 20, 30];
        let int_column: Column<'_, TestScalar> = Column::Int(int_values);
        let nullable_int_column = NullableColumn::new(int_column);
        assert_eq!(nullable_int_column.column_type(), ColumnType::Int);

        let scalar_values = &[
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];
        let scalar_column: Column<'_, TestScalar> = Column::Scalar(scalar_values);
        let nullable_scalar_column = NullableColumn::new(scalar_column);
        assert_eq!(nullable_scalar_column.column_type(), ColumnType::Scalar);
    }

    #[test]
    fn nullable_column_is_nullable_works_correctly() {
        let bool_values = &[true, false, true];
        let column: Column<'_, TestScalar> = Column::Boolean(bool_values);
        let nullable_column = NullableColumn::new(column);
        assert!(!nullable_column.is_nullable());

        let presence = &[true, true, true];
        let nullable_column = NullableColumn::with_presence(column, Some(presence)).unwrap();
        assert!(nullable_column.is_nullable());

        let nullable_column = NullableColumn::with_presence(column, None).unwrap();
        assert!(!nullable_column.is_nullable());
    }

    #[test]
    fn nullable_column_is_null_edge_cases() {
        let alloc = Bump::new();
        let bool_values = &[true, false, true];
        let column: Column<'_, TestScalar> = Column::Boolean(bool_values);
        let presence = alloc.alloc_slice_copy(&[true, true, true]);
        let nullable_column = NullableColumn::with_presence(column, Some(presence)).unwrap();
        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }

        let presence = alloc.alloc_slice_copy(&[false, false, false]);
        let nullable_column = NullableColumn::with_presence(column, Some(presence)).unwrap();
        for i in 0..3 {
            assert!(nullable_column.is_null(i));
        }

        let presence = alloc.alloc_slice_copy(&[false, true, false]);
        let nullable_column = NullableColumn::with_presence(column, Some(presence)).unwrap();
        assert!(nullable_column.is_null(0));
        assert!(!nullable_column.is_null(1));
        assert!(nullable_column.is_null(2));

        let nullable_column = NullableColumn::new(column);
        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn nullable_column_is_null_panics_on_out_of_bounds() {
        let bool_values = &[true, false, true];
        let column: Column<'_, TestScalar> = Column::Boolean(bool_values);
        let nullable_column = NullableColumn::new(column);

        // This should panic
        let _ = nullable_column.is_null(3);
    }

    #[test]
    fn nullable_column_scalar_at_with_different_column_types() {
        let alloc = Bump::new();
        let bool_values = &[true, false, true];
        let bool_column: Column<'_, TestScalar> = Column::Boolean(bool_values);
        let presence = alloc.alloc_slice_copy(&[true, false, true]);
        let nullable_bool_column =
            NullableColumn::with_presence(bool_column, Some(presence)).unwrap();

        assert_eq!(
            nullable_bool_column.scalar_at(0),
            Some(Some(TestScalar::from(1)))
        );
        assert_eq!(nullable_bool_column.scalar_at(1), Some(None));
        assert_eq!(
            nullable_bool_column.scalar_at(2),
            Some(Some(TestScalar::from(1)))
        );

        let int_values = &[10, 20, 30];
        let int_column: Column<'_, TestScalar> = Column::Int(int_values);
        let presence = alloc.alloc_slice_copy(&[true, false, true]);
        let nullable_int_column =
            NullableColumn::with_presence(int_column, Some(presence)).unwrap();

        assert_eq!(
            nullable_int_column.scalar_at(0),
            Some(Some(TestScalar::from(10)))
        );
        assert_eq!(nullable_int_column.scalar_at(1), Some(None));
        assert_eq!(
            nullable_int_column.scalar_at(2),
            Some(Some(TestScalar::from(30)))
        );

        let str_values = &["hello", "world", "test"];
        let hash_values = &[
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let varchar_column: Column<'_, TestScalar> = Column::VarChar((str_values, hash_values));
        let presence = alloc.alloc_slice_copy(&[true, false, true]);
        let nullable_varchar_column =
            NullableColumn::with_presence(varchar_column, Some(presence)).unwrap();

        assert_eq!(
            nullable_varchar_column.scalar_at(0),
            Some(Some(TestScalar::from(1)))
        );
        assert_eq!(nullable_varchar_column.scalar_at(1), Some(None));
        assert_eq!(
            nullable_varchar_column.scalar_at(2),
            Some(Some(TestScalar::from(3)))
        );
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn nullable_column_scalar_at_panics_on_out_of_bounds() {
        let scalar_values = &[
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];
        let column = Column::Scalar(scalar_values);
        let nullable_column = NullableColumn::new(column);

        // This should panic
        let _ = nullable_column.scalar_at(3);
    }

    #[test]
    fn nullable_column_with_presence_various_scenarios() {
        let bool_values = &[true, false, true];
        let column: Column<'_, TestScalar> = Column::Boolean(bool_values);
        let result = NullableColumn::with_presence(column, None);
        assert!(result.is_ok());
        let nullable_column = result.unwrap();
        assert!(!nullable_column.is_nullable());

        let presence = &[true, true, true];
        let result = NullableColumn::with_presence(column, Some(presence));
        assert!(result.is_ok());
        let nullable_column = result.unwrap();
        assert!(nullable_column.is_nullable());

        let presence = &[true, false];
        let result = NullableColumn::with_presence(column, Some(presence));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TableError::PresenceLengthMismatch
        ));

        let presence = &[true, false, true, false];
        let result = NullableColumn::with_presence(column, Some(presence));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TableError::PresenceLengthMismatch
        ));
    }

    #[test]
    fn nullable_column_from_owned_nullable_columns_to_nullable_columns() {
        let alloc = Bump::new();
        let bool_values = vec![true, false, true];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Boolean(bool_values);
        let presence = Some(vec![true, false, true]);
        let owned_nullable_column =
            OwnedNullableColumn::with_presence(owned_column, presence).unwrap();

        let nullable_column =
            NullableColumn::from_owned_nullable_column(&owned_nullable_column, &alloc);
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        let int_values = vec![10, 20, 30];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Int(int_values);
        let presence = Some(vec![false, true, false]);
        let owned_nullable_column =
            OwnedNullableColumn::with_presence(owned_column, presence).unwrap();

        let nullable_column =
            NullableColumn::from_owned_nullable_column(&owned_nullable_column, &alloc);
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(nullable_column.is_null(0));
        assert!(!nullable_column.is_null(1));
        assert!(nullable_column.is_null(2));

        let scalar_values = vec![
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Scalar(scalar_values);
        let owned_nullable_column = OwnedNullableColumn::new(owned_column);

        let nullable_column =
            NullableColumn::from_owned_nullable_column(&owned_nullable_column, &alloc);
        assert_eq!(nullable_column.len(), 3);
        assert!(!nullable_column.is_nullable());
        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }
    }

    #[test]
    fn nullable_column_empty_works_correctly() {
        let empty_bool_values: &[bool] = &[];
        let empty_column: Column<'_, TestScalar> = Column::Boolean(empty_bool_values);

        let nullable_column = NullableColumn::new(empty_column);
        assert_eq!(nullable_column.len(), 0);
        assert!(nullable_column.is_empty());

        let empty_presence: &[bool] = &[];
        let result = NullableColumn::with_presence(empty_column, Some(empty_presence));
        assert!(result.is_ok());
        let nullable_column = result.unwrap();
        assert_eq!(nullable_column.len(), 0);
        assert!(nullable_column.is_empty());
        assert!(nullable_column.is_nullable());
    }
}
