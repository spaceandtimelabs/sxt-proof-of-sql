//! This module provide `From` and `TryFrom` implementations to go between arrow and owned types
//! The mapping is as follows:
//! OwnedType <-> Array/ArrayRef
//! OwnedTable <-> RecordBatch
//! Boolean <-> Boolean
//! BigInt <-> Int64
//! VarChar <-> Utf8/String
//! Int128 <-> Decimal128(38,0)
//! Decimal75 <-> S
//!
//! Note: this converts `Int128` values to `Decimal128(38,0)`, which are backed by `i128`.
//! This is because there is no `Int128` type in Arrow.
//! This does not check that the values are less than 39 digits.
//! However, the actual arrow backing `i128` is the correct value.
use super::scalar_and_i256_conversions::convert_scalar_to_i256;
use crate::base::{
    database::{OwnedColumn, OwnedTable, OwnedTableError},
    scalar::Scalar,
};
use arrow::{
    array::{ArrayRef, BooleanArray, Decimal128Array, Decimal256Array, Int64Array, StringArray},
    datatypes::{i256, DataType, Schema, SchemaRef},
    error::ArrowError,
    record_batch::RecordBatch,
};
use indexmap::IndexMap;
use proofs_sql::{Identifier, ParseError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
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
    /// This error occurs when trying to convert from an Arrow array with nulls.
    #[error("null values are not supported in OwnedColumn yet")]
    NullNotSupportedYet,
}

impl<S: Scalar> From<OwnedColumn<S>> for ArrayRef {
    fn from(value: OwnedColumn<S>) -> Self {
        match value {
            OwnedColumn::Boolean(col) => Arc::new(BooleanArray::from(col)),
            OwnedColumn::BigInt(col) => Arc::new(Int64Array::from(col)),
            OwnedColumn::VarChar(col) => Arc::new(StringArray::from(col)),
            OwnedColumn::Int128(col) => Arc::new(
                Decimal128Array::from(col)
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            OwnedColumn::Decimal75(precision, scale, col) => {
                let converted_col: Vec<i256> = col.iter().map(convert_scalar_to_i256).collect();

                Arc::new(
                    Decimal256Array::from(converted_col)
                        .with_precision_and_scale(precision.value(), scale)
                        .unwrap(),
                )
            }
            OwnedColumn::Scalar(_) => unimplemented!("Cannot convert Scalar type to arrow type"),
        }
    }
}

impl<S: Scalar> TryFrom<OwnedTable<S>> for RecordBatch {
    type Error = ArrowError;
    fn try_from(value: OwnedTable<S>) -> Result<Self, Self::Error> {
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

impl<S: Scalar> TryFrom<ArrayRef> for OwnedColumn<S> {
    type Error = OwnedArrowConversionError;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
impl<S: Scalar> TryFrom<&ArrayRef> for OwnedColumn<S> {
    type Error = OwnedArrowConversionError;
    fn try_from(value: &ArrayRef) -> Result<Self, Self::Error> {
        match &value.data_type() {
            // Arrow uses a bit-packed representation for booleans.
            // Hence we need to unpack the bits to get the actual boolean values.
            DataType::Boolean => Ok(Self::Boolean(
                value
                    .as_any()
                    .downcast_ref::<BooleanArray>()
                    .unwrap()
                    .iter()
                    .collect::<Option<Vec<bool>>>()
                    .ok_or(OwnedArrowConversionError::NullNotSupportedYet)?,
            )),
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

impl<S: Scalar> TryFrom<RecordBatch> for OwnedTable<S> {
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
