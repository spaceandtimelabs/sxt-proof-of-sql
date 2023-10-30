//! This module provide `From` and `TryFrom` implementations to go between arrow and owned types
//! The mapping is as follows:
//! OwnedType <-> Array/ArrayRef
//! OwnedTable <-> RecordBatch
//! BigInt <-> Int64
//! VarChar <-> Utf8/String
//! Int128 <-> Decimal128(38,0)
//!
//! Note: this converts `Int128` values to `Decimal128(38,0)`, which are backed by `i128`.
//! This is because there is no `Int128` type in Arrow.
//! This does not check that the values are less than 39 digits.
//! However, the actual arrow backing `i128` is the correct value.

use crate::base::database::{OwnedColumn, OwnedTable, OwnedTableError};
use arrow::{
    array::{ArrayRef, Decimal128Array, Int64Array, StringArray},
    datatypes::{DataType, Schema, SchemaRef},
    error::ArrowError,
    record_batch::RecordBatch,
};
use indexmap::IndexMap;
use proofs_sql::{Identifier, ParseError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors cause by conversions between Arrow and owned types.
pub enum OwnedArrowConversionError {
    /// This error occurs when trying to convert from an unsupported arrow type.
    #[error("unsupported type: attempted conversion from ArrayRef of type {0} to OwnedColumn")]
    UnsupportedType(DataType),
    /// This error occurs when trying to convert from a record batch with duplicate identifiers (e.g. `"a"` and `"A"`).
    #[error("conversion resulted in duplicate identifiers")]
    DuplicateIdentifiers,
    #[error(transparent)]
    /// This error occurs when convering from a record batch name to an identifier fails. (Which may my impossible.)
    FieldParseFail(#[from] ParseError),
    #[error(transparent)]
    /// This error occurs when creating an owned table fails, which should only occur when there are zero columns.
    InvalidTable(#[from] OwnedTableError),
}

impl From<OwnedColumn> for ArrayRef {
    fn from(value: OwnedColumn) -> Self {
        match value {
            OwnedColumn::BigInt(col) => Arc::new(Int64Array::from(col)),
            OwnedColumn::VarChar(col) => Arc::new(StringArray::from(col)),
            OwnedColumn::Int128(col) => Arc::new(
                Decimal128Array::from(col)
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
        }
    }
}

impl TryFrom<OwnedTable> for RecordBatch {
    type Error = ArrowError;
    fn try_from(value: OwnedTable) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(RecordBatch::new_empty(SchemaRef::new(Schema::empty())))
        } else {
            RecordBatch::try_from_iter(
                value
                    .into_inner()
                    .into_iter()
                    .map(|(identifier, owned_column)| (identifier, ArrayRef::from(owned_column))),
            )
        }
    }
}

impl TryFrom<ArrayRef> for OwnedColumn {
    type Error = OwnedArrowConversionError;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
impl TryFrom<&ArrayRef> for OwnedColumn {
    type Error = OwnedArrowConversionError;
    fn try_from(value: &ArrayRef) -> Result<Self, Self::Error> {
        match &value.data_type() {
            DataType::Int64 => Ok(Self::BigInt(
                value
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Decimal128(38, 0) => Ok(Self::Int128(
                value
                    .as_any()
                    .downcast_ref::<Decimal128Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Utf8 => Ok(Self::VarChar(
                value
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .iter()
                    .map(|s| s.unwrap().to_string())
                    .collect(),
            )),
            &data_type => Err(OwnedArrowConversionError::UnsupportedType(
                data_type.clone(),
            )),
        }
    }
}

impl TryFrom<RecordBatch> for OwnedTable {
    type Error = OwnedArrowConversionError;
    fn try_from(value: RecordBatch) -> Result<Self, Self::Error> {
        let num_columns = value.num_columns();
        let table: Result<IndexMap<_, _>, Self::Error> = value
            .schema()
            .fields()
            .iter()
            .zip(value.columns())
            .map(|(field, array_ref)| {
                let owned_column = OwnedColumn::try_from(array_ref)?;
                let identifier = Identifier::try_new(field.name())?; //This may always succeed.
                Ok((identifier, owned_column))
            })
            .collect();
        let owned_table = Self::try_new(table?)?;
        if num_columns == owned_table.num_columns() {
            Ok(owned_table)
        } else {
            Err(OwnedArrowConversionError::DuplicateIdentifiers)
        }
    }
}
