use super::TableRef;
use crate::base::scalar::ArkScalar;
use arrow::datatypes::{DataType, Field};
use proofs_sql::Identifier;
use serde::{Deserialize, Serialize};

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Column<'a> {
    /// i64 columns
    BigInt(&'a [i64]),
    /// Byte columns (such as &[&[u8]] or &[&str.as_bytes()])
    ///  - the first element maps to the byte values.
    ///  - the second element maps to the byte hashes (see [crate::base::scalar::ArkScalar]).
    HashedBytes((&'a [&'a [u8]], &'a [ArkScalar])),
    /// i128 columns
    Int128(&'a [i128]),
}

/// Provides the column type associated with the column
impl Column<'_> {
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::BigInt(_) => ColumnType::BigInt,
            Self::HashedBytes(_) => ColumnType::VarChar,
            Self::Int128(_) => ColumnType::Int128,
        }
    }
}

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
pub enum ColumnType {
    /// Mapped to i64
    #[serde(alias = "BIGINT", alias = "bigint")]
    BigInt,
    /// Mapped to i128
    #[serde(alias = "INT128", alias = "int128")]
    Int128,
    /// Mapped to String
    #[serde(alias = "VARCHAR", alias = "varchar")]
    VarChar,
}

/// Convert ColumnType values to some arrow DataType
impl From<&ColumnType> for DataType {
    fn from(column_type: &ColumnType) -> Self {
        match column_type {
            ColumnType::BigInt => DataType::Int64,
            ColumnType::Int128 => DataType::Decimal128(38, 0),
            ColumnType::VarChar => DataType::Utf8,
        }
    }
}

/// Convert arrow DataType values to some ColumnType
impl TryFrom<DataType> for ColumnType {
    type Error = String;

    fn try_from(data_type: DataType) -> Result<Self, Self::Error> {
        match data_type {
            DataType::Int64 => Ok(ColumnType::BigInt),
            DataType::Decimal128(38, 0) => Ok(ColumnType::Int128),
            DataType::Utf8 => Ok(ColumnType::VarChar),
            _ => Err(format!("Unsupported arrow data type {:?}", data_type)),
        }
    }
}

/// Display the column type as a str name (in all caps)
impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::BigInt => write!(f, "BIGINT"),
            ColumnType::Int128 => write!(f, "INT128"),
            ColumnType::VarChar => write!(f, "VARCHAR"),
        }
    }
}

/// Parse the column type from a str name (flexible about case)
impl std::str::FromStr for ColumnType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BIGINT" => Ok(ColumnType::BigInt),
            "INT128" => Ok(ColumnType::Int128),
            "VARCHAR" => Ok(ColumnType::VarChar),
            _ => Err(format!("Unsupported column type {:?}", s)),
        }
    }
}

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct ColumnRef {
    column_id: Identifier,
    table_ref: TableRef,
    column_type: ColumnType,
}

impl ColumnRef {
    pub fn new(table_ref: TableRef, column_id: Identifier, column_type: ColumnType) -> Self {
        Self {
            column_id,
            column_type,
            table_ref,
        }
    }

    pub fn table_ref(&self) -> TableRef {
        self.table_ref
    }

    pub fn column_id(&self) -> Identifier {
        self.column_id
    }

    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }
}

// Represents an abstraction for the arrow Field
//
// This allows us to work with the proof column
// native types, while also easily converting to
// arrow Field structures.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct ColumnField {
    name: Identifier,
    data_type: ColumnType,
}

impl ColumnField {
    pub fn new(name: Identifier, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
    }

    pub fn name(&self) -> Identifier {
        self.name
    }

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

    #[test]
    fn column_type_serializes_to_string() {
        let column_type = ColumnType::BigInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""BigInt""#);

        let column_type = ColumnType::Int128;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Int128""#);

        let column_type = ColumnType::VarChar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""VarChar""#);
    }

    #[test]
    fn we_can_deserialize_columns_from_valid_strings() {
        let expected_column_type = ColumnType::BigInt;
        let deserialized: ColumnType = serde_json::from_str(r#""BigInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int128;
        let deserialized: ColumnType = serde_json::from_str(r#""Int128""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::VarChar;
        let deserialized: ColumnType = serde_json::from_str(r#""VarChar""#).unwrap();
        assert_eq!(deserialized, expected_column_type);
    }

    #[test]
    fn we_can_deserialize_columns_from_lowercase_or_uppercase_strings() {
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""bigint""#).unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""BIGINT""#).unwrap(),
            ColumnType::BigInt
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""int128""#).unwrap(),
            ColumnType::Int128
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""INT128""#).unwrap(),
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
    }

    #[test]
    fn we_cannot_deserialize_columns_from_invalid_strings() {
        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Bigint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""InT128""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Varchar""#);
        assert!(deserialized.is_err());
    }

    #[test]
    fn we_can_convert_columntype_to_string_and_back_with_display_and_parse() {
        assert_eq!(format!("{}", ColumnType::BigInt), "BIGINT");
        assert_eq!(format!("{}", ColumnType::Int128), "INT128");
        assert_eq!(format!("{}", ColumnType::VarChar), "VARCHAR");
        assert_eq!("BIGINT".parse::<ColumnType>().unwrap(), ColumnType::BigInt);
        assert_eq!("INT128".parse::<ColumnType>().unwrap(), ColumnType::Int128);
        assert_eq!(
            "VARCHAR".parse::<ColumnType>().unwrap(),
            ColumnType::VarChar
        );
    }
}
