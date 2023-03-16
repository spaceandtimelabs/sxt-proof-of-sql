use crate::base::database::Column;
use crate::base::scalar::ToScalar;

use arrow::array::{Array, ArrayRef, Int64Array, StringArray};
use arrow::datatypes::DataType;
use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use std::ops::Range;

/// This trait is used to provide utility functions to convert ArrayRefs into proof types (Column, Scalars, etc.)
pub trait ArrayRefExt {
    /// Convert an ArrayRef into a proofs Vec<Scalar>
    ///
    /// Note: this function must not be called from unsupported arrays or arrays with nulls.
    fn to_scalars(&self) -> Vec<Scalar>;

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
    fn to_column<'a>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        scals: Option<&'a [Scalar]>,
    ) -> Column<'a>;
}

impl ArrayRefExt for ArrayRef {
    fn to_scalars(&self) -> Vec<Scalar> {
        assert!(self.null_count() == 0);

        match self.data_type() {
            DataType::Int64 => self
                .as_any()
                .downcast_ref::<Int64Array>()
                .map(|array| array.values().iter().map(|v| v.to_scalar()).collect())
                .unwrap(),
            DataType::Utf8 => self
                .as_any()
                .downcast_ref::<StringArray>()
                .map(|array| {
                    array
                        .iter()
                        .map(|v| v.expect("null elements are invalid").to_scalar())
                        .collect()
                })
                .unwrap(),
            _ => unimplemented!(),
        }
    }

    fn to_column<'a>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        precomputed_scals: Option<&'a [Scalar]>,
    ) -> Column<'a> {
        assert!(self.null_count() == 0);

        match self.data_type() {
            DataType::Int64 => Column::BigInt(
                &self
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .map(|array| array.values())
                    .unwrap()[range.start..range.end],
            ),
            DataType::Utf8 => {
                let vals = self
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .map(|array| {
                        alloc.alloc_slice_fill_with(range.end - range.start, |i| -> &'a [u8] {
                            array.value(range.start + i).as_bytes()
                        })
                    })
                    .unwrap();

                let scals = if let Some(scals) = precomputed_scals {
                    &scals[range.start..range.end]
                } else {
                    // This `else` is just to simplify implementations at higher code levels.
                    // However, as the caller can always pass the correct scalar slice,
                    // this convenience `else` here may be dropped in the future.
                    alloc.alloc_slice_fill_with(vals.len(), |i| -> Scalar { vals[i].to_scalar() })
                };

                Column::HashedBytes((vals, scals))
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![1, -3]));
        assert_eq!(
            array.to_column(&alloc, &(0..2), None),
            Column::BigInt(&[1, -3])
        );

        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(Vec::<i64>::new()));
        assert_eq!(array.to_column(&alloc, &(0..0), None), Column::BigInt(&[]));
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(|v| v.to_scalar()).collect();
        let data_slices: Vec<_> = data.iter().map(|v| v.as_bytes()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data));
        assert_eq!(
            array.to_column(&alloc, &(0..2), None),
            Column::HashedBytes((&data_slices[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(vec![0, 1, 545]));
        assert_eq!(
            array.to_column(&alloc, &(1..3), None),
            Column::BigInt(&[1, 545])
        );
        assert_eq!(array.to_column(&alloc, &(0..0), None), Column::BigInt(&[]));
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_ranges_smaller_than_arrays()
    {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34", "ehfh43"];
        let scals: Vec<_> = data.iter().map(|v| v.to_scalar()).collect();
        let data_slices: Vec<_> = data.iter().map(|v| v.as_bytes()).collect();

        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.to_vec()));
        assert_eq!(
            array.to_column(&alloc, &(1..3), None),
            Column::HashedBytes((&data_slices[1..3], &scals[1..3]))
        );
        assert_eq!(
            array.to_column(&alloc, &(0..0), None),
            Column::HashedBytes((&[], &[]))
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_precomputed_scalars() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(|v| v.to_scalar()).collect();
        let data_slices: Vec<_> = data.iter().map(|v| v.as_bytes()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data));
        assert_eq!(
            array.to_column(&alloc, &(0..2), Some(&scals)),
            Column::HashedBytes((&data_slices[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_ranges_with_zero_size() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(|v| v.to_scalar()).collect();
        let data_slices: Vec<_> = data.iter().map(|v| v.as_bytes()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data));
        assert_eq!(
            array.to_column(&alloc, &(0..0), None),
            Column::HashedBytes((&data_slices[0..0], &scals[0..0]))
        );
    }

    #[test]
    #[should_panic]
    fn we_cannot_convert_valid_string_array_refs_into_valid_columns_using_out_of_ranges_sizes() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data));
        array.to_column(&alloc, &(0..3), None);
    }

    #[test]
    fn we_can_convert_valid_integer_array_refs_into_valid_vec_scalars() {
        let data = vec![1, -3];
        let array: ArrayRef = Arc::new(arrow::array::Int64Array::from(data.clone()));
        assert_eq!(
            array.to_scalars(),
            data.iter().map(|v| v.to_scalar()).collect::<Vec<Scalar>>()
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_vec_scalars() {
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array.to_scalars(),
            data.iter().map(|v| v.to_scalar()).collect::<Vec<Scalar>>()
        );
    }
}
