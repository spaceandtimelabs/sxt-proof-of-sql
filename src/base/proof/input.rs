use crate::base::{
    proof::{Commit, Commitment, ProofError, ProofResult},
    scalar::IntoScalar,
};
use curve25519_dalek::scalar::Scalar;
use datafusion::{
    arrow::{
        array::{Array, PrimitiveArray},
        datatypes::{
            ArrowPrimitiveType, DataType::*, Int16Type, Int32Type, Int64Type, Int8Type, UInt16Type,
            UInt32Type, UInt64Type, UInt8Type,
        },
    },
    physical_plan::ColumnarValue,
};
use derive_more::{Deref, DerefMut};
use std::convert::TryFrom;

/// New-type representing a database column.
#[derive(Clone, Default, Debug, Eq, PartialEq, Deref, DerefMut)]
pub struct Column<T> {
    pub data: Vec<T>,
}

impl<T> Column<T>
where
    T: IntoScalar + Clone,
{
    pub fn into_scalar_column(self) -> Column<Scalar> {
        Column::from(
            self.iter()
                .map(|d| d.clone().into_scalar())
                .collect::<Vec<Scalar>>(),
        )
    }
}

impl<T> Commit for Column<T>
where
    T: IntoScalar + Clone,
{
    type Commitment = Commitment;

    fn commit(&self) -> Self::Commitment {
        Commitment::from(
            self.iter()
                .map(|d| d.clone().into_scalar())
                .collect::<Vec<Scalar>>()
                .as_slice(),
        )
    }
}

impl<T> From<Vec<T>> for Column<T> {
    fn from(data: Vec<T>) -> Self {
        Column { data }
    }
}

// This requires the array to have no nulls
// TODO: make sure nulls are considered in the next version
// Does not cover string & binary types
impl<T> TryFrom<&PrimitiveArray<T>> for Column<T::Native>
where
    T: ArrowPrimitiveType,
{
    type Error = ProofError;
    fn try_from(data: &PrimitiveArray<T>) -> ProofResult<Self> {
        if data.null_count() > 0 {
            Err(ProofError::NullabilityError)
        } else {
            match data.data_type() {
                Boolean | UInt8 | UInt16 | UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 => {
                    let len = data.len();
                    let vec: Vec<T::Native> = (0..len).map(|index| data.value(index)).collect();
                    Ok(Column { data: vec })
                }
                _ => Err(ProofError::UnimplementedError),
            }
        }
    }
}

macro_rules! column_try_from_columnar_value {
    ($arrow_type:ty, $scalar_variant:ident, $native_type:ty) => {
        impl TryFrom<&ColumnarValue> for Column<$native_type> {
            type Error = ProofError;
            fn try_from(data: &ColumnarValue) -> ProofResult<Self> {
                type PA = PrimitiveArray<$arrow_type>;
                match data {
                    ColumnarValue::Array(arr) => {
                        let any = arr.as_any();
                        if any.is::<PA>() {
                            let pa: &PA = any.downcast_ref::<PA>().ok_or(ProofError::TypeError)?;
                            Column::try_from(pa)
                        } else {
                            Err(ProofError::TypeError)
                        }
                    }
                    _ => Err(ProofError::TypeError),
                }
            }
        }
    };
}

column_try_from_columnar_value!(UInt8Type, UInt8, u8);
column_try_from_columnar_value!(UInt16Type, UInt16, u16);
column_try_from_columnar_value!(UInt32Type, UInt32, u32);
column_try_from_columnar_value!(UInt64Type, UInt64, u64);
column_try_from_columnar_value!(Int8Type, Int8, i8);
column_try_from_columnar_value!(Int16Type, Int16, i16);
column_try_from_columnar_value!(Int32Type, Int32, i32);
column_try_from_columnar_value!(Int64Type, Int64, i64);

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::array::{Int32Array, Int64Array, UInt16Array},
        scalar::ScalarValue,
    };
    use std::sync::Arc;

    #[test]
    fn test_i32array_to_column() {
        let arr: Int32Array = PrimitiveArray::from_iter_values((0..7).map(|x| x + 1));
        let actual: Column<i32> = Column::try_from(&arr).unwrap();
        let expected: Column<i32> = Column {
            data: vec![1, 2, 3, 4, 5, 6, 7],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_i64array_columnarvalue_to_column() {
        let arr: Int64Array = PrimitiveArray::from_iter_values((0..7).map(|x| x + 2));
        let columnar_value = ColumnarValue::Array(Arc::new(arr));
        let actual: Column<i64> = Column::try_from(&columnar_value).unwrap();
        let expected: Column<i64> = Column {
            data: vec![2, 3, 4, 5, 6, 7, 8],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_u16array_columnarvalue_to_column() {
        let arr: UInt16Array = PrimitiveArray::from_iter_values((0..7).map(|x| x + 2));
        let columnar_value = ColumnarValue::Array(Arc::new(arr));
        let actual: Column<u16> = Column::try_from(&columnar_value).unwrap();
        let expected: Column<u16> = Column {
            data: vec![2, 3, 4, 5, 6, 7, 8],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_u32scalar_columnarvalue_to_column_panic() {
        let scalar = ScalarValue::UInt32(Some(20));
        let columnar_value = ColumnarValue::Scalar(scalar);
        let _column: Column<u32> = Column::try_from(&columnar_value).unwrap();
    }
}
