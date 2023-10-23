use crate::base::{database::Column, scalar::ArkScalar};
use arrow::{
    array::{Array, ArrayRef, Decimal128Array, Int64Array, StringArray},
    datatypes::DataType,
};
use bumpalo::Bump;
use std::ops::Range;

/// This trait is used to provide utility functions to convert ArrayRefs into proof types (Column, Scalars, etc.)
pub trait ArrayRefExt {
    /// Convert an ArrayRef into a proofs Vec<Scalar>
    ///
    /// Note: this function must not be called from unsupported arrays or arrays with nulls.
    fn to_ark_scalars(&self) -> Vec<ArkScalar>;

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
        scals: Option<&'a [ArkScalar]>,
    ) -> Column<'a>;
}

impl ArrayRefExt for ArrayRef {
    fn to_ark_scalars(&self) -> Vec<ArkScalar> {
        assert!(self.null_count() == 0);

        match self.data_type() {
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

    fn to_column<'a>(
        &'a self,
        alloc: &'a Bump,
        range: &Range<usize>,
        precomputed_scals: Option<&'a [ArkScalar]>,
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
            DataType::Decimal128(38, 0) => Column::Int128(
                &self
                    .as_any()
                    .downcast_ref::<Decimal128Array>()
                    .map(|array| array.values())
                    .unwrap()[range.start..range.end],
            ),
            DataType::Utf8 => {
                let vals = self
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .map(|array| {
                        alloc.alloc_slice_fill_with(range.end - range.start, |i| -> &'a str {
                            array.value(range.start + i)
                        })
                    })
                    .unwrap();

                let scals = if let Some(scals) = precomputed_scals {
                    &scals[range.start..range.end]
                } else {
                    // This `else` is just to simplify implementations at higher code levels.
                    // However, as the caller can always pass the correct scalar slice,
                    // this convenience `else` here may be dropped in the future.
                    alloc.alloc_slice_fill_with(vals.len(), |i| -> ArkScalar { vals[i].into() })
                };

                Column::VarChar((vals, scals))
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
        let scals: Vec<_> = data.iter().map(|v| v.into()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array.to_column(&alloc, &(0..2), None),
            Column::VarChar((&data[..], &scals[..]))
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
        let data = ["ab", "-f34", "ehfh43"];
        let scals: Vec<_> = data.iter().map(|v| v.into()).collect();

        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.to_vec()));
        assert_eq!(
            array.to_column(&alloc, &(1..3), None),
            Column::VarChar((&data[1..3], &scals[1..3]))
        );
        assert_eq!(
            array.to_column(&alloc, &(0..0), None),
            Column::VarChar((&[], &[]))
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_precomputed_scalars() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(|v| v.into()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array.to_column(&alloc, &(0..2), Some(&scals)),
            Column::VarChar((&data[..], &scals[..]))
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_columns_using_ranges_with_zero_size() {
        let alloc = Bump::new();
        let data = vec!["ab", "-f34"];
        let scals: Vec<_> = data.iter().map(|v| v.into()).collect();
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array.to_column(&alloc, &(0..0), None),
            Column::VarChar((&data[0..0], &scals[0..0]))
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
            array.to_ark_scalars(),
            data.iter().map(|v| v.into()).collect::<Vec<ArkScalar>>()
        );
    }

    #[test]
    fn we_can_convert_valid_string_array_refs_into_valid_vec_scalars() {
        let data = vec!["ab", "-f34"];
        let array: ArrayRef = Arc::new(arrow::array::StringArray::from(data.clone()));
        assert_eq!(
            array.to_ark_scalars(),
            data.iter().map(|v| v.into()).collect::<Vec<ArkScalar>>()
        );
    }
}
