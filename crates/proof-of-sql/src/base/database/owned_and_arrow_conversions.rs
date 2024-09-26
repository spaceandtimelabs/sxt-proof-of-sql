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
    database::{
        scalar_and_i256_conversions::convert_i256_to_scalar, OwnedColumn, OwnedTable,
        OwnedTableError,
    },
    map::IndexMap,
    math::decimal::Precision,
    scalar::Scalar,
};
use arrow::{
    array::{
        ArrayRef, BooleanArray, Decimal128Array, Decimal256Array, Int16Array, Int32Array,
        Int64Array, StringArray, TimestampMicrosecondArray, TimestampMillisecondArray,
        TimestampNanosecondArray, TimestampSecondArray,
    },
    datatypes::{i256, DataType, Schema, SchemaRef, TimeUnit as ArrowTimeUnit},
    error::ArrowError,
    record_batch::RecordBatch,
};
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestampError},
    Identifier, ParseError,
};
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
    /// Using TimeError to handle all time-related errors
    #[error(transparent)]
    TimestampConversionError(#[from] PoSQLTimestampError),
}

impl<S: Scalar> From<OwnedColumn<S>> for ArrayRef {
    fn from(value: OwnedColumn<S>) -> Self {
        match value {
            OwnedColumn::Boolean(col) => Arc::new(BooleanArray::from(col)),
            OwnedColumn::SmallInt(col) => Arc::new(Int16Array::from(col)),
            OwnedColumn::Int(col) => Arc::new(Int32Array::from(col)),
            OwnedColumn::BigInt(col) => Arc::new(Int64Array::from(col)),
            OwnedColumn::Int128(col) => Arc::new(
                //TODO: add panic docs
                Decimal128Array::from(col)
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            OwnedColumn::Decimal75(precision, scale, col) => {
                let converted_col: Vec<i256> = col.iter().map(convert_scalar_to_i256).collect();

                Arc::new(
                    //TODO: add oanic docs
                    Decimal256Array::from(converted_col)
                        .with_precision_and_scale(precision.value(), scale)
                        .unwrap(),
                )
            }
            OwnedColumn::Scalar(_) => unimplemented!("Cannot convert Scalar type to arrow type"),
            OwnedColumn::VarChar(col) => Arc::new(StringArray::from(col)),
            OwnedColumn::TimestampTZ(time_unit, _, col) => match time_unit {
                PoSQLTimeUnit::Second => Arc::new(TimestampSecondArray::from(col)),
                PoSQLTimeUnit::Millisecond => Arc::new(TimestampMillisecondArray::from(col)),
                PoSQLTimeUnit::Microsecond => Arc::new(TimestampMicrosecondArray::from(col)),
                PoSQLTimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::from(col)),
            },
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
                //TODO: add panic docs
                value
                    .as_any()
                    .downcast_ref::<BooleanArray>()
                    .unwrap()
                    .iter()
                    .collect::<Option<Vec<bool>>>()
                    .ok_or(OwnedArrowConversionError::NullNotSupportedYet)?,
            )),
            DataType::Int16 => Ok(Self::SmallInt(
                //TODO: add panic docs
                value
                    .as_any()
                    .downcast_ref::<Int16Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Int32 => Ok(Self::Int(
                //TODO: add panic docs
                value
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Int64 => Ok(Self::BigInt(
                //TODO: add panic docs
                value
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Decimal128(38, 0) => Ok(Self::Int128(
                //TODO: add panic docs
                value
                    .as_any()
                    .downcast_ref::<Decimal128Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Decimal256(precision, scale) if *precision <= 75 => Ok(Self::Decimal75(
                Precision::new(*precision).expect("precision is less than 76"),
                *scale,
                //TODO: add panic docs
                value
                    .as_any()
                    .downcast_ref::<Decimal256Array>()
                    .unwrap()
                    .values()
                    .iter()
                    .map(convert_i256_to_scalar)
                    .map(Option::unwrap)
                    .collect(),
            )),
            DataType::Utf8 => Ok(Self::VarChar(
                //TODO: add panic docs
                value
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .iter()
                    .map(|s| s.unwrap().to_string())
                    .collect(),
            )),
            DataType::Timestamp(time_unit, timezone) => match time_unit {
                ArrowTimeUnit::Second => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampSecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let timestamps = array.values().iter().copied().collect::<Vec<i64>>();
                    Ok(OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Second,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    ))
                }
                ArrowTimeUnit::Millisecond => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampMillisecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let timestamps = array.values().iter().copied().collect::<Vec<i64>>();
                    Ok(OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Millisecond,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    ))
                }
                ArrowTimeUnit::Microsecond => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampMicrosecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let timestamps = array.values().iter().copied().collect::<Vec<i64>>();
                    Ok(OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Microsecond,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    ))
                }
                ArrowTimeUnit::Nanosecond => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampNanosecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let timestamps = array.values().iter().copied().collect::<Vec<i64>>();
                    Ok(OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Nanosecond,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    ))
                }
            },
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
