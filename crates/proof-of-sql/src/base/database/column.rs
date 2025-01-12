use super::{OwnedColumn, TableRef};
use crate::{
    alloc::string::ToString,
    base::{
        math::{decimal::Precision, i256::I256},
        scalar::{Scalar, ScalarExt},
        slice_ops::slice_cast_with,
    },
    sql::parse::ConversionError,
};
use alloc::{format, string::String, vec::Vec};
use bumpalo::Bump;
use core::{
    fmt,
    fmt::{Display, Formatter},
    mem::size_of,
};
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use sqlparser::ast::{DataType, ExactNumberInfo, Expr as SqlExpr, Ident, TimezoneInfo, Value};

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
    TimestampTZ(PoSQLTimeUnit, TimezoneInfo, &'a [i64]),
}

impl<'a, S: Scalar> Column<'a, S> {
    /// Provides the column type associated with the column
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Boolean(_) => ColumnType::Boolean,
            Self::TinyInt(_) => ColumnType::TinyInt,
            Self::SmallInt(_) => ColumnType::SmallInt,
            Self::Int(_) => ColumnType::Int,
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::Int128(_) => ColumnType::Int128,
            Self::Scalar(_) => ColumnType::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnType::Decimal75(*precision, *scale),
            Self::TimestampTZ(time_unit, timezone, _) => {
                ColumnType::TimestampTZ(*time_unit, *timezone)
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
    /// Returns `true` if the column has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Generate a constant column from a literal value with a given length
    ///
    /// # Panics
    /// - Panics if the precision or scale for a decimal value cannot fit into `u8` or `i8`, respectively.
    /// - Panics if creating a `Precision` object fails.
    pub fn from_literal_with_length(
        expr: &SqlExpr,
        length: usize,
        alloc: &'a Bump,
    ) -> Result<Self, ColumnError> {
        match expr {
            // Boolean value
            SqlExpr::Value(Value::Boolean(value)) => {
                Ok(Column::Boolean(alloc.alloc_slice_fill_copy(length, *value)))
            }

            // Numeric values
            SqlExpr::Value(Value::Number(value, _)) => {
                let n = value
                    .parse::<i64>()
                    .map_err(|_| ColumnError::InvalidNumberFormat {
                        value: value.clone(),
                    })?;

                if let Ok(n_i8) = i8::try_from(n) {
                    Ok(Column::TinyInt(alloc.alloc_slice_fill_copy(length, n_i8)))
                } else if let Ok(n_i16) = i16::try_from(n) {
                    Ok(Column::SmallInt(alloc.alloc_slice_fill_copy(length, n_i16)))
                } else if let Ok(n_i32) = i32::try_from(n) {
                    Ok(Column::Int(alloc.alloc_slice_fill_copy(length, n_i32)))
                } else {
                    Ok(Column::BigInt(alloc.alloc_slice_fill_copy(length, n)))
                }
            }

            // String values
            SqlExpr::Value(Value::SingleQuotedString(string)) => Ok(Column::VarChar((
                alloc.alloc_slice_fill_with(length, |_| alloc.alloc_str(string) as &str),
                alloc.alloc_slice_fill_copy(length, S::from(string)),
            ))),

            // Typed string literals
            SqlExpr::TypedString { data_type, value } => match data_type {
                // Decimal values
                DataType::Decimal(ExactNumberInfo::PrecisionAndScale(precision, scale)) => {
                    let i256_value =
                        I256::from_string(value).map_err(|_| ColumnError::InvalidDecimal {
                            value: value.clone(),
                        })?;
                    let precision_u8 =
                        u8::try_from(*precision).expect("Precision must fit into u8");
                    let scale_i8 = i8::try_from(*scale).expect("Scale must fit into i8");
                    let precision_obj =
                        Precision::new(precision_u8).expect("Failed to create Precision");

                    Ok(Column::Decimal75(
                        precision_obj,
                        scale_i8,
                        alloc.alloc_slice_fill_copy(length, i256_value.into_scalar()),
                    ))
                }

                // Timestamp values
                DataType::Timestamp(Some(precision), tz) => {
                    let time_unit =
                        PoSQLTimeUnit::from_precision(*precision).unwrap_or(PoSQLTimeUnit::Second);
                    let timestamp_value =
                        value
                            .parse::<i64>()
                            .map_err(|_| ColumnError::InvalidNumberFormat {
                                value: value.clone(),
                            })?;
                    Ok(Column::TimestampTZ(
                        time_unit,
                        *tz,
                        alloc.alloc_slice_fill_copy(length, timestamp_value),
                    ))
                }
                DataType::Custom(_, _) if data_type.to_string() == "scalar" => {
                    let scalar_str = value.strip_prefix("scalar:").ok_or_else(|| {
                        ColumnError::InvalidScalarFormat {
                            value: value.clone(),
                        }
                    })?;
                    let limbs: Vec<u64> = scalar_str
                        .split(',')
                        .map(|x| {
                            x.parse::<u64>()
                                .map_err(|_| ColumnError::InvalidScalarFormat {
                                    value: value.clone(),
                                })
                        })
                        .collect::<Result<Vec<u64>, ColumnError>>()?;
                    if limbs.len() != 4 {
                        return Err(ColumnError::InvalidScalarFormat {
                            value: value.clone(),
                        });
                    }
                    Ok(Column::Scalar(
                        alloc.alloc_slice_fill_copy(length, value.clone().into()),
                    ))
                }
                _ => Err(ColumnError::UnsupportedDataType {
                    data_type: format!("{expr:?}"),
                }),
            },
            _ => Err(ColumnError::UnsupportedDataType {
                data_type: format!("{expr:?}"),
            }),
        }
    }

    /// Convert an `OwnedColumn` to a `Column`
    pub fn from_owned_column(owned_column: &'a OwnedColumn<S>, alloc: &'a Bump) -> Self {
        match owned_column {
            OwnedColumn::Boolean(col) => Column::Boolean(col.as_slice()),
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
        let scale_factor = S::pow10(u8::try_from(scale).expect("Upscale factor is nonnegative"));
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

/// Represents errors that can occur while working with columns.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ColumnError {
    #[snafu(display("Invalid number format: {value}"))]
    /// Error for invalid number format.
    InvalidNumberFormat {
        /// The invalid number value that caused the error.
        value: String,
    },

    #[snafu(display("Unsupported data type: {data_type}"))]
    /// Error for unsupported data types.
    UnsupportedDataType {
        /// The unsupported data type as a string.
        data_type: String,
    },

    #[snafu(display("Scalar parsing error: {value}"))]
    /// Error for scalar parsing failures.
    InvalidScalarFormat {
        /// The scalar value that caused the parsing error.
        value: String,
    },

    #[snafu(display("Invalid decimal format: {value}"))]
    /// Error for invalid decimal format.
    InvalidDecimal {
        /// The invalid decimal value that caused the error.
        value: String,
    },

    #[snafu(display("Conversion error: {source}"))]
    /// Error for column operation conversion issues.
    ConversionError {
        /// The underlying conversion error that occurred.
        source: ConversionError,
    },
}

impl From<ConversionError> for ColumnError {
    fn from(error: ConversionError) -> Self {
        ColumnError::ConversionError { source: error }
    }
}

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
pub enum ColumnType {
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
    TimestampTZ(PoSQLTimeUnit, TimezoneInfo),
    /// Mapped to `S`
    #[serde(alias = "SCALAR", alias = "scalar")]
    Scalar,
}

impl ColumnType {
    /// Returns true if this column is numeric and false otherwise
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ColumnType::TinyInt
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
            ColumnType::TinyInt
                | ColumnType::SmallInt
                | ColumnType::Int
                | ColumnType::BigInt
                | ColumnType::Int128
        )
    }

    /// Returns the number of bits in the integer type if it is an integer type. Otherwise, return None.
    fn to_integer_bits(self) -> Option<usize> {
        match self {
            ColumnType::TinyInt => Some(8),
            ColumnType::SmallInt => Some(16),
            ColumnType::Int => Some(32),
            ColumnType::BigInt => Some(64),
            ColumnType::Int128 => Some(128),
            _ => None,
        }
    }

    /// Returns the [`ColumnType`] of the integer type with the given number of bits if it is a valid integer type.
    ///
    /// Otherwise, return None.
    fn from_integer_bits(bits: usize) -> Option<Self> {
        match bits {
            8 => Some(ColumnType::TinyInt),
            16 => Some(ColumnType::SmallInt),
            32 => Some(ColumnType::Int),
            64 => Some(ColumnType::BigInt),
            128 => Some(ColumnType::Int128),
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
                .and_then(|other_bits| Self::from_integer_bits(self_bits.max(other_bits)))
        })
    }

    /// Returns the precision of a [`ColumnType`] if it is converted to a decimal wrapped in `Some()`. If it can not be converted to a decimal, return None.
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
    /// Returns scale of a [`ColumnType`] if it is convertible to a decimal wrapped in `Some()`. Otherwise return None.
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
            Self::Decimal75(_, _) | Self::Scalar | Self::VarChar | Self::Boolean => false,
        }
    }
}

/// Display the column type as a str name (in all caps)
impl Display for ColumnType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Boolean => write!(f, "BOOLEAN"),
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
        self.table_ref
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
        let column_type = ColumnType::TimestampTZ(PoSQLTimeUnit::Second, TimezoneInfo::None);
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"TimestampTZ":["Second","None"]}"#);

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
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, TimezoneInfo::None);
        let deserialized: ColumnType =
            serde_json::from_str(r#"{"TimestampTZ":["Second","None"]}"#).unwrap();
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

        let column = Column::Scalar(&scalar_values);
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

        let column = Column::Scalar(&scalar_values);
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
            Column::TimestampTZ(PoSQLTimeUnit::Second, TimezoneInfo::None, &[1, 2, 3]);
        assert_eq!(column.column_type().byte_size(), 8);
        assert_eq!(column.column_type().bit_size(), 64);
    }
}
