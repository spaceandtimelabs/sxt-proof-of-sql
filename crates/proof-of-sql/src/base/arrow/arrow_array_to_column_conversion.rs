use super::scalar_and_i256_conversions::convert_i256_to_scalar;
use crate::base::{
    database::{Column, NullableColumn},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestampError},
    scalar::{Scalar, ScalarExt},
};
use arrow::{
    array::{
        Array, ArrayRef, BinaryArray, BooleanArray, Decimal128Array, Decimal256Array, Int16Array,
        Int32Array, Int64Array, Int8Array, StringArray, TimestampMicrosecondArray,
        TimestampMillisecondArray, TimestampNanosecondArray, TimestampSecondArray, UInt8Array,
    },
    datatypes::{i256, DataType, TimeUnit as ArrowTimeUnit},
};
use bumpalo::Bump;
use core::ops::Range;
use snafu::Snafu;

#[derive(Snafu, Debug, PartialEq)]
/// Errors caused by conversions between Arrow and owned types.
pub enum ArrowArrayToColumnConversionError {
    /// This error occurs when trying to convert from an unsupported arrow type.
    #[snafu(display(
        "unsupported type: attempted conversion from ArrayRef of type {datatype} to OwnedColumn"
    ))]
    UnsupportedType {
        /// The unsupported datatype
        datatype: DataType,
    },
    /// Variant for decimal errors
    #[snafu(transparent)]
    DecimalError {
        /// The underlying source error
        source: crate::base::math::decimal::DecimalError,
    },
    /// This error occurs when trying to convert from an i256 to a Scalar.
    #[snafu(display("decimal conversion failed: {number}"))]
    DecimalConversionFailed {
        /// The `i256` value for which conversion is attempted
        number: i256,
    },
    /// This error occurs when the specified range is out of the bounds of the array.
    #[snafu(display("index out of bounds: the len is {len} but the index is {index}"))]
    IndexOutOfBounds {
        /// The actual length of the array
        len: usize,
        /// The out of bounds index requested
        index: usize,
    },
    /// Using `TimeError` to handle all time-related errors
    #[snafu(transparent)]
    TimestampConversionError {
        /// The underlying source error
        source: PoSQLTimestampError,
    },
    /// This error occurs when there's an issue with column operations
    #[snafu(display("column operation error"))]
    ColumnOperationError,
}

/// This trait is used to provide utility functions to convert [`ArrayRef`]s into proof types (Column, Scalars, etc.)
pub trait ArrayRefExt {
    /// Convert an [`ArrayRef`] into a Proof of SQL Column type
    ///
    /// Parameters:
    /// - `alloc`: used to allocate a slice of data when necessary
    ///    (vide [`StringArray`] into `Column::HashedBytes((_,_))`.
    ///
    /// - `range`: used to get a subslice out of [`ArrayRef`].
    ///
    /// - `scals`: scalar representation of each element in the [`ArrayRef`].
    ///    Some types don't require this slice (see [`Column::BigInt`]). But for types requiring it,
    ///    `scals` must be provided and have a length equal to `range.len()`.
    ///
    /// Note: this function must not be called from unsupported or nullable arrays as it will panic.
    fn to_column<'a, S: Scalar>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        scals: Option<&'a [S]>,
    ) -> Result<Column<'a, S>, ArrowArrayToColumnConversionError>;

    /// Convert an [`ArrayRef`] into a Proof of SQL `NullableColumn` type, handling null values
    ///
    /// Parameters:
    /// - `alloc`: used to allocate slices of data
    /// - `range`: used to get a subslice out of [`ArrayRef`]
    /// - `scals`: optional scalar representation of elements
    ///
    /// This function handles arrays with null values, unlike `to_column` which rejects them.
    fn to_nullable_column<'a, S: Scalar>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        scals: Option<&'a [S]>,
    ) -> Result<NullableColumn<'a, S>, ArrowArrayToColumnConversionError>;
}

impl ArrayRefExt for ArrayRef {
    /// Converts the given `ArrowArray` into a [`Column`] data type based on its [`DataType`]. Returns an
    /// empty [`Column`] for any empty range if it is in-bounds.
    ///
    /// # Parameters
    /// - `alloc`: Reference to a `Bump` allocator used for memory allocation during the conversion.
    /// - `range`: Reference to a `Range<usize>` specifying the slice of the array to convert.
    /// - `precomputed_scals`: Optional reference to a slice of `TestScalars` values.
    ///    `VarChar` columns store hashes to their values as scalars, which can be provided here.
    ///
    /// # Supported types
    /// - For `DataType::Int64` and `DataType::Decimal128(38, 0)`, it slices the array
    ///   based on the provided range and returns the corresponding `BigInt` or `Int128` column.
    /// - Decimal256, converts arrow i256 columns into Decimal75(precision, scale) columns.
    /// - For `DataType::Utf8`, it extracts string values and scalar values (if `precomputed_scals`
    ///   is provided) for the specified range and returns a `VarChar` column.
    ///
    /// # Panics
    /// - When any range is OOB, i.e. indexing 3..6 or 5..5 on array of size 2.
    #[expect(clippy::too_many_lines)]
    fn to_column<'a, S: Scalar>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        precomputed_scals: Option<&'a [S]>,
    ) -> Result<Column<'a, S>, ArrowArrayToColumnConversionError> {
        // Before performing any operations, check if the range is out of bounds
        if range.end > self.len() {
            return Err(ArrowArrayToColumnConversionError::IndexOutOfBounds {
                len: self.len(),
                index: range.end,
            });
        }

        // If the array has nulls, use to_nullable_column and extract the column
        if self.null_count() > 0 {
            let nullable_column = self.to_nullable_column(alloc, range, precomputed_scals)?;
            return Ok(nullable_column.values);
        }

        // Match supported types and attempt conversion
        match self.data_type() {
            DataType::Boolean => {
                if let Some(array) = self.as_any().downcast_ref::<BooleanArray>() {
                    let boolean_slice = array
                        .iter()
                        .skip(range.start)
                        .take(range.len())
                        .collect::<Option<Vec<bool>>>()
                        .ok_or(ArrowArrayToColumnConversionError::ColumnOperationError)?;
                    let values = alloc.alloc_slice_fill_with(range.len(), |i| boolean_slice[i]);
                    Ok(Column::Boolean(values))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::UInt8 => {
                if let Some(array) = self.as_any().downcast_ref::<UInt8Array>() {
                    Ok(Column::Uint8(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Int8 => {
                if let Some(array) = self.as_any().downcast_ref::<Int8Array>() {
                    Ok(Column::TinyInt(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Int16 => {
                if let Some(array) = self.as_any().downcast_ref::<Int16Array>() {
                    Ok(Column::SmallInt(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Int32 => {
                if let Some(array) = self.as_any().downcast_ref::<Int32Array>() {
                    Ok(Column::Int(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Int64 => {
                if let Some(array) = self.as_any().downcast_ref::<Int64Array>() {
                    Ok(Column::BigInt(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Decimal128(38, 0) => {
                if let Some(array) = self.as_any().downcast_ref::<Decimal128Array>() {
                    Ok(Column::Int128(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            #[allow(clippy::similar_names)]
            DataType::Decimal256(precision, scale) if *precision <= 75 => {
                if let Some(array) = self.as_any().downcast_ref::<Decimal256Array>() {
                    let i256_slice = &array.values()[range.start..range.end];
                    let scalars = alloc.alloc_slice_fill_default(i256_slice.len());
                    for (scalar, value) in scalars.iter_mut().zip(i256_slice) {
                        *scalar = convert_i256_to_scalar(value).ok_or(
                            ArrowArrayToColumnConversionError::DecimalConversionFailed {
                                number: *value,
                            },
                        )?;
                    }
                    Ok(Column::Decimal75(
                        Precision::new(*precision)?,
                        *scale,
                        scalars,
                    ))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            // Handle all possible TimeStamp TimeUnit instances
            DataType::Timestamp(time_unit, tz) => match time_unit {
                ArrowTimeUnit::Second => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampSecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Second,
                            PoSQLTimeZone::try_from(tz)?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType {
                            datatype: self.data_type().clone(),
                        })
                    }
                }
                ArrowTimeUnit::Millisecond => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampMillisecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Millisecond,
                            PoSQLTimeZone::try_from(tz)?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType {
                            datatype: self.data_type().clone(),
                        })
                    }
                }
                ArrowTimeUnit::Microsecond => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampMicrosecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Microsecond,
                            PoSQLTimeZone::try_from(tz)?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType {
                            datatype: self.data_type().clone(),
                        })
                    }
                }
                ArrowTimeUnit::Nanosecond => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampNanosecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Nanosecond,
                            PoSQLTimeZone::try_from(tz)?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType {
                            datatype: self.data_type().clone(),
                        })
                    }
                }
            },
            DataType::Utf8 => {
                if let Some(array) = self.as_any().downcast_ref::<StringArray>() {
                    let vals = alloc
                        .alloc_slice_fill_with(range.end - range.start, |i| -> &'a str {
                            array.value(range.start + i)
                        });

                    let scals = if let Some(scals) = precomputed_scals {
                        &scals[range.start..range.end]
                    } else {
                        alloc.alloc_slice_fill_with(vals.len(), |i| -> S { vals[i].into() })
                    };

                    Ok(Column::VarChar((vals, scals)))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Binary => {
                if let Some(array) = self.as_any().downcast_ref::<BinaryArray>() {
                    let vals = alloc
                        .alloc_slice_fill_with(range.end - range.start, |i| -> &'a [u8] {
                            array.value(range.start + i)
                        });

                    let scals = if let Some(scals) = precomputed_scals {
                        &scals[range.start..range.end]
                    } else {
                        alloc.alloc_slice_fill_with(vals.len(), |i| {
                            S::from_byte_slice_via_hash(vals[i])
                        })
                    };

                    Ok(Column::VarBinary((vals, scals)))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            data_type => Err(ArrowArrayToColumnConversionError::UnsupportedType {
                datatype: data_type.clone(),
            }),
        }
    }

    /// Converts the given `ArrowArray` into a [`NullableColumn`] data type, handling null values.
    ///
    /// # Parameters
    /// - `alloc`: Reference to a `Bump` allocator used for memory allocation during the conversion.
    /// - `range`: Reference to a `Range<usize>` specifying the slice of the array to convert.
    /// - `precomputed_scals`: Optional reference to a slice of `TestScalars` values.
    ///
    /// # Panics
    /// - When any range is OOB, i.e. indexing 3..6 or 5..5 on array of size 2.
    #[allow(clippy::too_many_lines)]
    fn to_nullable_column<'a, S: Scalar>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        precomputed_scals: Option<&'a [S]>,
    ) -> Result<NullableColumn<'a, S>, ArrowArrayToColumnConversionError> {
        // Before performing any operations, check if the range is out of bounds
        if range.end > self.len() {
            return Err(ArrowArrayToColumnConversionError::IndexOutOfBounds {
                len: self.len(),
                index: range.end,
            });
        }

        // If no nulls, defer to regular to_column and wrap the result
        if self.null_count() == 0 {
            let column = self.to_column(alloc, range, precomputed_scals)?;
            return Ok(NullableColumn::new(column));
        }

        // Create a presence slice to track nulls (true = present, false = null)
        let range_len = range.len();
        let mut presence_vec = Vec::with_capacity(range_len);
        for i in range.clone() {
            presence_vec.push(!self.is_null(i));
        }
        let presence_slice = alloc.alloc_slice_copy(&presence_vec);

        // Create a column with default values for null positions
        match self.data_type() {
            DataType::Boolean => {
                let array = self.as_any().downcast_ref::<BooleanArray>().unwrap();
                let mut bool_vec = Vec::with_capacity(range_len);
                for i in range.clone() {
                    // Use false as the default value for nulls
                    bool_vec.push(if array.is_null(i) {
                        false
                    } else {
                        array.value(i)
                    });
                }
                let values = alloc.alloc_slice_fill_with(range_len, |i| bool_vec[i]);
                NullableColumn::with_presence(Column::Boolean(values), Some(presence_slice))
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::UInt8 => {
                let array = self.as_any().downcast_ref::<UInt8Array>().unwrap();
                let mut values_vec = Vec::with_capacity(range_len);
                for i in range.clone() {
                    values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                let values_slice = alloc.alloc_slice_copy(&values_vec);
                NullableColumn::with_presence(Column::Uint8(values_slice), Some(presence_slice))
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::Int8 => {
                let array = self.as_any().downcast_ref::<Int8Array>().unwrap();
                let mut values_vec = Vec::with_capacity(range_len);
                for i in range.clone() {
                    values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                let values_slice = alloc.alloc_slice_copy(&values_vec);
                NullableColumn::with_presence(Column::TinyInt(values_slice), Some(presence_slice))
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::Int16 => {
                let array = self.as_any().downcast_ref::<Int16Array>().unwrap();
                let mut values_vec = Vec::with_capacity(range_len);
                for i in range.clone() {
                    values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                let values_slice = alloc.alloc_slice_copy(&values_vec);
                NullableColumn::with_presence(Column::SmallInt(values_slice), Some(presence_slice))
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::Int32 => {
                let array = self.as_any().downcast_ref::<Int32Array>().unwrap();
                let mut values_vec = Vec::with_capacity(range_len);
                for i in range.clone() {
                    values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                let values_slice = alloc.alloc_slice_copy(&values_vec);
                NullableColumn::with_presence(Column::Int(values_slice), Some(presence_slice))
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::Int64 => {
                let array = self.as_any().downcast_ref::<Int64Array>().unwrap();
                let mut values_vec = Vec::with_capacity(range_len);
                for i in range.clone() {
                    values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                let values_slice = alloc.alloc_slice_copy(&values_vec);
                NullableColumn::with_presence(Column::BigInt(values_slice), Some(presence_slice))
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::Decimal128(38, 0) => {
                let array = self.as_any().downcast_ref::<Decimal128Array>().unwrap();
                let mut values_vec = Vec::with_capacity(range_len);
                for i in range.clone() {
                    values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                }
                let values_slice = alloc.alloc_slice_copy(&values_vec);
                NullableColumn::with_presence(Column::Int128(values_slice), Some(presence_slice))
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::Decimal256(precision, scale) if *precision <= 75 => {
                let array = self.as_any().downcast_ref::<Decimal256Array>().unwrap();
                let mut scalar_values = Vec::with_capacity(range_len);
                for i in range.clone() {
                    scalar_values.push(if array.is_null(i) {
                        S::zero()
                    } else {
                        convert_i256_to_scalar(&array.value(i)).ok_or(
                            ArrowArrayToColumnConversionError::DecimalConversionFailed {
                                number: array.value(i),
                            },
                        )?
                    });
                }
                let scalars = alloc.alloc_slice_copy(&scalar_values);
                NullableColumn::with_presence(
                    Column::Decimal75(Precision::new(*precision)?, *scale, scalars),
                    Some(presence_slice),
                )
                .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            DataType::Utf8 => {
                let array = self.as_any().downcast_ref::<StringArray>().unwrap();
                let strings = alloc.alloc_slice_fill_with(range_len, |offset| {
                    let i = offset + range.start;
                    if array.is_null(i) {
                        // Use empty string as the default value for nulls
                        ""
                    } else {
                        array.value(i)
                    }
                });

                if let Some(scals) = precomputed_scals {
                    debug_assert_eq!(
                        scals.len(),
                        range_len,
                        "Precomputed scalars length must match range length"
                    );
                    NullableColumn::with_presence(
                        Column::VarChar((strings, scals)),
                        Some(presence_slice),
                    )
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Binary => {
                let array = self.as_any().downcast_ref::<BinaryArray>().unwrap();
                let mut binaries = Vec::with_capacity(range_len);
                for i in range.clone() {
                    if array.is_null(i) {
                        // Use empty Vec as the default value for nulls
                        binaries.push(&[] as &[u8]);
                    } else {
                        binaries.push(array.value(i));
                    }
                }
                let binary_refs = alloc.alloc_slice_fill_with(range_len, |offset| binaries[offset]);

                if let Some(scals) = precomputed_scals {
                    debug_assert_eq!(
                        scals.len(),
                        range_len,
                        "Precomputed scalars length must match range length"
                    );
                    NullableColumn::with_presence(
                        Column::VarBinary((binary_refs, scals)),
                        Some(presence_slice),
                    )
                    .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType {
                        datatype: self.data_type().clone(),
                    })
                }
            }
            DataType::Timestamp(time_unit, tz) => {
                let mut values_vec = Vec::with_capacity(range_len);

                match time_unit {
                    ArrowTimeUnit::Second => {
                        let array = self
                            .as_any()
                            .downcast_ref::<TimestampSecondArray>()
                            .unwrap();
                        for i in range.clone() {
                            values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                        }
                    }
                    ArrowTimeUnit::Millisecond => {
                        let array = self
                            .as_any()
                            .downcast_ref::<TimestampMillisecondArray>()
                            .unwrap();
                        for i in range.clone() {
                            values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                        }
                    }
                    ArrowTimeUnit::Microsecond => {
                        let array = self
                            .as_any()
                            .downcast_ref::<TimestampMicrosecondArray>()
                            .unwrap();
                        for i in range.clone() {
                            values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                        }
                    }
                    ArrowTimeUnit::Nanosecond => {
                        let array = self
                            .as_any()
                            .downcast_ref::<TimestampNanosecondArray>()
                            .unwrap();
                        for i in range.clone() {
                            values_vec.push(if array.is_null(i) { 0 } else { array.value(i) });
                        }
                    }
                }

                let values_slice = alloc.alloc_slice_copy(&values_vec);
                let time_unit = match time_unit {
                    ArrowTimeUnit::Second => PoSQLTimeUnit::Second,
                    ArrowTimeUnit::Millisecond => PoSQLTimeUnit::Millisecond,
                    ArrowTimeUnit::Microsecond => PoSQLTimeUnit::Microsecond,
                    ArrowTimeUnit::Nanosecond => PoSQLTimeUnit::Nanosecond,
                };

                NullableColumn::with_presence(
                    Column::TimestampTZ(time_unit, PoSQLTimeZone::try_from(tz)?, values_slice),
                    Some(presence_slice),
                )
                .map_err(|_| ArrowArrayToColumnConversionError::ColumnOperationError)
            }
            _ => Err(ArrowArrayToColumnConversionError::UnsupportedType {
                datatype: self.data_type().clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        base::{database::OwnedColumn, scalar::test_scalar::TestScalar},
        proof_primitive::dory::DoryScalar,
    };
    use alloc::sync::Arc;
    use arrow::array::Decimal256Builder;
    use core::str::FromStr;
    use proptest::prelude::*;

    #[test]
    fn we_can_convert_timestamp_array_normal_range() {
        let alloc = Bump::new();
        let data = vec![1_625_072_400, 1_625_076_000, 1_625_083_200]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("Z"),
        ));

        let result = array.to_column::<TestScalar>(&alloc, &(1..3), None);
        assert_eq!(
            result.unwrap(),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &data[1..3])
        );
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_timestamp() {
        let alloc = Bump::new();
        let data = vec![1_625_072_400, 1_625_076_000]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.into(),
            Some("+00:00"),
        ));

        let result = array
            .to_column::<DoryScalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(
            result,
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[])
        );
    }

    #[test]
    fn we_can_convert_timestamp_array_empty_range() {
        let alloc = Bump::new();
        let data = vec![1_625_072_400, 1_625_076_000, 1_625_083_200]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.into(),
            Some("+0:00"),
        ));

        let result = array.to_column::<DoryScalar>(&alloc, &(1..1), None);
        assert_eq!(
            result.unwrap(),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[])
        );
    }

    #[test]
    fn we_cannot_convert_timestamp_array_oob_range() {
        let alloc = Bump::new();
        let data = vec![1_625_072_400, 1_625_076_000, 1_625_083_200];
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.into(),
            Some("Utc"),
        ));

        let result = array.to_column::<TestScalar>(&alloc, &(3..5), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 5 })
        );
    }

    #[test]
    fn we_can_convert_timestamp_array_with_nulls() {
        let alloc = Bump::new();
        let array = TimestampSecondArray::from(vec![Some(100), None, Some(300)]);
        let array_ref = Arc::new(array) as ArrayRef;

        // Test to_nullable_column
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::TimestampTZ(time_unit, _, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Second);
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 100);
                assert_eq!(values[1], 0); // Default value for NULL
                assert_eq!(values[2], 300);
            }
            _ => panic!("Expected TimestampTZ column"),
        }
    }

    #[test]
    fn we_cannot_convert_utf8_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(StringArray::from(vec!["hello", "world", "test"]));
        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_can_convert_utf8_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(StringArray::from(vec!["hello", "world", "test"]));
        let result = array.to_column::<TestScalar>(&alloc, &(1..3), None);
        let expected_vals = vec!["world", "test"];
        let expected_scals: Vec<TestScalar> = expected_vals.iter().map(|&v| v.into()).collect();

        assert_eq!(
            result.unwrap(),
            Column::VarChar((expected_vals.as_slice(), expected_scals.as_slice()))
        );
    }

    #[test]
    fn we_can_convert_utf8_array_empty_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(StringArray::from(vec!["hello", "world", "test"]));
        let result = array.to_column::<DoryScalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::VarChar((&[], &[])));
    }

    #[test]
    fn we_can_convert_utf8_array_with_nulls() {
        let alloc = Bump::new();
        let array = StringArray::from(vec![Some("hello"), None, Some("world")]);
        let array_ref = Arc::new(array) as ArrayRef;

        // Create precomputed scalars
        let scalars = [
            TestScalar::from("hello"),
            TestScalar::from(""), // Default value for NULL
            TestScalar::from("world"),
        ];
        let scalar_slice = alloc.alloc_slice_copy(&scalars);

        // Test to_nullable_column
        let nullable_result =
            array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), Some(scalar_slice));
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), Some(scalar_slice));
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::VarChar((strings, scalars)) => {
                assert_eq!(strings.len(), 3);
                assert_eq!(strings[0], "hello");
                assert_eq!(strings[1], ""); // Default value for NULL
                assert_eq!(strings[2], "world");
                // Verify scalars are computed correctly
                assert_eq!(scalars[0], TestScalar::from("hello"));
                assert_eq!(scalars[1], TestScalar::from("")); // Default value for NULL
                assert_eq!(scalars[2], TestScalar::from("world"));
            }
            _ => panic!("Expected VarChar column"),
        }
    }

    #[test]
    fn we_can_convert_utf8_array_with_precomputed_scalars() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(StringArray::from(vec!["hello", "world", "test"]));
        let precomputed_scals: Vec<DoryScalar> = ["hello", "world", "test"]
            .iter()
            .map(|&v| v.into())
            .collect();
        let result = array.to_column::<DoryScalar>(&alloc, &(1..3), Some(&precomputed_scals));
        let expected_vals = vec!["world", "test"];
        let expected_scals = &precomputed_scals[1..3];

        assert_eq!(
            result.unwrap(),
            Column::VarChar((expected_vals.as_slice(), expected_scals))
        );
    }

    #[test]
    fn we_cannot_convert_decimal256_array_with_high_precision() {
        let alloc = Bump::new();
        let mut builder = Decimal256Builder::with_capacity(3);
        builder.append_value(i256::from_str("100000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("-300000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("4200000000000000000000000000000000000000").unwrap());

        let array: ArrayRef = Arc::new(builder.finish().with_precision_and_scale(76, 0).unwrap());
        let result = array.to_column::<TestScalar>(&alloc, &(1..3), None);
        assert!(result.is_err());
    }

    #[test]
    fn we_can_convert_decimal256_array_normal_range() {
        let alloc = Bump::new();
        let mut builder = Decimal256Builder::with_capacity(3);
        builder.append_value(i256::from_str("100000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("-300000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("4200000000000000000000000000000000000000").unwrap());
        let array: ArrayRef = Arc::new(builder.finish().with_precision_and_scale(75, 0).unwrap());

        let result = array.to_column::<TestScalar>(&alloc, &(1..3), None);
        let expected_scalars: Vec<TestScalar> = vec![
            convert_i256_to_scalar(
                &i256::from_str("-300000000000000000000000000000000000000").unwrap(),
            )
            .unwrap(),
            convert_i256_to_scalar(
                &i256::from_str("4200000000000000000000000000000000000000").unwrap(),
            )
            .unwrap(),
        ];
        assert_eq!(
            result.unwrap(),
            Column::Decimal75(Precision::new(75).unwrap(), 0, expected_scalars.as_slice())
        );
    }

    #[test]
    fn we_can_convert_decimal256_array_empty_range() {
        let alloc = Bump::new();
        let mut builder = Decimal256Builder::with_capacity(3);
        builder.append_value(i256::from_str("100000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("-300000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("4200000000000000000000000000000000000000").unwrap());
        let array: ArrayRef = Arc::new(builder.finish().with_precision_and_scale(75, 0).unwrap());

        let result = array.to_column::<TestScalar>(&alloc, &(1..1), None);
        assert_eq!(
            result.unwrap(),
            Column::Decimal75(Precision::new(75).unwrap(), 0, &[])
        );
    }

    #[test]
    fn we_cannot_convert_decimal256_array_oob_range() {
        let alloc = Bump::new();
        let mut builder = Decimal256Builder::with_capacity(3);
        builder.append_value(i256::from_str("100000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("-300000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("4200000000000000000000000000000000000000").unwrap());
        let array: ArrayRef = Arc::new(builder.finish().with_precision_and_scale(75, 0).unwrap());

        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_can_convert_decimal256_array_with_nulls() {
        let alloc = Bump::new();
        let array = Decimal256Array::from(vec![
            Some(i256::from_i128(10)),
            None,
            Some(i256::from_i128(30)),
        ])
        .with_precision_and_scale(75, 0)
        .unwrap();
        let array_ref = Arc::new(array) as ArrayRef;

        // Test to_nullable_column
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::Decimal75(precision, scale, values) => {
                assert_eq!(precision.value(), 75);
                assert_eq!(scale, 0);
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], TestScalar::from(10i128));
                assert_eq!(values[1], TestScalar::from(0i128)); // Default value for NULL
                assert_eq!(values[2], TestScalar::from(30i128));
            }
            _ => panic!("Expected Decimal75 column"),
        }
    }

    #[test]
    fn we_can_convert_decimal128_array_empty_range() {
        let alloc = Bump::new();
        let data = vec![100_i128, -300_i128, 4200_i128];
        let array: ArrayRef = Arc::new(
            Decimal128Array::from_iter_values(data.clone())
                .with_precision_and_scale(38, 0)
                .unwrap(),
        );

        let result = array.to_column::<DoryScalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::Int128(&[]));
    }

    #[test]
    fn we_cannot_convert_decimal128_array_oob_range() {
        let alloc = Bump::new();
        let data = vec![100_i128, -300_i128, 4200_i128];
        let array: ArrayRef = Arc::new(
            Decimal128Array::from_iter_values(data)
                .with_precision_and_scale(38, 0)
                .unwrap(),
        );

        let result = array.to_column::<TestScalar>(&alloc, &(2..4), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_can_convert_decimal128_array_with_nulls() {
        let alloc = Bump::new();
        let array = Decimal128Array::from(vec![Some(10), None, Some(30)])
            .with_precision_and_scale(38, 0)
            .unwrap();
        let array_ref = Arc::new(array) as ArrayRef;

        // Test to_nullable_column
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::Int128(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 0); // Default value for NULL
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected Int128 column"),
        }
    }

    #[test]
    fn we_can_convert_boolean_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(false),
            Some(true),
        ]));
        let result = array.to_column::<DoryScalar>(&alloc, &(1..3), None);
        assert_eq!(result.unwrap(), Column::Boolean(&[false, true]));
    }

    #[test]
    fn we_can_convert_boolean_array_empty_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(false),
            Some(true),
        ]));
        let result = array.to_column::<TestScalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::Boolean(&[]));
    }

    #[test]
    fn we_cannot_convert_boolean_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(false),
            Some(true),
        ]));

        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);

        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_can_convert_boolean_array_with_nulls() {
        let alloc = Bump::new();
        let array = BooleanArray::from(vec![Some(true), None, Some(false)]);
        let array_ref = Arc::new(array) as ArrayRef;

        // Test to_nullable_column
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::Boolean(values) => {
                assert_eq!(values.len(), 3);
                assert!(values[0]);
                assert!(!values[1]); // Default value for NULL
                assert!(!values[2]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn we_can_convert_int8_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int8Array::from(vec![1, -3, 42]));
        let result = array.to_column::<DoryScalar>(&alloc, &(1..3), None);
        assert_eq!(result.unwrap(), Column::TinyInt(&[-3, 42]));
    }

    #[test]
    fn we_can_convert_int16_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int16Array::from(vec![1, -3, 42]));
        let result = array.to_column::<DoryScalar>(&alloc, &(1..3), None);
        assert_eq!(result.unwrap(), Column::SmallInt(&[-3, 42]));
    }

    #[test]
    fn we_can_convert_int8_array_empty_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int8Array::from(vec![1, -3, 42]));
        let result = array.to_column::<TestScalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::TinyInt(&[]));
    }

    #[test]
    fn we_can_convert_int16_array_empty_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int16Array::from(vec![1, -3, 42]));
        let result = array.to_column::<TestScalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::SmallInt(&[]));
    }

    #[test]
    fn we_cannot_convert_int8_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int8Array::from(vec![1, -3, 42]));

        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);

        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_cannot_convert_int16_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int16Array::from(vec![1, -3, 42]));

        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);

        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_can_convert_int8_array_with_nulls() {
        let alloc = Bump::new();
        let array = Int8Array::from(vec![Some(10), None, Some(30)]);
        let array_ref = Arc::new(array) as ArrayRef;

        // Test to_nullable_column
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::TinyInt(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 0); // Default value for NULL
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected TinyInt column"),
        }
    }

    #[test]
    fn we_can_convert_int16_array_with_nulls() {
        let alloc = Bump::new();
        let array = Int16Array::from(vec![Some(10), None, Some(30)]);
        let array_ref = Arc::new(array) as ArrayRef;

        // Test to_nullable_column
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::SmallInt(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 0); // Default value for NULL
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected SmallInt column"),
        }
    }

    #[test]
    fn we_can_convert_int32_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int32Array::from(vec![1, -3, 42]));
        let result = array.to_column::<DoryScalar>(&alloc, &(1..3), None);
        assert_eq!(result.unwrap(), Column::Int(&[-3, 42]));
    }

    #[test]
    fn we_can_convert_int32_array_empty_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int32Array::from(vec![1, -3, 42]));
        let result = array.to_column::<TestScalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::Int(&[]));
    }

    #[test]
    fn we_cannot_convert_int32_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int32Array::from(vec![1, -3, 42]));

        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);

        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_can_convert_int32_array_with_nulls() {
        let alloc = Bump::new();
        let array = Int32Array::from(vec![Some(10), None, Some(30)]);
        let array_ref = Arc::new(array) as ArrayRef;

        // Test to_nullable_column
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        // Test to_column - should now handle nulls by delegating to to_nullable_column
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::Int(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 0); // Default value for NULL
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected Int column"),
        }
    }

    #[test]
    fn we_cannot_index_on_oob_range() {
        let alloc = Bump::new();

        let array0: ArrayRef = Arc::new(arrow::array::Int8Array::from(vec![1, -3]));
        let result0 = array0.to_column::<DoryScalar>(&alloc, &(2..3), None);
        assert_eq!(
            result0,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 3 })
        );

        let array1: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        let result1 = array1.to_column::<DoryScalar>(&alloc, &(2..3), None);
        assert_eq!(
            result1,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 3 })
        );

        let array2: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3]));
        let result2 = array2.to_column::<DoryScalar>(&alloc, &(2..3), None);
        assert_eq!(
            result2,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 3 })
        );

        let array3: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        let result3 = array3.to_column::<DoryScalar>(&alloc, &(2..3), None);
        assert_eq!(
            result3,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 3 })
        );
    }

    #[test]
    fn we_cannot_index_on_empty_oob_range() {
        let alloc = Bump::new();

        let array0: ArrayRef = Arc::new(arrow::array::Int8Array::from(vec![1, -3]));
        let result0 = array0.to_column::<DoryScalar>(&alloc, &(5..5), None);
        assert_eq!(
            result0,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 5 })
        );

        let array1: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        let result1 = array1.to_column::<TestScalar>(&alloc, &(5..5), None);
        assert_eq!(
            result1,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 5 })
        );

        let array2: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3]));
        let result2 = array2.to_column::<DoryScalar>(&alloc, &(5..5), None);
        assert_eq!(
            result2,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 5 })
        );

        let array3: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        let result3 = array3.to_column::<TestScalar>(&alloc, &(5..5), None);
        assert_eq!(
            result3,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 5 })
        );
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_boolean() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(vec![true, false]));
        let result = array
            .to_column::<DoryScalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::Boolean(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_int8() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int8Array::from(vec![1, -3]));
        let result = array
            .to_column::<TestScalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::TinyInt(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_int16() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        let result = array
            .to_column::<TestScalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::SmallInt(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_int32() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3, 42]));
        let result = array
            .to_column::<DoryScalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::Int(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_int64() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        let result = array
            .to_column::<TestScalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::BigInt(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_decimal128() {
        let alloc = Bump::new();
        let decimal_values = vec![
            12_345_678_901_234_567_890_i128,
            -12_345_678_901_234_567_890_i128,
        ];
        let array: ArrayRef = Arc::new(
            Decimal128Array::from(decimal_values)
                .with_precision_and_scale(38, 0)
                .unwrap(),
        );
        let result = array
            .to_column::<DoryScalar>(&alloc, &(0..0), None)
            .unwrap();
        assert_eq!(result, Column::Int128(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_utf8() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(1..1), None)
                .unwrap(),
            Column::VarChar((&[], &[]))
        );
    }

    #[test]
    fn we_cannot_convert_valid_string_array_refs_into_valid_columns_using_out_of_ranges_sizes() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data));
        let result = array.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 3 })
        );
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int8Array::from(vec![1, -3]));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::TinyInt(&[1, -3])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::SmallInt(&[1, -3])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3]));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::Int(&[1, -3])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::BigInt(&[1, -3])
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(core::convert::Into::into).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::VarChar((&data[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_binary_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec![b"cd".as_slice(), b"-f50".as_slice()];
        let scals: Vec<_> = data
            .iter()
            .copied()
            .map(DoryScalar::from_byte_slice_via_hash)
            .collect();
        let array: ArrayRef = Arc::new(arrow::array::BinaryArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::VarBinary((&data[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_boolean_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec![true, false];
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::Boolean(&data[..])
        );
    }

    #[test]
    fn we_can_convert_valid_timestamp_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec![1_625_072_400, 1_625_076_000]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("UTC"),
        ));

        let result = array
            .to_column::<TestScalar>(&alloc, &(0..2), None)
            .unwrap();
        assert_eq!(
            result,
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &data[..])
        );
    }

    #[test]
    fn we_can_convert_valid_boolean_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(vec![true, false, true]));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::Boolean(&[false, true])
        );
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();

        let array: ArrayRef = Arc::new(arrow::array::Int8Array::from(vec![0, 1, 127]));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::TinyInt(&[1, 127])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![0, 1, 545]));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::SmallInt(&[1, 545])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![0, 1, 545]));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::Int(&[1, 545])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![0, 1, 545]));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::BigInt(&[1, 545])
        );
    }

    #[test]
    fn we_can_convert_valid_timestamp_array_refs_into_valid_columns_using_ranges_smaller_than_arrays(
    ) {
        let alloc = Bump::new();
        let data = vec![1_625_072_400, 1_625_076_000, 1_625_083_200]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("Utc"),
        ));

        // Test using a range smaller than the array size
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &data[1..3])
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let data = ["ab", "-f34", "ehfh43"];
        let scals: Vec<_> = data.iter().map(core::convert::Into::into).collect();

        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.to_vec()));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::VarChar((&data[1..3], &scals[1..3]))
        );
    }

    #[test]
    fn we_can_convert_valid_binary_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let data = [b"ab".as_slice(), b"-f34".as_slice(), b"ehfh43".as_slice()];
        let scals: Vec<_> = data
            .iter()
            .copied()
            .map(DoryScalar::from_byte_slice_via_hash)
            .collect();

        let array: ArrayRef = Arc::new(arrow::array::BinaryArray::from(data.to_vec()));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::VarBinary((&data[1..3], &scals[1..3]))
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_precomputed_scalars() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(core::convert::Into::into).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(0..2), Some(&scals))
                .unwrap(),
            Column::VarChar((&data[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_binary_array_refs_into_valid_columns_using_precomputed_scalars() {
        let alloc = Bump::new();
        let data = [b"ab".as_slice(), b"-f34".as_slice()];
        let scals: Vec<_> = data
            .iter()
            .copied()
            .map(TestScalar::from_byte_slice_via_hash)
            .collect();
        let array: ArrayRef = Arc::new(arrow::array::BinaryArray::from(data.to_vec()));
        assert_eq!(
            array
                .to_column::<TestScalar>(&alloc, &(0..2), Some(&scals))
                .unwrap(),
            Column::VarBinary((&data[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_ranges_with_zero_size() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        let result = array
            .to_column::<DoryScalar>(&alloc, &(0..0), None)
            .unwrap();
        assert_eq!(result, Column::VarChar((&[], &[])));
    }

    #[test]
    fn we_can_convert_valid_binary_array_refs_into_valid_columns_using_ranges_with_zero_size() {
        let alloc = Bump::new();
        let data = vec![b"ab".as_slice(), b"-f34".as_slice()];
        let array: ArrayRef = Arc::new(arrow::array::BinaryArray::from(data.clone()));
        let result = array
            .to_column::<DoryScalar>(&alloc, &(0..0), None)
            .unwrap();
        assert_eq!(result, Column::VarBinary((&[], &[])));
    }

    #[test]
    fn we_can_convert_valid_timestamp_array_refs_into_valid_columns_using_ranges_with_zero_size() {
        let alloc = Bump::new();
        let data = vec![1_625_072_400, 1_625_076_000]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("Utc"),
        ));
        let result = array
            .to_column::<DoryScalar>(&alloc, &(0..0), None)
            .unwrap();
        assert_eq!(
            result,
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[])
        );
    }

    #[test]
    fn we_can_convert_array_with_nulls_to_nullable_column() {
        let alloc = Bump::new();
        let array = Int32Array::from(vec![Some(10), None, Some(30)]);
        let array_ref = Arc::new(array) as ArrayRef;

        let range = 0..3;
        let nullable_column = array_ref
            .to_nullable_column::<TestScalar>(&alloc, &range, None)
            .unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        match nullable_column.values {
            Column::Int(values) => {
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 0);
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected Int column"),
        }
    }

    #[test]
    fn we_can_convert_boolean_array_with_nulls_to_nullable_column() {
        let alloc = Bump::new();
        let array = BooleanArray::from(vec![Some(true), None, Some(false)]);
        let array_ref = Arc::new(array) as ArrayRef;

        let range = 0..3;
        let nullable_column = array_ref
            .to_nullable_column::<TestScalar>(&alloc, &range, None)
            .unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        match nullable_column.values {
            Column::Boolean(values) => {
                assert!(values[0]);
                assert!(!values[1]);
                assert!(!values[2]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn we_can_convert_string_array_with_nulls_to_nullable_column() {
        let alloc = Bump::new();
        let array = StringArray::from(vec![Some("hello"), None, Some("world")]);
        let array_ref = Arc::new(array) as ArrayRef;

        let range = 0..3;
        let scalars = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let scalar_slice = alloc.alloc_slice_copy(&scalars);
        let nullable_column = array_ref
            .to_nullable_column::<TestScalar>(&alloc, &range, Some(scalar_slice))
            .unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        match nullable_column.values {
            Column::VarChar((strings, _)) => {
                assert_eq!(strings[0], "hello");
                assert_eq!(strings[1], "");
                assert_eq!(strings[2], "world");
            }
            _ => panic!("Expected VarChar column"),
        }
    }

    #[test]
    fn we_can_convert_array_without_nulls_to_nullable_column() {
        let alloc = Bump::new();
        let array = Int32Array::from(vec![10, 20, 30]);
        let array_ref = Arc::new(array) as ArrayRef;

        let range = 0..3;
        let nullable_column = array_ref
            .to_nullable_column::<TestScalar>(&alloc, &range, None)
            .unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(!nullable_column.is_nullable());

        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }

        match nullable_column.values {
            Column::Int(values) => {
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 20);
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected Int column"),
        }
    }

    #[test]
    fn we_can_convert_subset_of_array_with_nulls() {
        let alloc = Bump::new();
        let array = Int32Array::from(vec![Some(10), None, Some(30), Some(40), None]);
        let array_ref = Arc::new(array) as ArrayRef;
        let range = 1..4;
        let nullable_column = array_ref
            .to_nullable_column::<TestScalar>(&alloc, &range, None)
            .unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(nullable_column.is_null(0)); // NULL at index 1 in original array
        assert!(!nullable_column.is_null(1)); // 30 at index 2 in original array
        assert!(!nullable_column.is_null(2)); // 40 at index 3 in original array

        match nullable_column.values {
            Column::Int(values) => {
                assert_eq!(values[0], 0); // Default value for NULL
                assert_eq!(values[1], 30);
                assert_eq!(values[2], 40);
            }
            _ => panic!("Expected Int column"),
        }
    }

    #[test]
    fn to_nullable_column_correctly_handles_array_with_no_nulls() {
        let alloc = Bump::new();
        let array = Int64Array::from(vec![10, 20, 30]);
        let array_ref = Arc::new(array) as ArrayRef;
        assert_eq!(array_ref.null_count(), 0);

        let range = 0..3;
        let nullable_column = array_ref
            .to_nullable_column::<TestScalar>(&alloc, &range, None)
            .unwrap();

        assert!(!nullable_column.is_nullable());
        assert_eq!(nullable_column.len(), 3);

        match nullable_column.values {
            Column::BigInt(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 20);
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected BigInt column"),
        }
    }

    proptest! {
        #[test]
        fn we_can_roundtrip_arbitrary_column(owned_column: OwnedColumn<TestScalar>) {
            let arrow = ArrayRef::from(owned_column.clone());
            let alloc = Bump::new();
            let column = arrow.to_column::<TestScalar>(&alloc, &(0..arrow.len()), None).unwrap();
            let actual = OwnedColumn::from(&column);

            prop_assert_eq!(actual, owned_column);
        }
    }

    #[test]
    fn we_can_convert_uint8_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(UInt8Array::from(vec![1, 3, 42]));
        let result = array.to_column::<DoryScalar>(&alloc, &(1..3), None);
        assert_eq!(result.unwrap(), Column::Uint8(&[3, 42]));
    }

    #[test]
    fn we_can_convert_uint8_array_empty_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(UInt8Array::from(vec![1, 3, 42]));
        let result = array.to_column::<DoryScalar>(&alloc, &(2..2), None);
        assert_eq!(result.unwrap(), Column::Uint8(&[]));
    }

    #[test]
    fn we_cannot_convert_uint8_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(UInt8Array::from(vec![1, 3, 42]));
        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 3, index: 4 })
        );
    }

    #[test]
    fn we_can_convert_uint8_array_with_nulls() {
        let alloc = Bump::new();
        let array = UInt8Array::from(vec![Some(10), None, Some(30)]);
        let array_ref = Arc::new(array) as ArrayRef;
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(column_result.is_ok());
        let column = column_result.unwrap();
        match column {
            Column::Uint8(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 0); // Default value for NULL
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected Uint8 column"),
        }
    }

    #[test]
    fn binary_array_with_nulls_without_precomputed_scalars_returns_error() {
        let alloc = Bump::new();
        let array = BinaryArray::from(vec![
            Some(b"hello".as_slice()),
            None,
            Some(b"world".as_slice()),
        ]);
        let array_ref = Arc::new(array) as ArrayRef;

        let range = 0..3;
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &range, None);

        assert!(matches!(
            nullable_result,
            Err(ArrowArrayToColumnConversionError::UnsupportedType { datatype })
            if datatype == DataType::Binary
        ));
    }

    #[test]
    fn binary_array_with_nulls_to_column_returns_error_without_precomputed_scalars() {
        let alloc = Bump::new();
        let array = BinaryArray::from(vec![
            Some(b"hello".as_slice()),
            None,
            Some(b"world".as_slice()),
        ]);
        let array_ref = Arc::new(array) as ArrayRef;
        let result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);

        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::UnsupportedType { datatype })
            if datatype == DataType::Binary
        ));
    }

    #[test]
    fn we_can_convert_binary_array_with_nulls_to_nullable_column() {
        let alloc = Bump::new();
        let array = BinaryArray::from(vec![
            Some(b"hello".as_slice()),
            None,
            Some(b"world".as_slice()),
        ]);
        let array_ref = Arc::new(array) as ArrayRef;
        let scalars = [
            TestScalar::from_byte_slice_via_hash(b"hello"),
            TestScalar::from_byte_slice_via_hash(b""), // Default value for NULL
            TestScalar::from_byte_slice_via_hash(b"world"),
        ];
        let scalar_slice = alloc.alloc_slice_copy(&scalars);
        let nullable_result =
            array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), Some(scalar_slice));
        assert!(nullable_result.is_ok());

        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        match &nullable_column.values {
            Column::VarBinary((binary_data, _scalars)) => {
                assert_eq!(binary_data[0], b"hello");
                assert_eq!(binary_data[1], b""); // Default value for NULL
                assert_eq!(binary_data[2], b"world");
            }
            _ => panic!("Expected VarBinary column"),
        }
    }

    #[test]
    fn we_can_convert_binary_array_with_nulls_using_to_column() {
        let alloc = Bump::new();
        let array = BinaryArray::from(vec![
            Some(b"hello".as_slice()),
            None,
            Some(b"world".as_slice()),
        ]);
        let array_ref = Arc::new(array) as ArrayRef;
        let scalars = [
            TestScalar::from_byte_slice_via_hash(b"hello"),
            TestScalar::from_byte_slice_via_hash(b""), // Default value for NULL
            TestScalar::from_byte_slice_via_hash(b"world"),
        ];
        let scalar_slice = alloc.alloc_slice_copy(&scalars);
        let column_result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), Some(scalar_slice));
        assert!(column_result.is_ok());

        let column = column_result.unwrap();
        match column {
            Column::VarBinary((binary_data, scalars)) => {
                assert_eq!(binary_data.len(), 3);
                assert_eq!(binary_data[0], b"hello");
                assert_eq!(binary_data[1], b""); // Default value for NULL
                assert_eq!(binary_data[2], b"world");
                assert_eq!(scalars[0], TestScalar::from_byte_slice_via_hash(b"hello"));
                assert_eq!(scalars[1], TestScalar::from_byte_slice_via_hash(b""));
                assert_eq!(scalars[2], TestScalar::from_byte_slice_via_hash(b"world"));
            }
            _ => panic!("Expected VarBinary column"),
        }
    }

    #[test]
    fn we_can_convert_subset_of_binary_array_with_nulls() {
        let alloc = Bump::new();
        let array = BinaryArray::from(vec![
            Some(b"first".as_slice()),
            Some(b"second".as_slice()),
            None,
            Some(b"fourth".as_slice()),
            None,
        ]);
        let array_ref = Arc::new(array) as ArrayRef;
        let scalars = [
            TestScalar::from_byte_slice_via_hash(b"first"),
            TestScalar::from_byte_slice_via_hash(b"second"),
            TestScalar::from_byte_slice_via_hash(b""), // Default for NULL
            TestScalar::from_byte_slice_via_hash(b"fourth"),
            TestScalar::from_byte_slice_via_hash(b""), // Default for NULL
        ];

        let range = 1..4;
        let scalar_slice = alloc.alloc_slice_copy(&scalars[range.clone()]);

        let nullable_result =
            array_ref.to_nullable_column::<TestScalar>(&alloc, &range, Some(scalar_slice));
        assert!(nullable_result.is_ok());

        let nullable_column = nullable_result.unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0)); // "second" at index 1 in original array
        assert!(nullable_column.is_null(1)); // NULL at index 2 in original array
        assert!(!nullable_column.is_null(2)); // "fourth" at index 3 in original array

        match &nullable_column.values {
            Column::VarBinary((binary_data, slice_scalars)) => {
                assert_eq!(binary_data[0], b"second");
                assert_eq!(binary_data[1], b""); // Default for NULL
                assert_eq!(binary_data[2], b"fourth");
                assert_eq!(slice_scalars.len(), 3);
                assert_eq!(
                    slice_scalars[0],
                    TestScalar::from_byte_slice_via_hash(b"second")
                );
                assert_eq!(slice_scalars[1], TestScalar::from_byte_slice_via_hash(b""));
                assert_eq!(
                    slice_scalars[2],
                    TestScalar::from_byte_slice_via_hash(b"fourth")
                );
            }
            _ => panic!("Expected VarBinary column"),
        }
    }

    #[test]
    fn string_array_with_nulls_without_precomputed_scalars_returns_error() {
        let alloc = Bump::new();
        let array = StringArray::from(vec![Some("hello"), None, Some("world")]);
        let array_ref = Arc::new(array) as ArrayRef;
        let range = 0..3;
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &range, None);
        assert!(matches!(
            nullable_result,
            Err(ArrowArrayToColumnConversionError::UnsupportedType { datatype })
            if datatype == DataType::Utf8
        ));
    }

    #[test]
    fn string_array_with_nulls_to_column_returns_error_without_precomputed_scalars() {
        let alloc = Bump::new();
        let array = StringArray::from(vec![Some("hello"), None, Some("world")]);
        let array_ref = Arc::new(array) as ArrayRef;
        let result = array_ref.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::UnsupportedType { datatype })
            if datatype == DataType::Utf8
        ));
    }

    #[test]
    fn we_can_convert_binary_array_without_precomputed_scalars() {
        let alloc = Bump::new();
        let data = vec![
            b"test1".as_slice(),
            b"test2".as_slice(),
            b"test3".as_slice(),
        ];
        let array: ArrayRef = Arc::new(BinaryArray::from(data.clone()));
        let result = array.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(result.is_ok());

        let column = result.unwrap();
        match column {
            Column::VarBinary((binary_data, scalars)) => {
                assert_eq!(binary_data.len(), 3);
                assert_eq!(binary_data[0], b"test1");
                assert_eq!(binary_data[1], b"test2");
                assert_eq!(binary_data[2], b"test3");
                assert_eq!(scalars.len(), 3);
                assert_eq!(scalars[0], TestScalar::from_byte_slice_via_hash(b"test1"));
                assert_eq!(scalars[1], TestScalar::from_byte_slice_via_hash(b"test2"));
                assert_eq!(scalars[2], TestScalar::from_byte_slice_via_hash(b"test3"));
            }
            _ => panic!("Expected VarBinary column"),
        }
    }

    #[test]
    fn we_can_convert_string_array_without_precomputed_scalars() {
        let alloc = Bump::new();
        let data = vec!["test1", "test2", "test3"];
        let array: ArrayRef = Arc::new(StringArray::from(data.clone()));
        let result = array.to_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(result.is_ok());

        let column = result.unwrap();
        match column {
            Column::VarChar((strings, scalars)) => {
                assert_eq!(strings.len(), 3);
                assert_eq!(strings[0], "test1");
                assert_eq!(strings[1], "test2");
                assert_eq!(strings[2], "test3");
                assert_eq!(scalars.len(), 3);
                assert_eq!(scalars[0], TestScalar::from("test1"));
                assert_eq!(scalars[1], TestScalar::from("test2"));
                assert_eq!(scalars[2], TestScalar::from("test3"));
            }
            _ => panic!("Expected VarChar column"),
        }
    }

    #[test]
    fn we_can_convert_timestamp_array_with_timezone_to_nullable_column() {
        let alloc = Bump::new();
        let timezone = Some("+02:00".to_string());
        let array = TimestampSecondArray::from(vec![100, 200, 300]);
        let array_with_tz = array.with_timezone_opt(timezone);
        let array_ref = Arc::new(array_with_tz) as ArrayRef;
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);
        assert!(nullable_result.is_ok());
        let nullable_column = nullable_result.unwrap();

        match &nullable_column.values {
            Column::TimestampTZ(time_unit, tz, values) => {
                assert_eq!(*time_unit, PoSQLTimeUnit::Second);
                assert_eq!(tz.offset(), 7200); // +02:00 = 2 hours = 7200 seconds
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], 100);
                assert_eq!(values[1], 200);
                assert_eq!(values[2], 300);
            }
            _ => panic!("Expected TimestampTZ column"),
        }
    }

    #[test]
    fn timestamp_array_with_invalid_timezone_returns_error() {
        let alloc = Bump::new();
        let invalid_timezone = Some("INVALID_TZ".to_string());
        let array = TimestampSecondArray::from(vec![100, 200, 300]);
        let array_with_tz = array.with_timezone_opt(invalid_timezone);
        let array_ref = Arc::new(array_with_tz) as ArrayRef;
        let nullable_result = array_ref.to_nullable_column::<TestScalar>(&alloc, &(0..3), None);

        assert!(matches!(
            nullable_result,
            Err(ArrowArrayToColumnConversionError::TimestampConversionError { .. })
        ));

        if let Err(ArrowArrayToColumnConversionError::TimestampConversionError { source }) =
            nullable_result
        {
            assert!(matches!(
                source,
                PoSQLTimestampError::InvalidTimezone { .. }
            ));
        }
    }
}
