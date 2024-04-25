use super::scalar_and_i256_conversions::convert_i256_to_scalar;
use crate::base::{
    database::Column,
    math::decimal::Precision,
    scalar::{Curve25519Scalar, Scalar},
};
use arrow::{
    array::{
        Array, ArrayRef, BooleanArray, Decimal128Array, Decimal256Array, Int64Array, StringArray,
    },
    datatypes::{i256, DataType},
};
use bumpalo::Bump;
use std::ops::Range;
use thiserror::Error;

#[derive(Error, Debug)]
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
}

/// This trait is used to provide utility functions to convert ArrayRefs into proof types (Column, Scalars, etc.)
pub trait ArrayRefExt {
    /// Convert an ArrayRef into a proofs Vec<Scalar>
    ///
    /// Note: this function must not be called from unsupported arrays or arrays with nulls.
    fn to_curve25519_scalars(&self) -> Vec<Curve25519Scalar>;

    /// Convert an ArrayRef into a proofs Column type
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
    ///TODO: This needs to return Result one day
    fn to_curve25519_scalars(&self) -> Vec<Curve25519Scalar> {
        assert!(self.null_count() == 0);

        match self.data_type() {
            DataType::Boolean => self
                .as_any()
                .downcast_ref::<BooleanArray>()
                .map(|array| array.iter().map(|v| v.unwrap().into()).collect())
                .unwrap(),
            DataType::Int64 => self
                .as_any()
                .downcast_ref::<Int64Array>()
                .map(|array| array.values().iter().map(|v| v.into()).collect())
                .unwrap(),
            DataType::Decimal128(38, 0) => self
                .as_any()
                .downcast_ref::<Decimal128Array>()
                .map(|array| array.values().iter().map(|v| v.into()).collect())
                .unwrap(),
            DataType::Decimal256(_, _) => self
                .as_any()
                .downcast_ref::<Decimal256Array>()
                .map(|array| {
                    array
                        .values()
                        .iter()
                        .map(|v| convert_i256_to_scalar(v).unwrap())
                        .collect()
                })
                .unwrap(),
            DataType::Utf8 => self
                .as_any()
                .downcast_ref::<StringArray>()
                .map(|array| {
                    array
                        .iter()
                        .map(|v| v.expect("null elements are invalid").into())
                        .collect()
                })
                .unwrap(),
            _ => unimplemented!(),
        }
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
        // Match supported types and attempt conversion
        match self.data_type() {
            DataType::Boolean => {
                let array = self.as_any().downcast_ref::<BooleanArray>().expect(
                    "Failed to downcast to BooleanArray in arrow_array_to_column_conversion",
                );
                let boolean_slice = &array
                    .iter()
                    .skip(range.start)
                    .take(range.len())
                    .collect::<Option<Vec<bool>>>()
                    .ok_or(ArrowArrayToColumnConversionError::ArrayContainsNulls)?;
                let values = alloc.alloc_slice_fill_with(range.len(), |i| boolean_slice[i]);

                Ok(Column::Boolean(values))
            }
            DataType::Int64 => {
                let array = self
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .expect("Failed to downcast to Int64Array in arrow_array_to_column_conversion");

                Ok(Column::BigInt(&array.values()[range.start..range.end]))
            }
            DataType::Decimal128(38, 0) => {
                let array = self.as_any().downcast_ref::<Decimal128Array>().expect(
                    "Failed to downcast to Decimal128Array in arrow_array_to_column_conversion",
                );

                Ok(Column::Int128(&array.values()[range.start..range.end]))
            }
            DataType::Decimal256(precision, scale) if *precision <= 75 => {
                let array = self.as_any().downcast_ref::<Decimal256Array>().expect(
                    "Failed to downcast to Decimal256Array in arrow_array_to_column_conversion",
                );

                let i256_slice = &array.values()[range.start..range.end];
                let curve25519_scalars = alloc.alloc_slice_fill_default(i256_slice.len());
                for (scalar, value) in curve25519_scalars.iter_mut().zip(i256_slice) {
                    *scalar = convert_i256_to_scalar(value).ok_or(
                        ArrowArrayToColumnConversionError::DecimalConversionFailed(*value),
                    )?;
                }

                Ok(Column::Decimal75(
                    Precision::new(*precision).unwrap(),
                    *scale,
                    curve25519_scalars,
                ))
            }
            DataType::Utf8 => {
                let array = self.as_any().downcast_ref::<StringArray>().expect(
                    "Failed to downcast to StringArray in arrow_array_to_column_conversion",
                );

                let vals = alloc.alloc_slice_fill_with(range.end - range.start, |i| -> &'a str {
                    array.value(range.start + i)
                });

                let scals = if let Some(scals) = precomputed_scals {
                    &scals[range.start..range.end]
                } else {
                    alloc.alloc_slice_fill_with(vals.len(), |i| -> S { vals[i].into() })
                };

                Ok(Column::VarChar((vals, scals)))
            }
            data_type => Err(ArrowArrayToColumnConversionError::UnsupportedType(
                data_type.clone(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    #[should_panic(expected = "range end index 3 out of range for slice of length 2")]
    fn we_cannot_index_on_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        array
            .to_column::<Curve25519Scalar>(&alloc, &(2..3), None)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "range end index 5 out of range for slice of length 2")]
    fn we_cannot_index_on_empty_oob_range() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        array
            .to_column::<Curve25519Scalar>(&alloc, &(5..5), None)
            .unwrap();
    }

    #[test]
    fn we_can_build_an_empty_column_from_an_empty_range_boolean() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(vec![true, false]));
        let result = array
            .to_column::<Curve25519Scalar>(&alloc, &(2..2), None)
            .unwrap();
        assert_eq!(result, Column::Boolean(&[]));
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
            .to_column::<Curve25519Scalar>(&alloc, &(0..0), None)
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
        let result = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
        assert!(matches!(
            result,
            Err(ArrowArrayToColumnConversionError::ArrayContainsNulls)
        ));
    }

    #[test]
    #[should_panic]
    fn we_cannot_convert_valid_string_array_refs_into_valid_columns_using_out_of_ranges_sizes() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data));
        let _ = array.to_column::<Curve25519Scalar>(&alloc, &(0..3), None);
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_columns() {
        let alloc = Bump::new();
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
                .to_column::<Curve25519Scalar>(&alloc, &(0..2), None)
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
    fn we_can_convert_valid_boolean_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(vec![true, false, true]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::Boolean(&[false, true])
        );
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![0, 1, 545]));
        assert_eq!(
            array
                .to_column::<Curve25519Scalar>(&alloc, &(1..3), None)
                .unwrap(),
            Column::BigInt(&[1, 545])
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
                .to_column::<Curve25519Scalar>(&alloc, &(1..3), None)
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
            .to_column::<Curve25519Scalar>(&alloc, &(0..0), None)
            .unwrap();
        assert_eq!(result, Column::VarChar((&[], &[])));
    }

    #[test]
    fn we_can_convert_valid_boolean_array_refs_into_valid_vec_scalars() {
        let data = vec![false, true];
        let array: ArrayRef = Arc::new(arrow::array::BooleanArray::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            data.iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>()
        );
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_vec_scalars() {
        let data = vec![1, -3];
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            data.iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>()
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_vec_scalars() {
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array.to_curve25519_scalars(),
            data.iter()
                .map(|v| v.into())
                .collect::<Vec<Curve25519Scalar>>()
        );
    }
}
