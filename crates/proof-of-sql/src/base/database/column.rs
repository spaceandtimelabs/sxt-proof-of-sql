use super::{LiteralValue, TableRef};
use crate::base::{
    math::decimal::{scale_scalar, Precision},
    scalar::Scalar,
    time::timestamp::{ProofsTimeUnit, ProofsTimeZone},
};
use arrow::datatypes::{DataType, Field, TimeUnit as ArrowTimeUnit};
use bumpalo::Bump;
use proof_of_sql_parser::Identifier;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone)]
#[non_exhaustive]
pub enum Column<'a, S: Scalar> {
    /// Boolean columns
    Boolean(&'a [bool]),
    /// i16 columns
    SmallInt(&'a [i16]),
    /// i32 columns
    Int(&'a [i32]),
    /// i64 columns
    BigInt(&'a [i64]),
    /// i128 columns
    Int128(&'a [i128]),
    /// Decimal columns with a max width of 252 bits
    ///  - the backing store maps to the type [crate::base::scalar::Curve25519Scalar]
    Decimal75(Precision, i8, &'a [S]),
    /// Scalar columns
    Scalar(&'a [S]),
    /// String columns
    ///  - the first element maps to the str values.
    ///  - the second element maps to the str hashes (see [crate::base::scalar::Scalar]).
    VarChar((&'a [&'a str], &'a [S])),
    /// Timestamp columns
    /// - the first element maps to the stored [`TimeUnit`]
    /// - the second element maps to an optional timezone as a string
    /// - the third element maps to columns of timeunits since unix epoch
    Timestamp(ProofsTimeUnit, ProofsTimeZone, &'a [i64]),
}

impl<'a, S: Scalar> Column<'a, S> {
    /// Provides the column type associated with the column
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Boolean(_) => ColumnType::Boolean,
            Self::SmallInt(_) => ColumnType::SmallInt,
            Self::Int(_) => ColumnType::Int,
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::Int128(_) => ColumnType::Int128,
            Self::Scalar(_) => ColumnType::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnType::Decimal75(*precision, *scale),
            Self::Timestamp(time_unit, timezone, _) => ColumnType::Timestamp(*time_unit, *timezone),
        }
    }
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            Self::Boolean(col) => col.len(),
            Self::SmallInt(col) => col.len(),
            Self::Int(col) => col.len(),
            Self::BigInt(col) => col.len(),
            Self::VarChar((col, scals)) => {
                assert_eq!(col.len(), scals.len());
                col.len()
            }
            Self::Int128(col) => col.len(),
            Self::Scalar(col) => col.len(),
            Self::Decimal75(_, _, col) => col.len(),
            Self::Timestamp(_, _, col) => col.len(),
        }
    }
    /// Returns `true` if the column has no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Generate a constant column from a literal value with a given length
    pub fn from_literal_with_length(
        literal: &LiteralValue<S>,
        length: usize,
        alloc: &'a Bump,
    ) -> Self {
        match literal {
            LiteralValue::Boolean(value) => {
                Column::Boolean(alloc.alloc_slice_fill_copy(length, *value))
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
                Column::Scalar(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Decimal75(precision, scale, value) => Column::Decimal75(
                *precision,
                *scale,
                alloc.alloc_slice_fill_copy(length, *value),
            ),
            LiteralValue::VarChar((string, scalar)) => Column::VarChar((
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

    /// Convert a column to a vector of Scalar values with scaling
    pub(crate) fn to_scalar_with_scaling(&self, scale: i8) -> Vec<S> {
        let scale_factor = scale_scalar(S::ONE, scale).expect("Invalid scale factor");
        match self {
            Self::Boolean(col) => col
                .par_iter()
                .map(|b| S::from(b) * scale_factor)
                .collect::<Vec<_>>(),
            Self::Decimal75(_, _, col) => col
                .par_iter()
                .map(|s| *s * scale_factor)
                .collect::<Vec<_>>(),
            Self::VarChar((_, scals)) => scals
                .par_iter()
                .map(|s| *s * scale_factor)
                .collect::<Vec<_>>(),

            Self::SmallInt(col) => col
                .par_iter()
                .map(|i| S::from(i) * scale_factor)
                .collect::<Vec<_>>(),
            Self::Int(col) => col
                .par_iter()
                .map(|i| S::from(i) * scale_factor)
                .collect::<Vec<_>>(),
            Self::BigInt(col) => col
                .par_iter()
                .map(|i| S::from(i) * scale_factor)
                .collect::<Vec<_>>(),
            Self::Int128(col) => col
                .par_iter()
                .map(|i| S::from(i) * scale_factor)
                .collect::<Vec<_>>(),
            Self::Scalar(col) => col
                .par_iter()
                .map(|s| *s * scale_factor)
                .collect::<Vec<_>>(),
            Self::Timestamp(_, _, col) => col
                .par_iter()
                .map(|i| S::from(i) * scale_factor)
                .collect::<Vec<_>>(),
        }
    }
}

/// The precision for [ColumnType::INT128] values
pub const INT128_PRECISION: usize = 38;

/// The scale for [ColumnType::INT128] values
pub const INT128_SCALE: usize = 0;

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
    /// Mapped to Curve25519Scalar
    #[serde(alias = "SCALAR", alias = "scalar")]
    Scalar,
    /// Mapped to i256
    #[serde(rename = "Decimal75", alias = "DECIMAL75", alias = "decimal75")]
    Decimal75(Precision, i8),
    /// Mapped to i64
    #[serde(alias = "TIMESTAMP", alias = "timestamp")]
    Timestamp(ProofsTimeUnit, ProofsTimeZone),
}

impl ColumnType {
    /// Returns true if this column is numeric and false otherwise
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ColumnType::SmallInt
                | ColumnType::Int
                | ColumnType::BigInt
                | ColumnType::Int128
                | ColumnType::Scalar
                | ColumnType::Decimal75(_, _) // TODO: is a timestamp numeric?
        )
    }

    /// Returns true if this column is an integer and false otherwise
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            ColumnType::SmallInt | ColumnType::Int | ColumnType::BigInt | ColumnType::Int128 // TODO: is a timestamp an integer?
        )
    }

    /// Returns the precision of a ColumnType if it is converted to a decimal wrapped in Some(). If it can not be converted to a decimal, return None.
    pub fn precision_value(&self) -> Option<u8> {
        match self {
            Self::SmallInt => Some(5_u8),
            Self::Int => Some(10_u8),
            Self::BigInt => Some(19_u8),
            Self::Timestamp(_, _) => Some(19_u8),
            Self::Int128 => Some(39_u8),
            Self::Decimal75(precision, _) => Some(precision.value()),
            // Scalars are not in database & are only used for typeless comparisons for testing so we return 0
            // so that they do not cause errors when used in comparisons.
            Self::Scalar => Some(0_u8),
            _ => None,
        }
    }
    /// Returns scale of a ColumnType if it is convertible to a decimal wrapped in Some(). Otherwise return None.
    pub fn scale(&self) -> Option<i8> {
        match self {
            Self::Decimal75(_, scale) => Some(*scale),
            Self::BigInt | Self::Int128 | Self::Scalar => Some(0),
            _ => None,
        }
    }
}

/// Convert ColumnType values to some arrow DataType
impl From<&ColumnType> for DataType {
    fn from(column_type: &ColumnType) -> Self {
        match column_type {
            ColumnType::Boolean => DataType::Boolean,
            ColumnType::SmallInt => DataType::Int16,
            ColumnType::Int => DataType::Int32,
            ColumnType::BigInt => DataType::Int64,
            ColumnType::Int128 => DataType::Decimal128(38, 0),
            ColumnType::Decimal75(precision, scale) => {
                DataType::Decimal256(precision.value(), *scale)
            }
            ColumnType::VarChar => DataType::Utf8,
            ColumnType::Scalar => unimplemented!("Cannot convert Scalar type to arrow type"),
            ColumnType::Timestamp(timeunit, timezone) => {
                DataType::Timestamp(ArrowTimeUnit::from(*timeunit), Some(Arc::from(timezone)))
            }
        }
    }
}

/// Convert arrow DataType values to some ColumnType
impl TryFrom<DataType> for ColumnType {
    type Error = String;

    fn try_from(data_type: DataType) -> Result<Self, Self::Error> {
        match data_type {
            DataType::Boolean => Ok(ColumnType::Boolean),
            DataType::Int16 => Ok(ColumnType::SmallInt),
            DataType::Int32 => Ok(ColumnType::Int),
            DataType::Int64 => Ok(ColumnType::BigInt),
            DataType::Decimal128(38, 0) => Ok(ColumnType::Int128),
            DataType::Decimal256(precision, scale) if precision <= 75 => {
                Ok(ColumnType::Decimal75(Precision::new(precision)?, scale))
            }
            DataType::Timestamp(time_unit, timezone_option) => {
                let custom_time_unit = ProofsTimeUnit::from(time_unit);

                let timezone = match timezone_option {
                    Some(tz_arc) => {
                        let tz_str = &*tz_arc; // Deref Arc<str> to &str
                        chrono_tz::Tz::from_str(tz_str)
                            .map_err(|_| format!("Invalid timezone string: {}", tz_str))?
                    }
                    None => chrono_tz::Tz::UTC, // Default to UTC if None
                };

                Ok(ColumnType::Timestamp(
                    custom_time_unit,
                    ProofsTimeZone(timezone),
                ))
            }
            DataType::Utf8 => Ok(ColumnType::VarChar),
            _ => Err(format!("Unsupported arrow data type {:?}", data_type)),
        }
    }
}

/// Display the column type as a str name (in all caps)
impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Boolean => write!(f, "BOOLEAN"),
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
            ColumnType::Timestamp(timeunit, timezone) => write!(
                f,
                "TIMESTAMP(TIMEUNIT: {:?}, TIMEZONE: {timeunit})",
                timezone
            ),
        }
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
    pub fn new(table_ref: TableRef, column_id: Identifier, column_type: ColumnType) -> Self {
        Self {
            column_id,
            column_type,
            table_ref,
        }
    }

    /// Returns the table reference of this column
    pub fn table_ref(&self) -> TableRef {
        self.table_ref
    }

    /// Returns the column identifier of this column
    pub fn column_id(&self) -> Identifier {
        self.column_id
    }

    /// Returns the column type of this column
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
    pub fn new(name: Identifier, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
    }

    /// Returns the name of the column
    pub fn name(&self) -> Identifier {
        self.name
    }

    /// Returns the type of the column
    pub fn data_type(&self) -> ColumnType {
        self.data_type
    }
}

/// Convert ColumnField values to arrow Field
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

    #[test]
    fn column_type_serializes_to_string() {
        let column_type = ColumnType::Boolean;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Boolean""#);

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
        let expected_column_type = ColumnType::Boolean;
        let deserialized: ColumnType = serde_json::from_str(r#""Boolean""#).unwrap();
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

        let scals = [
            Curve25519Scalar::from(1),
            Curve25519Scalar::from(2),
            Curve25519Scalar::from(3),
        ];

        // Test non-empty columns
        let column = Column::<DoryScalar>::Boolean(&[true, false, true]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<Curve25519Scalar>::SmallInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<Curve25519Scalar>::Int(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<Curve25519Scalar>::BigInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::VarChar((&["a", "b", "c"], &scals));
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<DoryScalar>::Int128(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::Scalar(&scals);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let decimal_data = [
            Curve25519Scalar::from(1),
            Curve25519Scalar::from(2),
            Curve25519Scalar::from(3),
        ];

        let precision = Precision::new(precision).unwrap();
        let column = Column::Decimal75(precision, scale, &decimal_data);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        // Test empty columns
        let column = Column::<DoryScalar>::Boolean(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::SmallInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::Int(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::BigInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::VarChar((&[], &[]));
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<Curve25519Scalar>::Int128(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::Scalar(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column: Column<'_, Curve25519Scalar> = Column::Decimal75(precision, scale, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());
    }
}
