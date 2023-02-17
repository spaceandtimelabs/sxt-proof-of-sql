use super::TableRef;
use arrow::datatypes::DataType;
use arrow::datatypes::Field;
use curve25519_dalek::scalar::Scalar;
use proofs_sql::Identifier;
use serde::{Deserialize, Serialize};

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq)]
pub enum Column<'a> {
    /// i64 columns
    BigInt(&'a [i64]),
    /// Byte columns (such as &[&[u8]] or &[&str.as_bytes()])
    ///  - the first element maps to the byte values.
    ///  - the second element maps to the byte hashes (see [crate::base::scalar::ToScalar] trait).
    HashedBytes((&'a [&'a [u8]], &'a [Scalar])),
}

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
pub enum ColumnType {
    /// Mapped to i64
    BigInt,
    /// Mapped to String
    VarChar,
}

/// Convert ColumnType values to some arrow DataType
impl From<&ColumnType> for DataType {
    fn from(column_type: &ColumnType) -> Self {
        match column_type {
            ColumnType::BigInt => DataType::Int64,
            ColumnType::VarChar => DataType::Utf8,
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
            column_field.name.name(),
            (&column_field.data_type).into(),
            false,
        )
    }
}
