use super::scalar_and_i256_conversions::convert_i256_to_scalar;
use crate::{
    base::{database::Column, math::decimal::Precision, scalar::Scalar},
    sql::parse::ConversionError,
};
use arrow::{
    array::{
        Array, ArrayRef, BooleanArray, Decimal128Array, Decimal256Array, Int16Array, Int32Array,
        Int64Array, StringArray, TimestampMicrosecondArray, TimestampMillisecondArray,
        TimestampNanosecondArray, TimestampSecondArray,
    },
    datatypes::{i256, DataType, TimeUnit as ArrowTimeUnit},
};
use bumpalo::Bump;
use proof_of_sql_parser::intermediate_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestampError};
use std::ops::Range;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
/// Errors caused by conversions between Arrow and owned types.
pub enum ArrowArrayToColumnConversionError {
    /// This error occurs when an array contains a non-zero number of null elements
    #[error("arrow array must not contain nulls")]
    ArrayContainsNulls,
    /// This error occurs when trying to convert from an unsupported arrow type.
    #[error("unsupported type: attempted conversion from ArrayRef of type {0} to OwnedColumn")]
    UnsupportedType(DataType),
    /// This error occurs when trying to convert from an i256 to a Scalar.
    #[error("decimal conversion failed: {0}")]
    DecimalConversionFailed(i256),
    /// This error occurs when the specified range is out of the bounds of the array.
    #[error("index out of bounds: the len is {0} but the index is {1}")]
    IndexOutOfBounds(usize, usize),
    /// Variant for conversion errors
    #[error("conversion error: {0}")]
    ConversionError(#[from] ConversionError),
    /// Using TimeError to handle all time-related errors
    #[error(transparent)]
    TimestampConversionError(#[from] PoSQLTimestampError),
}

/// This trait is used to provide utility functions to convert ArrayRefs into proof types (Column, Scalars, etc.)
pub trait ArrayRefExt {
    /// Convert an ArrayRef into a Proof of SQL Vec<Scalar>
    ///
    /// Note: this function must not be called from unsupported arrays or arrays with nulls.
    /// It should only be used during testing.
    #[cfg(any(test, feature = "test"))]
    #[cfg(feature = "blitzar")]
    fn to_curve25519_scalars(
        &self,
    ) -> Result<Vec<crate::base::scalar::Curve25519Scalar>, ArrowArrayToColumnConversionError>;

    /// Convert an ArrayRef into a Proof of SQL Column type
    ///
    /// Parameters:
    /// - `alloc`: used to allocate a slice of data when necessary
    ///    (vide StringArray into Column::HashedBytes((_,_)).
    ///
    /// - `range`: used to get a subslice out of ArrayRef.
    ///
    /// - `scals`: scalar representation of each element in the ArrayRef.
    ///    Some types don't require this slice (see Column::BigInt). But for types requiring it,
    ///    `scals` must be provided and have a length equal to `range.len()`.
    ///
    /// Note: this function must not be called from unsupported or nullable arrays as it will panic.
    fn to_column<'a, S: Scalar>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        scals: Option<&'a [S]>,
    ) -> Result<Column<'a, S>, ArrowArrayToColumnConversionError>;
}

impl ArrayRefExt for ArrayRef {
    #[cfg(any(test, feature = "test"))]
    #[cfg(feature = "blitzar")]
    fn to_curve25519_scalars(
        &self,
    ) -> Result<Vec<crate::base::scalar::Curve25519Scalar>, ArrowArrayToColumnConversionError> {
        if self.null_count() != 0 {
            return Err(ArrowArrayToColumnConversionError::ArrayContainsNulls);
        }

        let result = match self.data_type() {
            DataType::Boolean => self.as_any().downcast_ref::<BooleanArray>().map(|array| {
                array
                    .iter()
                    .map(|v| {
                        v.ok_or(ArrowArrayToColumnConversionError::ArrayContainsNulls)
                            .map(Into::into)
                    })
                    .collect()
            }),
            DataType::Int16 => self
                .as_any()
                .downcast_ref::<Int16Array>()
                .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
            DataType::Int32 => self
                .as_any()
                .downcast_ref::<Int32Array>()
                .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
            DataType::Int64 => self
                .as_any()
                .downcast_ref::<Int64Array>()
                .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
            DataType::Decimal128(38, 0) => self
                .as_any()
                .downcast_ref::<Decimal128Array>()
                .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
            DataType::Decimal256(_, _) => {
                self.as_any()
                    .downcast_ref::<Decimal256Array>()
                    .map(|array| {
                        array
                            .values()
                            .iter()
                            .map(|v| {
                                convert_i256_to_scalar(v).ok_or(
                                    ArrowArrayToColumnConversionError::DecimalConversionFailed(*v),
                                )
                            })
                            .collect()
                    })
            }
            DataType::Utf8 => self.as_any().downcast_ref::<StringArray>().map(|array| {
                array
                    .iter()
                    .map(|v| {
                        v.ok_or(ArrowArrayToColumnConversionError::ArrayContainsNulls)
                            .map(Into::into)
                    })
                    .collect()
            }),
            DataType::Timestamp(time_unit, _) => match time_unit {
                ArrowTimeUnit::Second => self
                    .as_any()
                    .downcast_ref::<TimestampSecondArray>()
                    .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
                ArrowTimeUnit::Millisecond => self
                    .as_any()
                    .downcast_ref::<TimestampMillisecondArray>()
                    .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
                ArrowTimeUnit::Microsecond => self
                    .as_any()
                    .downcast_ref::<TimestampMicrosecondArray>()
                    .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
                ArrowTimeUnit::Nanosecond => self
                    .as_any()
                    .downcast_ref::<TimestampNanosecondArray>()
                    .map(|array| array.values().iter().map(|v| Ok((*v).into())).collect()),
            },
            _ => None,
        };

        result.unwrap_or_else(|| {
            Err(ArrowArrayToColumnConversionError::UnsupportedType(
                self.data_type().clone(),
            ))
        })
    }

    /// Converts the given ArrowArray into a `Column` data type based on its `DataType`. Returns an
    /// empty `Column` for any empty tange if it is in-bounds.
    ///
    /// # Parameters
    /// - `alloc`: Reference to a `Bump` allocator used for memory allocation during the conversion.
    /// - `range`: Reference to a `Range<usize>` specifying the slice of the array to convert.
    /// - `precomputed_scals`: Optional reference to a slice of `Curve25519Scalar` values.
    ///    VarChar columns store hashes to their values as scalars, which can be provided here.
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
    fn to_column<'a, S: Scalar>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        precomputed_scals: Option<&'a [S]>,
    ) -> Result<Column<'a, S>, ArrowArrayToColumnConversionError> {
        // Start by checking for nulls
        if self.null_count() != 0 {
            return Err(ArrowArrayToColumnConversionError::ArrayContainsNulls);
        }

        // Before performing any operations, check if the range is out of bounds
        if range.end > self.len() {
            return Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(
                self.len(),
                range.end,
            ));
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
                        .ok_or(ArrowArrayToColumnConversionError::ArrayContainsNulls)?;
                    let values = alloc.alloc_slice_fill_with(range.len(), |i| boolean_slice[i]);
                    Ok(Column::Boolean(values))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType(
                        self.data_type().clone(),
                    ))
                }
            }
            DataType::Int16 => {
                if let Some(array) = self.as_any().downcast_ref::<Int16Array>() {
                    Ok(Column::SmallInt(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType(
                        self.data_type().clone(),
                    ))
                }
            }
            DataType::Int32 => {
                if let Some(array) = self.as_any().downcast_ref::<Int32Array>() {
                    Ok(Column::Int(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType(
                        self.data_type().clone(),
                    ))
                }
            }
            DataType::Int64 => {
                if let Some(array) = self.as_any().downcast_ref::<Int64Array>() {
                    Ok(Column::BigInt(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType(
                        self.data_type().clone(),
                    ))
                }
            }
            DataType::Decimal128(38, 0) => {
                if let Some(array) = self.as_any().downcast_ref::<Decimal128Array>() {
                    Ok(Column::Int128(&array.values()[range.start..range.end]))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType(
                        self.data_type().clone(),
                    ))
                }
            }
            DataType::Decimal256(precision, scale) if *precision <= 75 => {
                if let Some(array) = self.as_any().downcast_ref::<Decimal256Array>() {
                    let i256_slice = &array.values()[range.start..range.end];
                    let scalars = alloc.alloc_slice_fill_default(i256_slice.len());
                    for (scalar, value) in scalars.iter_mut().zip(i256_slice) {
                        *scalar = convert_i256_to_scalar(value).ok_or(
                            ArrowArrayToColumnConversionError::DecimalConversionFailed(*value),
                        )?;
                    }
                    Ok(Column::Decimal75(
                        Precision::new(*precision)?,
                        *scale,
                        scalars,
                    ))
                } else {
                    Err(ArrowArrayToColumnConversionError::UnsupportedType(
                        self.data_type().clone(),
                    ))
                }
            }
            // Handle all possible TimeStamp TimeUnit instances
            DataType::Timestamp(time_unit, tz) => match time_unit {
                ArrowTimeUnit::Second => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampSecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Second,
                            PoSQLTimeZone::try_from(tz.clone())?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType(
                            self.data_type().clone(),
                        ))
                    }
                }
                ArrowTimeUnit::Millisecond => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampMillisecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Millisecond,
                            PoSQLTimeZone::try_from(tz.clone())?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType(
                            self.data_type().clone(),
                        ))
                    }
                }
                ArrowTimeUnit::Microsecond => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampMicrosecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Microsecond,
                            PoSQLTimeZone::try_from(tz.clone())?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType(
                            self.data_type().clone(),
                        ))
                    }
                }
                ArrowTimeUnit::Nanosecond => {
                    if let Some(array) = self.as_any().downcast_ref::<TimestampNanosecondArray>() {
                        Ok(Column::TimestampTZ(
                            PoSQLTimeUnit::Nanosecond,
                            PoSQLTimeZone::try_from(tz.clone())?,
                            &array.values()[range.start..range.end],
                        ))
                    } else {
                        Err(ArrowArrayToColumnConversionError::UnsupportedType(
                            self.data_type().clone(),
                        ))
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
                    Err(ArrowArrayToColumnConversionError::UnsupportedType(
                        self.data_type().clone(),
                    ))
                }
            }
            data_type => Err(ArrowArrayToColumnConversionError::UnsupportedType(
                data_type.clone(),
            )),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "blitzar")]
mod tests {

    use super::*;
    use crate::{base::scalar::Curve25519Scalar, proof_primitive::dory::DoryScalar};
    use arrow::array::Decimal256Builder;
    use std::{str::FromStr, sync::Arc};

    #[test]
    fn we_can_convert_timestamp_array_normal_range() {
        let alloc = Bump::new();
        let data = vec![1625072400, 1625076000, 1625083200]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("UTC"),
        ));

        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..3), None);
        assert_eq!(
            result.unwrap(),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::UTC, &data[1..3])
        );
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_timestamp() {
        let alloc = Bump::new();
        let data = vec![1625072400, 1625076000]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.into(),
            Some("UTC"),
        ));

        let result = array
            .to_column::<DoryScalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(
            result,
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::UTC, &[])
        );
    }

    #[test]
    fn we_can_convert_timestamp_array_empty_range() {
        let alloc = Bump::new();
        let data = vec![1625072400, 1625076000, 1625083200]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.into(),
            Some("UTC"),
        ));

        let result = array.to_column::<DoryScalar>(&alloc, &(1..1), None);
        assert_eq!(
            result.unwrap(),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::UTC, &[])
        );
    }

    #[test]
    fn we_cannot_convert_timestamp_array_oob_range() {
        let alloc = Bump::new();
        let data = vec![1625072400, 1625076000, 1625083200];
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.into(),
            Some("UTC"),
        ));

        let result = array.to_column::<Curve25519Scalar>(&alloc, &(3..5), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(3, 5))
        );
    }

    #[test]
    fn we_can_convert_timestamp_array_with_nulls() {
        let alloc = Bump::new();
        let data = vec![Some(1625072400), None, Some(1625083200)];
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.into(),
            Some("UTC"),
        ));

        let result = array.to_column::<DoryScalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
    }

    #[test]
    fn we_cannot_convert_utf8_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(StringArray::from(vec!["hello", "world", "test"]));
        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(3, 4))
        );
    }

    #[test]
    fn we_can_convert_utf8_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(StringArray::from(vec!["hello", "world", "test"]));
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..3), None);
        let expected_vals = vec!["world", "test"];
        let expected_scals: Vec<Curve25519Scalar> =
            expected_vals.iter().map(|&v| v.into()).collect();

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
        let array: ArrayRef = Arc::new(StringArray::from(vec![Some("hello"), None, Some("test")]));
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
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
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..3), None);
        assert!(result.is_err())
    }

    #[test]
    fn we_can_convert_decimal256_array_normal_range() {
        let alloc = Bump::new();
        let mut builder = Decimal256Builder::with_capacity(3);
        builder.append_value(i256::from_str("100000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("-300000000000000000000000000000000000000").unwrap());
        builder.append_value(i256::from_str("4200000000000000000000000000000000000000").unwrap());
        let array: ArrayRef = Arc::new(builder.finish().with_precision_and_scale(75, 0).unwrap());

        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..3), None);
        let expected_scalars: Vec<Curve25519Scalar> = vec![
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

        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..1), None);
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
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(3, 4))
        );
    }

    #[test]
    fn we_can_convert_decimal256_array_with_nulls() {
        let alloc = Bump::new();
        let mut builder = Decimal256Builder::with_capacity(3);
        builder.append_value(i256::from_str("100000000000000000000000000000000000000").unwrap());
        builder.append_null();
        builder.append_value(i256::from_str("4200000000000000000000000000000000000000").unwrap());
        let array: ArrayRef = Arc::new(builder.finish().with_precision_and_scale(75, 0).unwrap());

        let result = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
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

        let result = array.to_column::<Curve25519Scalar>(&alloc, &(2..4), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(3, 4))
        );
    }

    #[test]
    fn we_can_convert_decimal128_array_with_nulls() {
        let alloc = Bump::new();
        let data = vec![Some(100_i128), None, Some(4200_i128)];
        let array: ArrayRef = Arc::new(
            Decimal128Array::from(data.clone())
                .with_precision_and_scale(38, 0)
                .unwrap(),
        );

        let result = array.to_column::<DoryScalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
    }

    #[test]
    fn we_can_convert_decimal128_array_normal_range() {
        let alloc = Bump::new();
        let data = vec![100_i128, -300_i128, 4200_i128];
        let array: ArrayRef = Arc::new(
            Decimal128Array::from_iter_values(data.clone())
                .with_precision_and_scale(38, 0)
                .unwrap(),
        );

        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..3), None);
        assert_eq!(result.unwrap(), Column::Int128(&data[1..3]));
    }

    #[test]
    fn we_can_we_can_convert_boolean_array_normal_range() {
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
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..1), None);
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
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(3, 4))
        );
    }

    #[test]
    fn we_can_convert_boolean_array_with_nulls() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(BooleanArray::from(vec![Some(true), None, Some(true)]));
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
    }

    #[test]
    fn we_can_convert_int16_array_normal_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int16Array::from(vec![1, -3, 42]));
        let result = array.to_column::<DoryScalar>(&alloc, &(1..3), None);
        assert_eq!(result.unwrap(), Column::SmallInt(&[-3, 42]));
    }

    #[test]
    fn we_can_convert_int16_array_empty_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int16Array::from(vec![1, -3, 42]));
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::SmallInt(&[]));
    }

    #[test]
    fn we_cannot_convert_int16_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int16Array::from(vec![1, -3, 42]));

        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);

        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(3, 4))
        );
    }

    #[test]
    fn we_can_convert_int16_array_with_nulls() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int16Array::from(vec![Some(1), None, Some(42)]));
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
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
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(1..1), None);
        assert_eq!(result.unwrap(), Column::Int(&[]));
    }

    #[test]
    fn we_cannot_convert_int32_array_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int32Array::from(vec![1, -3, 42]));

        let result = array.to_column::<DoryScalar>(&alloc, &(2..4), None);

        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(3, 4))
        );
    }

    #[test]
    fn we_can_convert_int32_array_with_nulls() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(Int32Array::from(vec![Some(1), None, Some(42)]));
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
    }

    #[test]
    fn we_cannot_index_on_oob_range() {
        let alloc = Bump::new();

        let array1: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        let result1 = array1.to_column::<DoryScalar>(&alloc, &(2..3), None);
        assert_eq!(
            result1,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(2, 3))
        );

        let array2: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3]));
        let result2 = array2.to_column::<DoryScalar>(&alloc, &(2..3), None);
        assert_eq!(
            result2,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(2, 3))
        );

        let array3: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        let result3 = array3.to_column::<DoryScalar>(&alloc, &(2..3), None);
        assert_eq!(
            result3,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(2, 3))
        );
    }

    #[test]
    fn we_cannot_index_on_empty_oob_range() {
        let alloc = Bump::new();

        let array1: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        let result1 = array1.to_column::<Curve25519Scalar>(&alloc, &(5..5), None);
        assert_eq!(
            result1,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(2, 5))
        );

        let array2: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3]));
        let result2 = array2.to_column::<DoryScalar>(&alloc, &(5..5), None);
        assert_eq!(
            result2,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(2, 5))
        );

        let array3: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        let result3 = array3.to_column::<Curve25519Scalar>(&alloc, &(5..5), None);
        assert_eq!(
            result3,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(2, 5))
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
    fn we_can_build_an_empty_column_from_an_empty_range_int16() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        let result = array
            .to_column::<Curve25519Scalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::SmallInt(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_int32() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3]));
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
            .to_column::<Curve25519Scalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::BigInt(&[]));
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_decimal128() {
        let alloc = Bump::new();
        let decimal_values = vec![12345678901234567890_i128, -12345678901234567890_i128];
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
                .to_column::<Curve25519Scalar>(&alloc, &(1..1), None)
                .unwrap(),
            Column::VarChar((&[], &[]))
        );
    }

    #[test]
    fn we_cannot_build_a_column_from_an_array_with_nulls_utf8() {
        let alloc = Bump::new();
        let data = vec![Some("ab"), Some("-f34"), None];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        let result = array.to_column::<DoryScalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
    }

    #[test]
    fn we_cannot_convert_valid_string_array_refs_into_valid_columns_using_out_of_ranges_sizes() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data));
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
        assert_eq!(
            result,
            Err(ArrowArrayToColumnConversionError::IndexOutOfBounds(2, 3))
        );
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![1, -3]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::SmallInt(&[1, -3])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![1, -3]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::Int(&[1, -3])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::BigInt(&[1, -3])
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(|v| v.into()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::VarChar((&data[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_boolean_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec![true, false];
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(0..2), None)
                .unwrap(),
            Column::Boolean(&data[..])
        );
    }

    #[test]
    fn we_can_convert_valid_timestamp_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec![1625072400, 1625076000]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("UTC"),
        ));

        let result = array
            .to_column::<Curve25519Scalar>(&alloc, &(0..2), None)
            .unwrap();
        assert_eq!(
            result,
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::UTC, &data[..])
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

        let array: ArrayRef = Arc::new(arrow::array::Int16Array::from(vec![0, 1, 545]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::SmallInt(&[1, 545])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int32Array::from(vec![0, 1, 545]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::Int(&[1, 545])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![0, 1, 545]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::BigInt(&[1, 545])
        );
    }

    #[test]
    fn we_can_convert_valid_timestamp_array_refs_into_valid_columns_using_ranges_smaller_than_arrays(
    ) {
        let alloc = Bump::new();
        let data = vec![1625072400, 1625076000, 1625083200]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("UTC"),
        ));

        // Test using a range smaller than the array size
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::UTC, &data[1..3])
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let data = ["ab", "-f34", "ehfh43"];
        let scals: Vec<_> = data.iter().map(|v| v.into()).collect();

        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.to_vec()));
        assert_eq!(
            array
                .to_column::<DoryScalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::VarChar((&data[1..3], &scals[1..3]))
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_precomputed_scalars() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(|v| v.into()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(0..2), Some(&scals))
                .unwrap(),
            Column::VarChar((&data[..], &scals[..]))
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
    fn we_can_convert_valid_timestamp_array_refs_into_valid_columns_using_ranges_with_zero_size() {
        let alloc = Bump::new();
        let data = vec![1625072400, 1625076000]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("UTC"),
        ));
        let result = array
            .to_column::<DoryScalar>(&alloc, &(0..0), None)
            .unwrap();
        assert_eq!(
            result,
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::UTC, &[])
        );
    }

    #[test]
    fn we_can_convert_valid_boolean_array_refs_into_valid_vec_scalars() {
        let data = vec![false, true];
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            Ok(data
                .iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>())
        );
    }

    #[test]
    fn we_can_convert_valid_timestamp_array_refs_into_valid_vec_scalars() {
        let data = vec![1625072400, 1625076000]; // Example Unix timestamps
        let array: ArrayRef = Arc::new(TimestampSecondArray::with_timezone_opt(
            data.clone().into(),
            Some("UTC"),
        ));

        assert_eq!(
            array.to_curve25519_scalars(),
            Ok(data
                .iter()
                .map(|&v| Curve25519Scalar::from(v))
                .collect::<Vec<Curve25519Scalar>>())
        );
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_vec_scalars() {
        let data = vec![1, -3];

        let array: ArrayRef = Arc::new(Int16Array::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            Ok(data
                .iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>())
        );

        let data = vec![1, -3];
        let array: ArrayRef = Arc::new(Int32Array::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            Ok(data
                .iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>())
        );

        let data = vec![1, -3];
        let array: ArrayRef = Arc::new(Int64Array::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            Ok(data
                .iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>())
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_vec_scalars() {
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            Ok(data
                .iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>())
        );
    }
}
