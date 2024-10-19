use super::{LiteralValue, OwnedColumn, TableRef};
use crate::base::{
    math::decimal::{scale_scalar, Precision},
    scalar::Scalar,
    slice_ops::slice_cast_with,
};
use alloc::{sync::Arc, vec::Vec};
use ark_std::iterable::Iterable;
#[cfg(feature = "arrow")]
use arrow::datatypes::{DataType, Field, TimeUnit as ArrowTimeUnit};
use bumpalo::Bump;
use core::{
    fmt,
    fmt::{Display, Formatter},
    mem::size_of,
};
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    Identifier,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum ColumnKind<'a, S: Scalar> {
    /// Boolean columns
    Boolean(&'a [bool]),
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
    ///  - the backing store maps to the type [`crate::base::scalar::Curve25519Scalar`]
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
}

impl<'a, S: Scalar> ColumnKind<'a, S> {
    pub fn get_type_kind(&self) -> ColumnTypeKind {
        match *self {
            Self::Boolean(_) => ColumnTypeKind::Boolean,
            Self::TinyInt(_) => ColumnTypeKind::TinyInt,
            Self::SmallInt(_) => ColumnTypeKind::SmallInt,
            Self::Int(_) => ColumnTypeKind::Int,
            Self::BigInt(_) => ColumnTypeKind::BigInt,
            Self::VarChar(_) => ColumnTypeKind::VarChar,
            Self::Int128(_) => ColumnTypeKind::Int128,
            Self::Scalar(_) => ColumnTypeKind::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnTypeKind::Decimal75(*precision, *scale),
            Self::TimestampTZ(time_unit, timezone, _) => {
                ColumnTypeKind::TimestampTZ(*time_unit, *timezone)
            }
        }
    }
    /// Returns the length of the column.
    /// # Panics
    /// this function requires that `col` and `scals` have the same length.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Boolean(col) => col.len(),
            Self::TinyInt(col) => col.len(),
            Self::SmallInt(col) => col.len(),
            Self::Int(col) => col.len(),
            Self::BigInt(col) | Self::TimestampTZ(_, _, col) => col.len(),
            Self::VarChar((col, scals)) => {
                assert_eq!(col.len(), scals.len());
                col.len()
            }
            Self::Int128(col) => col.len(),
            Self::Scalar(col) | Self::Decimal75(_, _, col) => col.len(),
        }
    }
    /// Generate a constant column from a literal value with a given length
    pub fn from_literal_with_length(
        literal: &LiteralValue<S>,
        length: usize,
        alloc: &'a Bump,
    ) -> Self {
        match literal {
            LiteralValue::Boolean(value) => {
                Self::Boolean(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::TinyInt(value) => {
                Self::TinyInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::SmallInt(value) => {
                Self::SmallInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Int(value) => Self::Int(alloc.alloc_slice_fill_copy(length, *value)),
            LiteralValue::BigInt(value) => {
                Self::BigInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Int128(value) => {
                Self::Int128(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Scalar(value) => {
                Self::Scalar(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Decimal75(precision, scale, value) => Self::Decimal75(
                *precision,
                *scale,
                alloc.alloc_slice_fill_copy(length, *value),
            ),
            LiteralValue::TimeStampTZ(tu, tz, value) => {
                Self::TimestampTZ(*tu, *tz, alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::VarChar((string, scalar)) => Self::VarChar((
                alloc.alloc_slice_fill_with(length, |_| alloc.alloc_str(string) as &str),
                alloc.alloc_slice_fill_copy(length, *scalar),
            )),
        }
    }

    /// Returns the column as a slice of booleans if it is a boolean column. Otherwise, returns None.
    pub(crate) fn as_boolean(&self) -> Option<&'a [bool]> {
        match self {
            Self::Boolean(col) => Some(col),
            _ => None,
        }
    }
    /// Returns the column as a slice of scalars
    pub(crate) fn as_scalar(&self, alloc: &'a Bump) -> &'a [S] {
        match self {
            Self::Boolean(col) => alloc.alloc_slice_fill_with(col.len(), |i| S::from(col[i])),
            Self::TinyInt(col) => alloc.alloc_slice_fill_with(col.len(), |i| S::from(col[i])),
            Self::SmallInt(col) => alloc.alloc_slice_fill_with(col.len(), |i| S::from(col[i])),
            Self::Int(col) => alloc.alloc_slice_fill_with(col.len(), |i| S::from(col[i])),
            Self::BigInt(col) => alloc.alloc_slice_fill_with(col.len(), |i| S::from(col[i])),
            Self::Int128(col) => alloc.alloc_slice_fill_with(col.len(), |i| S::from(col[i])),
            Self::Scalar(col) | Self::Decimal75(_, _, col) => col,
            Self::VarChar((_, scals)) => scals,
            Self::TimestampTZ(_, _, col) => {
                alloc.alloc_slice_fill_with(col.len(), |i| S::from(col[i]))
            }
        }
    }
    /// Returns element at index as scalar
    ///
    /// Note that if index is out of bounds, this function will return None
    pub(crate) fn scalar_at(&self, index: usize) -> Option<S> {
        (index < self.len()).then_some(match self {
            Self::Boolean(col) => S::from(col[index]),
            Self::TinyInt(col) => S::from(col[index]),
            Self::SmallInt(col) => S::from(col[index]),
            Self::Int(col) => S::from(col[index]),
            Self::BigInt(col) | Self::TimestampTZ(_, _, col) => S::from(col[index]),
            Self::Int128(col) => S::from(col[index]),
            Self::Scalar(col) | Self::Decimal75(_, _, col) => col[index],
            Self::VarChar((_, scals)) => scals[index],
        })
    }

    /// Convert a column to a vector of Scalar values with scaling
    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn to_scalar_with_scaling(self, scale: i8) -> Vec<S> {
        let scale_factor = scale_scalar(S::ONE, scale).expect("Invalid scale factor");
        match self {
            Self::Boolean(col) => slice_cast_with(col, |b| S::from(b) * scale_factor),
            Self::Decimal75(_, _, col) => slice_cast_with(col, |s| *s * scale_factor),
            Self::VarChar((_, values)) => slice_cast_with(values, |s| *s * scale_factor),
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
/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Column<'a, S: Scalar> {
    kind: ColumnKind<'a, S>,
    nullable: bool,
}

impl<'a, S: Scalar> Column<'a, S> {
    pub fn new(kind: ColumnKind<'a, S>, nullable: bool) -> Self {
        Self { kind, nullable }
    }
    /// Can null be stored in this column
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }
    /// Can null be stored in this column
    pub fn get_column_kind(&self) -> ColumnKind<'a, S> {
        self.kind
    }
    /// Provides the column type associated with the column
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        ColumnType {
            kind: self.kind.get_type_kind(),
            nullable: self.nullable,
        }
    }
    /// Returns the length of the column.
    /// # Panics
    /// this function requires that `col` and `scals` have the same length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.kind.len()
    }
    /// Returns `true` if the column has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Generate a constant column from a literal value with a given length
    pub fn from_literal_with_length(
        literal: &LiteralValue<S>,
        length: usize,
        alloc: &'a Bump,
    ) -> Self {
        Self::new(
            ColumnKind::from_literal_with_length(literal, length, alloc),
            false,
        )
    }

    /// Convert an `OwnedColumn` to a `Column`
    pub fn from_owned_column(owned_column: &'a OwnedColumn<S>, alloc: &'a Bump) -> Self {
        match owned_column {
            OwnedColumn::Boolean(meta, col) => Column::Boolean(*meta, col.as_slice()),
            OwnedColumn::TinyInt(meta, col) => Column::TinyInt(*meta, col.as_slice()),
            OwnedColumn::SmallInt(meta, col) => Column::SmallInt(*meta, col.as_slice()),
            OwnedColumn::Int(meta, col) => Column::Int(*meta, col.as_slice()),
            OwnedColumn::BigInt(meta, col) => Column::BigInt(*meta, col.as_slice()),
            OwnedColumn::Int128(meta, col) => Column::Int128(*meta, col.as_slice()),
            OwnedColumn::Decimal75(meta, precision, scale, col) => {
                Column::Decimal75(*meta, *precision, *scale, col.as_slice())
            }
            OwnedColumn::Scalar(meta, col) => Column::Scalar(*meta, col.as_slice()),
            OwnedColumn::VarChar(meta, col) => {
                let scalars = col.iter().map(S::from).collect::<Vec<_>>();
                let strs = col
                    .iter()
                    .map(|s| s.as_str() as &'a str)
                    .collect::<Vec<_>>();
                Column::VarChar(
                    *meta,
                    (
                        alloc.alloc_slice_clone(strs.as_slice()),
                        alloc.alloc_slice_copy(scalars.as_slice()),
                    ),
                )
            }
            OwnedColumn::TimestampTZ(meta, tu, tz, col) => {
                Column::TimestampTZ(*meta, *tu, *tz, col.as_slice())
            }
        }
    }

    /// Returns the column as a slice of booleans if it is a boolean column. Otherwise, returns None.
    pub(crate) fn as_boolean(&self) -> Option<&'a [bool]> {
        self.kind.as_boolean()
    }

    /// Returns the column as a slice of scalars
    pub(crate) fn as_scalar(&self, alloc: &'a Bump) -> &'a [S] {
        self.kind.as_scalar(alloc)
    }

    /// Returns element at index as scalar
    ///
    /// Note that if index is out of bounds, this function will return None
    pub(crate) fn scalar_at(&self, index: usize) -> Option<S> {
        self.kind.scalar_at(index)
    }

    /// Convert a column to a vector of Scalar values with scaling
    #[allow(clippy::missing_panics_doc)]
    pub(crate) fn to_scalar_with_scaling(self, scale: i8) -> Vec<S> {
        self.to_scalar_with_scaling(scale)
    }
}
/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
pub struct ColumnType {
    kind: ColumnTypeKind,
    nullable: bool,
}
impl Display for ColumnType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.nullable {
            f.write_str("NOT NULL")
        } else {
            Ok(())
        }
    }
}
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
pub enum ColumnTypeKind {
    /// Mapped to bool
    #[serde(alias = "BOOLEAN", alias = "boolean")]
    Boolean,
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
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone),
    /// Mapped to [`Curve25519Scalar`](crate::base::scalar::Curve25519Scalar)
    #[serde(alias = "SCALAR", alias = "scalar")]
    Scalar,
}
impl ColumnTypeKind {
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ColumnTypeKind::TinyInt
                | ColumnTypeKind::SmallInt
                | ColumnTypeKind::Int
                | ColumnTypeKind::BigInt
                | ColumnTypeKind::Int128
                | ColumnTypeKind::Scalar
                | ColumnTypeKind::Decimal75(_, _)
        )
    }
    /// Returns true if this column is an integer and false otherwise
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            ColumnTypeKind::TinyInt
                | ColumnTypeKind::SmallInt
                | ColumnTypeKind::Int
                | ColumnTypeKind::BigInt
                | ColumnTypeKind::Int128
        )
    }
    /// Returns the number of bits in the integer type if it is an integer type. Otherwise, return None.
    pub(self) fn to_integer_bits(self) -> Option<usize> {
        match self {
            Self::TinyInt => Some(8),
            Self::SmallInt => Some(16),
            Self::Int => Some(32),
            Self::BigInt => Some(64),
            Self::Int128 => Some(128),
            _ => None,
        }
    }

    /// Returns the [`ColumnType`] of the integer type with the given number of bits if it is a valid integer type.
    ///
    /// Otherwise, return None.from_literal_with_length
    pub(self) fn from_integer_bits(bits: usize) -> Option<Self> {
        match bits {
            8 => Some(ColumnTypeKind::TinyInt),
            16 => Some(ColumnTypeKind::SmallInt),
            32 => Some(ColumnTypeKind::Int),
            64 => Some(ColumnTypeKind::BigInt),
            128 => Some(ColumnTypeKind::Int128),
            _ => None,
        }
    }

    /// Returns the precision of a [`ColumnTypeKind`] if it is converted to a decimal wrapped in `Some()`. If it can not be converted to a decimal, return None.
    #[must_use]
    pub fn precision_value(&self) -> Option<u8> {
        match self {
            Self::TinyInt => Some(3_u8),
            Self::SmallInt => Some(5_u8),
            Self::Int => Some(10_u8),
            Self::BigInt | Self::TimestampTZ(_, _) => Some(19_u8),
            Self::Int128 => Some(39_u8),
            Self::Decimal75(precision, _) => Some(precision.value()),
            // Scalars are not in database & are only used for typeless comparisons for testing so we return 0
            // so that they do not cause errors when used in comparisons.
            Self::Scalar => Some(0_u8),
            Self::Boolean | Self::VarChar => None,
        }
    }
    /// Returns scale of a [`ColumnTypeKind`] if it is convertible to a decimal wrapped in `Some()`. Otherwise return None.
    #[must_use]
    pub fn scale(&self) -> Option<i8> {
        match self {
            Self::Decimal75(_, scale) => Some(*scale),
            Self::TinyInt
            | Self::SmallInt
            | Self::Int
            | Self::BigInt
            | Self::Int128
            | Self::Scalar => Some(0),
            Self::Boolean | Self::VarChar => None,
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
            Self::TinyInt => size_of::<i8>(),
            Self::SmallInt => size_of::<i16>(),
            Self::Int => size_of::<i32>(),
            Self::BigInt | Self::TimestampTZ(_, _) => size_of::<i64>(),
            Self::Int128 => size_of::<i128>(),
            Self::Scalar | Self::Decimal75(_, _) | Self::VarChar => size_of::<[u64; 4]>(),
        }
    }
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
            Self::Decimal75(_, _) | Self::Scalar | Self::VarChar | Self::Boolean => false,
        }
    }
}

/// Display the column type as a str name (in all caps)
impl Display for ColumnTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => f.write_str("BOOLEAN"),
            Self::TinyInt => f.write_str("TINYINT"),
            Self::SmallInt => f.write_str("SMALLINT"),
            Self::Int => f.write_str("INT"),
            Self::BigInt => f.write_str("BIGINT"),
            Self::Int128 => f.write_str("DECIMAL"),
            Self::Decimal75(precision, scale) => {
                write!(
                    f,
                    "DECIMAL75(PRECISION: {:?}, SCALE: {scale})",
                    precision.value()
                )
            }
            Self::VarChar => f.write_str("VARCHAR"),
            Self::Scalar => f.write_str("SCALAR"),
            Self::TimestampTZ(timeunit, timezone) => {
                write!(f, "TIMESTAMP(TIMEUNIT: {timeunit}, TIMEZONE: {timezone})")
            }
        }
    }
}
impl ColumnType {
    pub fn new(kind: ColumnTypeKind, nullable: bool) -> Self {
        Self { kind, nullable }
    }
    pub fn get_kind(&self) -> ColumnTypeKind {
        self.kind
    }
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }
    /// Returns true if this column is numeric and false otherwise
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        self.kind.is_numeric()
    }

    /// Returns true if this column is an integer and false otherwise
    #[must_use]
    pub fn is_integer(&self) -> bool {
        self.is_integer()
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
            other.to_integer_bits().and_then(|other_bits| {
                ColumnType::new(
                    ColumnTypeKind::from_integer_bits(self_bits),
                    self.nullable || other.nullable,
                )
            })
        })
    }

    /// Returns the precision of a [`ColumnType`] if it is converted to a decimal wrapped in `Some()`. If it can not be converted to a decimal, return None.
    #[must_use]
    pub fn precision_value(&self) -> Option<u8> {
        self.kind.precision_value()
    }
    /// Returns scale of a [`ColumnType`] if it is convertible to a decimal wrapped in `Some()`. Otherwise return None.
    #[must_use]
    pub fn scale(&self) -> Option<i8> {
        self.kind.scale()
    }

    /// Returns the byte size of the column type.
    #[must_use]
    pub fn byte_size(&self) -> usize {
        self.kind.byte_size()
    }

    /// Returns the bit size of the column type.
    #[must_use]
    pub fn bit_size(&self) -> u32 {
        self.kind.bit_size()
    }

    /// Returns if the column type supports signed values.
    #[must_use]
    pub const fn is_signed(&self) -> bool {
        self.kind.is_signed()
    }
}

/// Convert [`ColumnType`] values to some arrow [`DataType`]
#[cfg(feature = "arrow")]
impl From<&ColumnTypeKind> for DataType {
    fn from(column_type: &ColumnTypeKind) -> Self {
        match column_type {
            ColumnTypeKind::Boolean => DataType::Boolean,
            ColumnTypeKind::TinyInt => DataType::Int8,
            ColumnTypeKind::SmallInt => DataType::Int16,
            ColumnTypeKind::Int => DataType::Int32,
            ColumnTypeKind::BigInt => DataType::Int64,
            ColumnTypeKind::Int128 => DataType::Decimal128(38, 0),
            ColumnTypeKind::Decimal75(precision, scale) => {
                DataType::Decimal256(precision.value(), *scale)
            }
            ColumnTypeKind::VarChar => DataType::Utf8,
            ColumnTypeKind::Scalar => unimplemented!("Cannot convert Scalar type to arrow type"),
            ColumnTypeKind::TimestampTZ(timeunit, timezone) => {
                let arrow_timezone = Some(Arc::from(timezone.to_string()));
                let arrow_timeunit = match timeunit {
                    PoSQLTimeUnit::Second => ArrowTimeUnit::Second,
                    PoSQLTimeUnit::Millisecond => ArrowTimeUnit::Millisecond,
                    PoSQLTimeUnit::Microsecond => ArrowTimeUnit::Microsecond,
                    PoSQLTimeUnit::Nanosecond => ArrowTimeUnit::Nanosecond,
                };
                DataType::Timestamp(arrow_timeunit, arrow_timezone)
            }
        }
    }
}

/// Convert arrow [`DataType`] values to some [`ColumnType`]
#[cfg(feature = "arrow")]
impl TryFrom<DataType> for ColumnTypeKind {
    type Error = String;

    fn try_from(data_type: DataType) -> Result<Self, Self::Error> {
        match data_type {
            DataType::Boolean => Ok(ColumnTypeKind::Boolean),
            DataType::Int8 => Ok(ColumnTypeKind::TinyInt),
            DataType::Int16 => Ok(ColumnTypeKind::SmallInt),
            DataType::Int32 => Ok(ColumnTypeKind::Int),
            DataType::Int64 => Ok(ColumnTypeKind::BigInt),
            DataType::Decimal128(38, 0) => Ok(ColumnTypeKind::Int128),
            DataType::Decimal256(precision, scale) if precision <= 75 => {
                Ok(ColumnTypeKind::Decimal75(Precision::new(precision)?, scale))
            }
            DataType::Timestamp(time_unit, timezone_option) => {
                let posql_time_unit = match time_unit {
                    ArrowTimeUnit::Second => PoSQLTimeUnit::Second,
                    ArrowTimeUnit::Millisecond => PoSQLTimeUnit::Millisecond,
                    ArrowTimeUnit::Microsecond => PoSQLTimeUnit::Microsecond,
                    ArrowTimeUnit::Nanosecond => PoSQLTimeUnit::Nanosecond,
                };
                Ok(ColumnTypeKind::TimestampTZ(
                    posql_time_unit,
                    PoSQLTimeZone::try_from(&timezone_option)?,
                ))
            }
            DataType::Utf8 => Ok(ColumnTypeKind::VarChar),
            _ => Err(format!("Unsupported arrow data type {data_type:?}")),
        }
    }
}

/// Display the column type as a str name (in all caps)
impl Display for ColumnType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let my = self.kind;
        let meta = if self.nullable { "" } else { "NOT NULL" };
        write!(f, "{my} {meta}")
    }
}

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy, Serialize, Deserialize)]
pub struct ColumnRef {
    column_id: Identifier,
    table_ref: TableRef,
    column_type: ColumnType,
}

impl ColumnRef {
    /// Create a new `ColumnRef` from a table, column identifier and column type
    #[must_use]
    pub fn new(table_ref: TableRef, column_id: Identifier, column_type: ColumnType) -> Self {
        Self {
            column_id,
            table_ref,
            column_type,
        }
    }

    /// Returns the table reference of this column
    #[must_use]
    pub fn table_ref(&self) -> TableRef {
        self.table_ref
    }

    /// Returns the column identifier of this column
    #[must_use]
    pub fn column_id(&self) -> Identifier {
        self.column_id
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
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy, Serialize, Deserialize)]
pub struct ColumnField {
    name: Identifier,
    data_type: ColumnType,
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    #[must_use]
    pub fn new(name: Identifier, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
    }

    /// Returns the name of the column
    #[must_use]
    pub fn name(&self) -> Identifier {
        self.name
    }

    /// Returns the type of the column
    #[must_use]
    pub fn data_type(&self) -> ColumnType {
        self.data_type
    }
}

/// Convert [`ColumnField`] values to arrow Field
#[cfg(feature = "arrow")]
impl From<&ColumnField> for Field {
    fn from(column_field: &ColumnField) -> Self {
        Field::new(
            column_field.name().name(),
            (&column_field.data_type()).into(),
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::scalar::Curve25519Scalar, proof_primitive::dory::DoryScalar};
    use alloc::{string::String, vec};

    #[test]
    fn column_type_serializes_to_string() {
        let column_type = ColumnTypeKind::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc);
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"TimestampTZ":["Second","Utc"]}"#);

        let column_type = ColumnTypeKind::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc);
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"TimestampTZ":["Second","Utc"]} NOT NULL"#);

        let column_type = ColumnTypeKind::Boolean;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Boolean""#);

        let column_type = ColumnTypeKind::TinyInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""TinyInt""#);

        let column_type = ColumnTypeKind::SmallInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""SmallInt""#);

        let column_type = ColumnTypeKind::Int;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Int""#);

        let column_type = ColumnTypeKind::BigInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""BigInt""#);

        let column_type = ColumnTypeKind::Int128;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Decimal""#);

        let column_type = ColumnTypeKind::VarChar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""VarChar""#);

        let column_type = ColumnTypeKind::Scalar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Scalar""#);

        let column_type = ColumnTypeKind::Decimal75(Precision::new(1).unwrap(), 0);
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"Decimal75":[1,0]}"#);
    }

    #[test]
    fn we_can_deserialize_columns_from_valid_strings() {
        let expected_column_type =
            ColumnTypeKind::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc);
        let deserialized: ColumnTypeKind =
            serde_json::from_str(r#"{"TimestampTZ":["Second","Utc"]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::Boolean;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""Boolean""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::TinyInt;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""TinyInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::SmallInt;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""SmallInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::Int;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""Int""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::BigInt;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""BigInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::TinyInt;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""TINYINT""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::SmallInt;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""SMALLINT""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::Int128;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""DECIMAL""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::Int128;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""Decimal""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::VarChar;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""VarChar""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::Scalar;
        let deserialized: ColumnTypeKind = serde_json::from_str(r#""SCALAR""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::Decimal75(Precision::new(75).unwrap(), i8::MAX);
        let deserialized: ColumnTypeKind =
            serde_json::from_str(r#"{"Decimal75":[75, 127]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type =
            ColumnTypeKind::Decimal75(Precision::new(u8::MIN + 1).unwrap(), i8::MIN);
        let deserialized: ColumnTypeKind =
            serde_json::from_str(r#"{"Decimal75":[1, -128]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnTypeKind::Decimal75(Precision::new(1).unwrap(), 0);
        let deserialized: ColumnTypeKind = serde_json::from_str(r#"{"Decimal75":[1, 0]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);
    }

    #[test]
    fn we_can_deserialize_columns_from_lowercase_or_uppercase_strings() {
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""boolean""#).unwrap(),
            ColumnTypeKind::Boolean
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""BOOLEAN""#).unwrap(),
            ColumnTypeKind::Boolean
        );

        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""bigint""#).unwrap(),
            ColumnTypeKind::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""BIGINT""#).unwrap(),
            ColumnTypeKind::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""TINYINT""#).unwrap(),
            ColumnTypeKind::TinyInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""tinyint""#).unwrap(),
            ColumnTypeKind::TinyInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""SMALLINT""#).unwrap(),
            ColumnTypeKind::SmallInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""smallint""#).unwrap(),
            ColumnTypeKind::SmallInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""int""#).unwrap(),
            ColumnTypeKind::Int
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""INT""#).unwrap(),
            ColumnTypeKind::Int
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""decimal""#).unwrap(),
            ColumnTypeKind::Int128
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""DECIMAL""#).unwrap(),
            ColumnTypeKind::Int128
        );

        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""VARCHAR""#).unwrap(),
            ColumnTypeKind::VarChar
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""varchar""#).unwrap(),
            ColumnTypeKind::VarChar
        );

        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""SCALAR""#).unwrap(),
            ColumnTypeKind::Scalar
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#""scalar""#).unwrap(),
            ColumnTypeKind::Scalar
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#"{"decimal75":[1,0]}"#).unwrap(),
            ColumnTypeKind::Decimal75(Precision::new(1).unwrap(), 0)
        );
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#"{"DECIMAL75":[1,0]}"#).unwrap(),
            ColumnTypeKind::Decimal75(Precision::new(1).unwrap(), 0)
        );

        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#"{"decimal75":[10,5]}"#).unwrap(),
            ColumnTypeKind::Decimal75(Precision::new(10).unwrap(), 5)
        );

        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(r#"{"DECIMAL75":[1,-128]}"#).unwrap(),
            ColumnTypeKind::Decimal75(Precision::new(1).unwrap(), -128)
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
        let boolean = ColumnTypeKind::Boolean;
        let boolean_json = serde_json::to_string(&boolean).unwrap();
        assert_eq!(boolean_json, "\"Boolean\"");
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&boolean_json).unwrap(),
            boolean
        );

        let tinyint = ColumnTypeKind::TinyInt;
        let tinyint_json = serde_json::to_string(&tinyint).unwrap();
        assert_eq!(tinyint_json, "\"TinyInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&tinyint_json).unwrap(),
            tinyint
        );

        let smallint = ColumnTypeKind::SmallInt;
        let smallint_json = serde_json::to_string(&smallint).unwrap();
        assert_eq!(smallint_json, "\"SmallInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&smallint_json).unwrap(),
            smallint
        );

        let int = ColumnTypeKind::Int;
        let int_json = serde_json::to_string(&int).unwrap();
        assert_eq!(int_json, "\"Int\"");
        assert_eq!(serde_json::from_str::<ColumnTypeKind>(&int_json).unwrap(), int);

        let bigint = ColumnTypeKind::BigInt;
        let bigint_json = serde_json::to_string(&bigint).unwrap();
        assert_eq!(bigint_json, "\"BigInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&bigint_json).unwrap(),
            bigint
        );

        let int128 = ColumnTypeKind::Int128;
        let int128_json = serde_json::to_string(&int128).unwrap();
        assert_eq!(int128_json, "\"Decimal\"");
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&int128_json).unwrap(),
            int128
        );

        let varchar = ColumnTypeKind::VarChar;
        let varchar_json = serde_json::to_string(&varchar).unwrap();
        assert_eq!(varchar_json, "\"VarChar\"");
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&varchar_json).unwrap(),
            varchar
        );

        let scalar = ColumnTypeKind::Scalar;
        let scalar_json = serde_json::to_string(&scalar).unwrap();
        assert_eq!(scalar_json, "\"Scalar\"");
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&scalar_json).unwrap(),
            scalar
        );

        let decimal75 = ColumnTypeKind::Decimal75(Precision::new(75).unwrap(), 0);
        let decimal75_json = serde_json::to_string(&decimal75).unwrap();
        assert_eq!(decimal75_json, r#"{"Decimal75":[75,0]}"#);
        assert_eq!(
            serde_json::from_str::<ColumnTypeKind>(&decimal75_json).unwrap(),
            decimal75
        );
    }

    #[test]
    fn we_can_get_the_len_of_a_column() {
        let precision = 10;
        let scale = 2;

        let scalar_values = [
            Curve25519Scalar::from(1),
            Curve25519Scalar::from(2),
            Curve25519Scalar::from(3),
        ];

        // Test non-empty columns
        let column = Column::<DoryScalar>::new(ColumnKind::Boolean( &[true, false, true]), false);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<DoryScalar>::new(ColumnKind::Int( &[1, 2, 3]), false);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<Curve25519Scalar>::new(ColumnKind::SmallInt( &[1, 2, 3]), false);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<Curve25519Scalar>::new(ColumnKind::Int(&[1, 2, 3]), false);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<Curve25519Scalar>::new(ColumnKind::BigInt( &[1, 2, 3]), false);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::new(ColumnKind::VarChar( (&["a", "b", "c"])), false);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<DoryScalar>::Int1::new(ColumnKind::28(null_meta &[1, 2, 3]));
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::Scalar(null_meta, &scalar_values);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let decimal_data = [
            Curve25519Scalar::from(1),
            Curve25519Scalar::from(2),
            Curve25519Scalar::from(3),
        ];

        let precision = Precision::new(precision).unwrap();
        let column = Column::Decimal75(null_meta, precision, scale, &decimal_data);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        // Test empty columns
        let column = Column::<DoryScalar>::Boolean(null_meta, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::TinyInt(null_meta, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::SmallInt(null_meta, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::Int(null_meta, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::BigInt(null_meta, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::VarChar(null_meta, (&[], &[]));
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::Int128(null_meta, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::Scalar(null_meta, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column: Column<'_, Curve25519Scalar> =
            Column::Decimal75(null_meta, precision, scale, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());
    }

    #[test]
    fn we_can_convert_owned_columns_to_columns_round_trip() {
        let meta = ColumnNullability::NotNullable;
        let alloc = Bump::new();
        // Integers
        let owned_col: OwnedColumn<Curve25519Scalar> =
            OwnedColumn::Int128(meta, vec![1, 2, 3, 4, 5]);
        let col = Column::<Curve25519Scalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::Int128(meta, &[1, 2, 3, 4, 5]));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Booleans
        let owned_col: OwnedColumn<Curve25519Scalar> =
            OwnedColumn::Boolean(meta, vec![true, false, true, false, true]);
        let col = Column::<Curve25519Scalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(
            col,
            Column::Boolean(meta, &[true, false, true, false, true])
        );
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
        let scalars = strs.iter().map(Curve25519Scalar::from).collect::<Vec<_>>();
        let owned_col = OwnedColumn::VarChar(
            meta,
            strs.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        let col = Column::<Curve25519Scalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::VarChar(meta, (&strs, &scalars)));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Decimals
        let scalars: Vec<Curve25519Scalar> =
            [1, 2, 3, 4, 5].iter().map(Curve25519Scalar::from).collect();
        let owned_col: OwnedColumn<Curve25519Scalar> =
            OwnedColumn::Decimal75(meta, Precision::new(75).unwrap(), 127, scalars.clone());
        let col = Column::<Curve25519Scalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(
            col,
            Column::Decimal75(meta, Precision::new(75).unwrap(), 127, &scalars)
        );
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);
    }

    #[test]
    fn we_can_get_the_data_size_of_a_column() {
        let meta = ColumnNullability::NotNullable;
        let column = Column::<DoryScalar>::Boolean(meta, &[true, false, true]);
        assert_eq!(column.column_type().byte_size(), 1);
        assert_eq!(column.column_type().bit_size(), 8);

        let column = Column::<Curve25519Scalar>::TinyInt(meta, &[1, 2, 3, 4]);
        assert_eq!(column.column_type().byte_size(), 1);
        assert_eq!(column.column_type().bit_size(), 8);

        let column = Column::<Curve25519Scalar>::SmallInt(meta, &[1, 2, 3, 4]);
        assert_eq!(column.column_type().byte_size(), 2);
        assert_eq!(column.column_type().bit_size(), 16);

        let column = Column::<Curve25519Scalar>::Int(meta, &[1, 2, 3]);
        assert_eq!(column.column_type().byte_size(), 4);
        assert_eq!(column.column_type().bit_size(), 32);

        let column = Column::<Curve25519Scalar>::BigInt(meta, &[1]);
        assert_eq!(column.column_type().byte_size(), 8);
        assert_eq!(column.column_type().bit_size(), 64);

        let column = Column::<DoryScalar>::Int128(meta, &[1, 2]);
        assert_eq!(column.column_type().byte_size(), 16);
        assert_eq!(column.column_type().bit_size(), 128);

        let scalar_values = [
            Curve25519Scalar::from(1),
            Curve25519Scalar::from(2),
            Curve25519Scalar::from(3),
        ];

        let column = Column::VarChar(meta, (&["a", "b", "c", "d", "e"], &scalar_values));
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let column = Column::Scalar(meta, &scalar_values);
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let precision = 10;
        let scale = 2;
        let decimal_data = [
            Curve25519Scalar::from(1),
            Curve25519Scalar::from(2),
            Curve25519Scalar::from(3),
        ];

        let precision = Precision::new(precision).unwrap();
        let column = Column::Decimal75(meta, precision, scale, &decimal_data);
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let column: Column<'_, DoryScalar> =
            Column::TimestampTZ(meta, PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]);
        assert_eq!(column.column_type().byte_size(), 8);
        assert_eq!(column.column_type().bit_size(), 64);
    }
}
