//! This module provides `From` and `TryFrom` implementations to go between arrow and owned types
//! The mapping is as follows:
//! `OwnedType` <-> `Array/ArrayRef`
//! `OwnedTable` <-> `RecordBatch`
//! `Boolean` <-> `Boolean`
//! `BigInt` <-> `Int64`
//! `VarChar` <-> `Utf8/String`
//! `Int128` <-> `Decimal128(38,0)`
//! `Decimal75` <-> `S`
//!
//! Note: this converts `Int128` values to `Decimal128(38,0)`, which are backed by `i128`.
//! This is because there is no `Int128` type in Arrow.
//! This does not check that the values are less than 39 digits.
//! However, the actual arrow backing `i128` is the correct value.
use super::scalar_and_i256_conversions::{convert_i256_to_scalar, convert_scalar_to_i256};
use crate::base::{
    database::{OwnedColumn, OwnedNullableColumn, OwnedTable, OwnedTableError, TableError},
    map::IndexMap,
    math::decimal::{DecimalError, Precision},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestampError},
    scalar::Scalar,
};
use alloc::sync::Arc;
use arrow::{
    array::{
        Array, ArrayRef, BinaryArray, BooleanArray, Decimal128Array, Decimal256Array, Int16Array,
        Int32Array, Int64Array, Int8Array, StringArray, TimestampMicrosecondArray,
        TimestampMillisecondArray, TimestampNanosecondArray, TimestampSecondArray, UInt8Array,
    },
    buffer::NullBuffer,
    datatypes::{i256, DataType, Schema, SchemaRef, TimeUnit as ArrowTimeUnit},
    error::ArrowError,
    record_batch::RecordBatch,
};
use snafu::Snafu;
use sqlparser::ast::Ident;

#[derive(Snafu, Debug)]
#[non_exhaustive]
/// Errors caused by conversions between Arrow and owned types.
pub enum OwnedArrowConversionError {
    /// This error occurs when trying to convert from an unsupported arrow type.
    #[snafu(display(
        "unsupported type: attempted conversion from ArrayRef of type {datatype} to OwnedColumn"
    ))]
    UnsupportedType {
        /// The unsupported datatype
        datatype: DataType,
    },
    /// This error occurs when trying to convert from a record batch with duplicate idents(e.g. `"a"` and `"A"`).
    #[snafu(display("conversion resulted in duplicate idents"))]
    DuplicateIdents,
    /// This error occurs when creating an owned table fails, which should only occur when there are zero columns.
    #[snafu(transparent)]
    InvalidTable {
        /// The underlying source error
        source: OwnedTableError,
    },
    /// Using `TimeError` to handle all time-related errors
    #[snafu(transparent)]
    TimestampConversionError {
        /// The underlying source error
        source: PoSQLTimestampError,
    },
    /// This error occurs when converting from an owned column to an array fails.
    #[snafu(display("decimal conversion failed"))]
    DecimalConversionFailed {
        /// The number that failed to convert
        number: i256,
    },
    /// This error occurs when there's an issue with decimal operations
    #[snafu(transparent)]
    DecimalError {
        /// The underlying decimal error
        source: DecimalError,
    },
    /// This error occurs when there's an issue with table operations.
    #[snafu(display("table error: {source}"))]
    TableError {
        /// The underlying table error
        source: TableError,
    },
}

impl From<TableError> for OwnedArrowConversionError {
    fn from(error: TableError) -> Self {
        Self::TableError { source: error }
    }
}

/// # Panics
///
/// Will panic if setting precision and scale fails when converting `OwnedColumn::Int128`.
/// Will panic if setting precision and scale fails when converting `OwnedColumn::Decimal75`.
/// Will panic if trying to convert `OwnedColumn::Scalar`, as this conversion is not implemented
impl<S: Scalar> From<OwnedColumn<S>> for ArrayRef {
    fn from(value: OwnedColumn<S>) -> Self {
        match value {
            OwnedColumn::Boolean(col) => Arc::new(BooleanArray::from(col)),
            OwnedColumn::Uint8(col) => Arc::new(UInt8Array::from(col)),
            OwnedColumn::TinyInt(col) => Arc::new(Int8Array::from(col)),
            OwnedColumn::SmallInt(col) => Arc::new(Int16Array::from(col)),
            OwnedColumn::Int(col) => Arc::new(Int32Array::from(col)),
            OwnedColumn::BigInt(col) => Arc::new(Int64Array::from(col)),
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
            OwnedColumn::VarChar(col) => Arc::new(StringArray::from(col)),
            OwnedColumn::VarBinary(col) => {
                Arc::new(BinaryArray::from_iter_values(col.iter().map(Vec::as_slice)))
            }
            OwnedColumn::TimestampTZ(time_unit, _, col) => match time_unit {
                PoSQLTimeUnit::Second => Arc::new(TimestampSecondArray::from(col)),
                PoSQLTimeUnit::Millisecond => Arc::new(TimestampMillisecondArray::from(col)),
                PoSQLTimeUnit::Microsecond => Arc::new(TimestampMicrosecondArray::from(col)),
                PoSQLTimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::from(col)),
            },
        }
    }
}

/// # Panics
///
/// Will panic if setting precision and scale fails when converting `OwnedColumn::Int128`.
/// Will panic if setting precision and scale fails when converting `OwnedColumn::Decimal75`.
/// Will panic if trying to convert `OwnedColumn::Scalar`, as this conversion is not implemented
impl<S: Scalar> From<OwnedNullableColumn<S>> for ArrayRef {
    fn from(value: OwnedNullableColumn<S>) -> Self {
        if !value.is_nullable() {
            return ArrayRef::from(value.values);
        }

        let presence = value.presence.unwrap();
        let null_buffer = (0..presence.len())
            .map(|i| presence[i])
            .collect::<NullBuffer>();

        match value.values {
            OwnedColumn::Boolean(col) => Arc::new(BooleanArray::new(col.into(), Some(null_buffer))),
            OwnedColumn::Uint8(col) => Arc::new(UInt8Array::new(col.into(), Some(null_buffer))),
            OwnedColumn::TinyInt(col) => Arc::new(Int8Array::new(col.into(), Some(null_buffer))),
            OwnedColumn::SmallInt(col) => Arc::new(Int16Array::new(col.into(), Some(null_buffer))),
            OwnedColumn::Int(col) => Arc::new(Int32Array::new(col.into(), Some(null_buffer))),
            OwnedColumn::BigInt(col) => Arc::new(Int64Array::new(col.into(), Some(null_buffer))),
            OwnedColumn::Int128(col) => Arc::new(
                Decimal128Array::new(col.into(), Some(null_buffer))
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            OwnedColumn::Decimal75(precision, scale, col) => {
                let converted_col: Vec<i256> = col.iter().map(convert_scalar_to_i256).collect();
                Arc::new(
                    Decimal256Array::new(converted_col.into(), Some(null_buffer))
                        .with_precision_and_scale(precision.value(), scale)
                        .unwrap(),
                )
            }
            OwnedColumn::Scalar(_) => unimplemented!("Cannot convert Scalar type to arrow type"),
            OwnedColumn::VarChar(col) => {
                let mut builder = arrow::array::StringBuilder::with_capacity(
                    col.len(),
                    col.iter().map(String::len).sum(),
                );
                for (i, s) in col.iter().enumerate() {
                    if presence[i] {
                        builder.append_value(s);
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish())
            }
            OwnedColumn::VarBinary(col) => {
                let mut builder = arrow::array::BinaryBuilder::with_capacity(
                    col.len(),
                    col.iter().map(Vec::len).sum(),
                );
                for (i, s) in col.iter().enumerate() {
                    if presence[i] {
                        builder.append_value(s);
                    } else {
                        builder.append_null();
                    }
                }
                Arc::new(builder.finish())
            }
            OwnedColumn::TimestampTZ(time_unit, _, col) => match time_unit {
                PoSQLTimeUnit::Second => {
                    Arc::new(TimestampSecondArray::new(col.into(), Some(null_buffer)))
                }
                PoSQLTimeUnit::Millisecond => Arc::new(TimestampMillisecondArray::new(
                    col.into(),
                    Some(null_buffer),
                )),
                PoSQLTimeUnit::Microsecond => Arc::new(TimestampMicrosecondArray::new(
                    col.into(),
                    Some(null_buffer),
                )),
                PoSQLTimeUnit::Nanosecond => {
                    Arc::new(TimestampNanosecondArray::new(col.into(), Some(null_buffer)))
                }
            },
        }
    }
}

#[allow(clippy::too_many_lines)]
impl<S: Scalar> TryFrom<OwnedTable<S>> for RecordBatch {
    type Error = ArrowError;
    fn try_from(value: OwnedTable<S>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(RecordBatch::new_empty(SchemaRef::new(Schema::empty())))
        } else {
            // Convert each column to an ArrayRef, handling nulls properly
            RecordBatch::try_from_iter(value.into_inner().into_iter().map(
                |(identifier, owned_column)| {
                    // Convert the OwnedColumn to an ArrayRef
                    let array_ref = ArrayRef::from(owned_column);
                    (identifier.value, array_ref)
                },
            ))
        }
    }
}

impl<S: Scalar> TryFrom<ArrayRef> for OwnedNullableColumn<S> {
    type Error = OwnedArrowConversionError;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

#[allow(clippy::too_many_lines)]
impl<S: Scalar> TryFrom<&ArrayRef> for OwnedNullableColumn<S> {
    type Error = OwnedArrowConversionError;

    #[allow(clippy::needless_range_loop)]
    fn try_from(value: &ArrayRef) -> Result<Self, Self::Error> {
        let has_nulls = value.null_count() > 0;
        let len = value.len();

        // If there are no nulls, create a non-nullable column
        if !has_nulls {
            let owned_column = OwnedColumn::try_from(value)?;
            return Ok(OwnedNullableColumn::new(owned_column));
        }

        // Create a presence vector to track which values are NULL
        let mut presence = vec![true; len];

        // Mark NULL values in the presence vector
        for i in 0..len {
            if value.is_null(i) {
                presence[i] = false;
            }
        }

        // Create the column with appropriate default values for NULL entries
        let owned_column = match value.data_type() {
            DataType::Boolean => {
                let array = value.as_any().downcast_ref::<BooleanArray>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) {
                        false // Default value for NULL boolean
                    } else {
                        array.value(i)
                    });
                }
                OwnedColumn::Boolean(values)
            }
            DataType::UInt8 => {
                let array = value.as_any().downcast_ref::<UInt8Array>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                OwnedColumn::Uint8(values)
            }
            DataType::Int8 => {
                let array = value.as_any().downcast_ref::<Int8Array>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                OwnedColumn::TinyInt(values)
            }
            DataType::Int16 => {
                let array = value.as_any().downcast_ref::<Int16Array>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                OwnedColumn::SmallInt(values)
            }
            DataType::Int32 => {
                let array = value.as_any().downcast_ref::<Int32Array>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                OwnedColumn::Int(values)
            }
            DataType::Int64 => {
                let array = value.as_any().downcast_ref::<Int64Array>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                OwnedColumn::BigInt(values)
            }
            DataType::Decimal128(38, 0) => {
                let array = value.as_any().downcast_ref::<Decimal128Array>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                OwnedColumn::Int128(values)
            }
            DataType::Decimal256(precision, scale) if *precision <= 75 => {
                let array = value.as_any().downcast_ref::<Decimal256Array>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    let val = if array.is_null(i) {
                        S::zero()
                    } else {
                        convert_i256_to_scalar(&array.value(i)).ok_or(
                            OwnedArrowConversionError::DecimalConversionFailed {
                                number: array.value(i),
                            },
                        )?
                    };
                    values.push(val);
                }
                OwnedColumn::Decimal75(
                    Precision::new(*precision).expect("precision is less than 76"),
                    *scale,
                    values,
                )
            }
            DataType::Utf8 => {
                let array = value.as_any().downcast_ref::<StringArray>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) {
                        String::new()
                    } else {
                        array.value(i).to_string()
                    });
                }
                OwnedColumn::VarChar(values)
            }
            DataType::Binary => {
                let array = value.as_any().downcast_ref::<BinaryArray>().unwrap();
                let mut values = Vec::with_capacity(len);
                for i in 0..len {
                    values.push(if array.is_null(i) {
                        Vec::new()
                    } else {
                        array.value(i).to_vec()
                    });
                }
                OwnedColumn::VarBinary(values)
            }
            DataType::Timestamp(time_unit, timezone) => match time_unit {
                ArrowTimeUnit::Second => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampSecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let mut timestamps = Vec::with_capacity(len);
                    for i in 0..len {
                        timestamps.push(if array.is_null(i) { 0 } else { array.value(i) });
                    }
                    OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Second,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    )
                }
                ArrowTimeUnit::Millisecond => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampMillisecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let mut timestamps = Vec::with_capacity(len);
                    for i in 0..len {
                        timestamps.push(if array.is_null(i) { 0 } else { array.value(i) });
                    }
                    OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Millisecond,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    )
                }
                ArrowTimeUnit::Microsecond => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampMicrosecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let mut timestamps = Vec::with_capacity(len);
                    for i in 0..len {
                        timestamps.push(if array.is_null(i) { 0 } else { array.value(i) });
                    }
                    OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Microsecond,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    )
                }
                ArrowTimeUnit::Nanosecond => {
                    let array = value
                        .as_any()
                        .downcast_ref::<TimestampNanosecondArray>()
                        .expect(
                            "This cannot fail, all Arrow TimeUnits are mapped to PoSQL TimeUnits",
                        );
                    let mut timestamps = Vec::with_capacity(len);
                    for i in 0..len {
                        timestamps.push(if array.is_null(i) { 0 } else { array.value(i) });
                    }
                    OwnedColumn::TimestampTZ(
                        PoSQLTimeUnit::Nanosecond,
                        PoSQLTimeZone::try_from(timezone)?,
                        timestamps,
                    )
                }
            },
            _ => {
                return Err(OwnedArrowConversionError::UnsupportedType {
                    datatype: value.data_type().clone(),
                })
            }
        };

        Ok(OwnedNullableColumn::with_presence(
            owned_column,
            Some(presence),
        )?)
    }
}

impl<S: Scalar> TryFrom<RecordBatch> for OwnedTable<S> {
    type Error = OwnedArrowConversionError;
    #[allow(clippy::too_many_lines, clippy::needless_range_loop)]
    fn try_from(value: RecordBatch) -> Result<Self, Self::Error> {
        let num_columns = value.schema().fields().len();
        if num_columns == 0 {
            return Ok(Self::try_new(IndexMap::default())?);
        }

        let num_rows = value.num_rows();

        // First pass: collect all NULL positions across all columns
        let mut null_rows = vec![false; num_rows];
        for array_ref in value.columns() {
            for i in 0..num_rows {
                if array_ref.is_null(i) {
                    null_rows[i] = true;
                }
            }
        }

        // Create maps to store columns and presence information
        let mut columns = IndexMap::default();
        let mut presence_map = IndexMap::default();

        // Create a single presence vector that will be used for all columns
        // If any column has NULL in a row, that row is NULL for all columns
        let presence = null_rows
            .iter()
            .map(|&is_null| !is_null)
            .collect::<Vec<bool>>();

        // Second pass: convert columns while zeroing out values in NULL rows
        for (field, array_ref) in value.schema().fields().iter().zip(value.columns()) {
            let identifier = Ident::new(field.name());

            // Convert array to values, setting rows with NULLs anywhere to 0
            let column = match array_ref.data_type() {
                DataType::Boolean => {
                    let array = array_ref.as_any().downcast_ref::<BooleanArray>().unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] { false } else { array.value(i) });
                    }
                    OwnedColumn::Boolean(values)
                }
                DataType::UInt8 => {
                    let array = array_ref.as_any().downcast_ref::<UInt8Array>().unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                    }
                    OwnedColumn::Uint8(values)
                }
                DataType::Int8 => {
                    let array = array_ref.as_any().downcast_ref::<Int8Array>().unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                    }
                    OwnedColumn::TinyInt(values)
                }
                DataType::Int16 => {
                    let array = array_ref.as_any().downcast_ref::<Int16Array>().unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                    }
                    OwnedColumn::SmallInt(values)
                }
                DataType::Int32 => {
                    let array = array_ref.as_any().downcast_ref::<Int32Array>().unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                    }
                    OwnedColumn::Int(values)
                }
                DataType::Int64 => {
                    let array = array_ref.as_any().downcast_ref::<Int64Array>().unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                    }
                    OwnedColumn::BigInt(values)
                }
                DataType::Decimal128(38, 0) => {
                    let array = array_ref
                        .as_any()
                        .downcast_ref::<Decimal128Array>()
                        .unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                    }
                    OwnedColumn::Int128(values)
                }
                DataType::Decimal256(precision, scale) if *precision <= 75 => {
                    let array = array_ref
                        .as_any()
                        .downcast_ref::<Decimal256Array>()
                        .unwrap();
                    let mut values = Vec::with_capacity(num_rows);
                    for i in 0..num_rows {
                        values.push(if null_rows[i] {
                            S::zero()
                        } else {
                            convert_i256_to_scalar(&array.value(i)).ok_or(
                                OwnedArrowConversionError::DecimalConversionFailed {
                                    number: array.value(i),
                                },
                            )?
                        });
                    }
                    OwnedColumn::Decimal75(Precision::new(*precision)?, *scale, values)
                }
                // For non-numeric types, still use the null_rows information
                _ => {
                    let array = match array_ref.data_type() {
                        DataType::Utf8 => {
                            let array = array_ref.as_any().downcast_ref::<StringArray>().unwrap();
                            let mut values = Vec::with_capacity(num_rows);
                            for i in 0..num_rows {
                                values.push(if null_rows[i] {
                                    String::new()
                                } else {
                                    array.value(i).to_string()
                                });
                            }
                            OwnedColumn::VarChar(values)
                        }
                        DataType::Binary => {
                            let array = array_ref.as_any().downcast_ref::<BinaryArray>().unwrap();
                            let mut values = Vec::with_capacity(num_rows);
                            for i in 0..num_rows {
                                values.push(if null_rows[i] {
                                    Vec::new()
                                } else {
                                    array.value(i).to_vec()
                                });
                            }
                            OwnedColumn::VarBinary(values)
                        }
                        DataType::Timestamp(time_unit, timezone) => {
                            let timestamps = match time_unit {
                                ArrowTimeUnit::Second => {
                                    let array = array_ref
                                        .as_any()
                                        .downcast_ref::<TimestampSecondArray>()
                                        .unwrap();
                                    let mut values = Vec::with_capacity(num_rows);
                                    for i in 0..num_rows {
                                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                                    }
                                    values
                                }
                                ArrowTimeUnit::Millisecond => {
                                    let array = array_ref
                                        .as_any()
                                        .downcast_ref::<TimestampMillisecondArray>()
                                        .unwrap();
                                    let mut values = Vec::with_capacity(num_rows);
                                    for i in 0..num_rows {
                                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                                    }
                                    values
                                }
                                ArrowTimeUnit::Microsecond => {
                                    let array = array_ref
                                        .as_any()
                                        .downcast_ref::<TimestampMicrosecondArray>()
                                        .unwrap();
                                    let mut values = Vec::with_capacity(num_rows);
                                    for i in 0..num_rows {
                                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                                    }
                                    values
                                }
                                ArrowTimeUnit::Nanosecond => {
                                    let array = array_ref
                                        .as_any()
                                        .downcast_ref::<TimestampNanosecondArray>()
                                        .unwrap();
                                    let mut values = Vec::with_capacity(num_rows);
                                    for i in 0..num_rows {
                                        values.push(if null_rows[i] { 0 } else { array.value(i) });
                                    }
                                    values
                                }
                            };
                            OwnedColumn::TimestampTZ(
                                match time_unit {
                                    ArrowTimeUnit::Second => PoSQLTimeUnit::Second,
                                    ArrowTimeUnit::Millisecond => PoSQLTimeUnit::Millisecond,
                                    ArrowTimeUnit::Microsecond => PoSQLTimeUnit::Microsecond,
                                    ArrowTimeUnit::Nanosecond => PoSQLTimeUnit::Nanosecond,
                                },
                                PoSQLTimeZone::try_from(timezone)?,
                                timestamps,
                            )
                        }
                        _ => {
                            return Err(OwnedArrowConversionError::UnsupportedType {
                                datatype: array_ref.data_type().clone(),
                            })
                        }
                    };
                    array
                }
            };

            // Store the column and use the same presence vector for all columns
            columns.insert(identifier.clone(), column);
            presence_map.insert(identifier, presence.clone());
        }

        // Create the OwnedTable with columns and presence information
        let mut owned_table = Self::try_new(columns)?;

        // Add presence information to the table
        for (ident, presence) in presence_map {
            owned_table.set_presence(ident, presence);
        }

        if num_columns == owned_table.num_columns() {
            Ok(owned_table)
        } else {
            Err(OwnedArrowConversionError::DuplicateIdents)
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

    #[allow(clippy::too_many_lines)]
    /// # Panics
    ///
    /// Will panic if downcasting fails for the following types:
    /// - `BooleanArray` when converting from `DataType::Boolean`.
    /// - `Int16Array` when converting from `DataType::Int16`.
    /// - `Int32Array` when converting from `DataType::Int32`.
    /// - `Int64Array` when converting from `DataType::Int64`.
    /// - `Decimal128Array` when converting from `DataType::Decimal128(38, 0)`.
    /// - `Decimal256Array` when converting from `DataType::Decimal256` if precision is less than or equal to 75.
    /// - `StringArray` when converting from `DataType::Utf8`.
    #[allow(clippy::too_many_lines)]
    fn try_from(value: &ArrayRef) -> Result<Self, Self::Error> {
        // If the array has nulls, we should use OwnedNullableColumn instead
        // This function only handles non-nullable columns
        if value.null_count() > 0 {
            // Return an error indicating that the array has nulls and should be converted to OwnedNullableColumn
            return Err(OwnedArrowConversionError::UnsupportedType {
                datatype: DataType::Null,
            });
        }

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
                    .unwrap_or_default(),
            )),
            DataType::UInt8 => Ok(Self::Uint8(
                value
                    .as_any()
                    .downcast_ref::<UInt8Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Int8 => Ok(Self::TinyInt(
                value
                    .as_any()
                    .downcast_ref::<Int8Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Int16 => Ok(Self::SmallInt(
                value
                    .as_any()
                    .downcast_ref::<Int16Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
            )),
            DataType::Int32 => Ok(Self::Int(
                value
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .unwrap()
                    .values()
                    .to_vec(),
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
            DataType::Decimal256(precision, scale) if *precision <= 75 => Ok(Self::Decimal75(
                Precision::new(*precision).expect("precision is less than 76"),
                *scale,
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
                value
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .iter()
                    .map(|s| s.unwrap().to_string())
                    .collect(),
            )),
            DataType::Binary => Ok(Self::VarBinary(
                value
                    .as_any()
                    .downcast_ref::<BinaryArray>()
                    .unwrap()
                    .iter()
                    .map(|s| s.map(<[u8]>::to_vec).unwrap())
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
            &data_type => Err(OwnedArrowConversionError::UnsupportedType {
                datatype: data_type.clone(),
            }),
        }
    }
}
