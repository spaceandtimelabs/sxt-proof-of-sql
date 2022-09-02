/// Definitions of Column, GeneralColumn, Table and conversions
/// from Arrow Arrays, RecordBatches as well as Datafusion ColumnarValues into them.
use crate::base::{
    proof::{Commit, Commitment, ProofError, ProofResult},
    scalar::IntoScalar,
};
use curve25519_dalek::scalar::Scalar;
use datafusion::{
    arrow::{
        array::{
            Array, ArrayRef, BooleanArray, Int16Array, Int32Array, Int64Array, Int8Array,
            PrimitiveArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
        },
        datatypes::{ArrowPrimitiveType, DataType::*},
        record_batch::RecordBatch,
    },
    physical_plan::ColumnarValue,
};
use derive_more::{Deref, DerefMut, TryInto};
use std::convert::TryFrom;

/// Definition of Column, GeneralColumn and Table

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

impl<T> IntoIterator for Column<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// Enum of columns of all the supported types
#[derive(Clone, Debug, Eq, PartialEq, TryInto)]
#[try_into(owned, ref, ref_mut)]
pub enum GeneralColumn {
    BooleanColumn(Column<bool>),
    Int8Column(Column<i8>),
    Int16Column(Column<i16>),
    Int32Column(Column<i32>),
    Int64Column(Column<i64>),
    UInt8Column(Column<u8>),
    UInt16Column(Column<u16>),
    UInt32Column(Column<u32>),
    UInt64Column(Column<u64>),
}

impl GeneralColumn {
    pub fn len(&self) -> usize {
        match self {
            GeneralColumn::BooleanColumn(c) => c.data.len(),
            GeneralColumn::Int8Column(c) => c.data.len(),
            GeneralColumn::Int16Column(c) => c.data.len(),
            GeneralColumn::Int32Column(c) => c.data.len(),
            GeneralColumn::Int64Column(c) => c.data.len(),
            GeneralColumn::UInt8Column(c) => c.data.len(),
            GeneralColumn::UInt16Column(c) => c.data.len(),
            GeneralColumn::UInt32Column(c) => c.data.len(),
            GeneralColumn::UInt64Column(c) => c.data.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Commit for GeneralColumn {
    type Commitment = Commitment;

    fn commit(&self) -> Self::Commitment {
        match self {
            GeneralColumn::BooleanColumn(c) => c.commit(),
            GeneralColumn::Int8Column(c) => c.commit(),
            GeneralColumn::Int16Column(c) => c.commit(),
            GeneralColumn::Int32Column(c) => c.commit(),
            GeneralColumn::Int64Column(c) => c.commit(),
            GeneralColumn::UInt8Column(c) => c.commit(),
            GeneralColumn::UInt16Column(c) => c.commit(),
            GeneralColumn::UInt32Column(c) => c.commit(),
            GeneralColumn::UInt64Column(c) => c.commit(),
        }
    }
}

impl From<GeneralColumn> for Column<Scalar> {
    fn from(general_column: GeneralColumn) -> Self {
        match general_column {
            GeneralColumn::BooleanColumn(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::Int8Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::Int16Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::Int32Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::Int64Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::UInt8Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::UInt16Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::UInt32Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::UInt64Column(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

/// The proof version of RecordBatch
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table {
    pub data: Vec<GeneralColumn>,
    /// Num of rows in any GeneralColumn in data
    pub num_rows: usize,
}

impl Table {
    pub fn try_new(data: Vec<GeneralColumn>, num_rows: usize) -> ProofResult<Self> {
        for col in data.iter() {
            if col.len() != num_rows {
                return Err(ProofError::TableColumnLengthError);
            }
        }
        Ok(Table { data, num_rows })
    }
}

impl Commit for Table {
    type Commitment = Vec<Commitment>;

    fn commit(&self) -> Self::Commitment {
        self.data.iter().map(|c| c.commit()).collect()
    }
}

/// Array and ColumnarValue to Column

impl TryFrom<&BooleanArray> for Column<bool> {
    type Error = ProofError;
    fn try_from(data: &BooleanArray) -> ProofResult<Self> {
        if data.null_count() > 0 {
            Err(ProofError::NullabilityError)
        } else {
            let len = data.len();
            let vec: Vec<bool> = (0..len).map(|index| data.value(index)).collect();
            Ok(Column { data: vec })
        }
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
                UInt8 | UInt16 | UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 => {
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
    ($arrow_type:ty, $native_type:ty, $arrow_array_type:ty) => {
        impl TryFrom<&ColumnarValue> for Column<$native_type> {
            type Error = ProofError;
            fn try_from(data: &ColumnarValue) -> ProofResult<Self> {
                match data {
                    ColumnarValue::Array(arr) => {
                        let any = arr.as_any();
                        if any.is::<$arrow_array_type>() {
                            let pa: &$arrow_array_type = any
                                .downcast_ref::<$arrow_array_type>()
                                .ok_or(ProofError::TypeError)?;
                            Column::try_from(pa)
                        } else {
                            Err(ProofError::TypeError)
                        }
                    }
                    // num_rows needed for Scalars. See the try_from function below.
                    _ => Err(ProofError::TypeError),
                }
            }
        }

        impl TryFrom<(&ColumnarValue, usize)> for Column<$native_type> {
            type Error = ProofError;
            fn try_from(data: (&ColumnarValue, usize)) -> ProofResult<Self> {
                let arr = data.0.clone().into_array(data.1);
                let any = arr.as_any();
                if any.is::<$arrow_array_type>() {
                    let pa: &$arrow_array_type = any
                        .downcast_ref::<$arrow_array_type>()
                        .ok_or(ProofError::TypeError)?;
                    Column::try_from(pa)
                } else {
                    Err(ProofError::TypeError)
                }
            }
        }
    };
}

column_try_from_columnar_value!(BooleanType, bool, BooleanArray);
column_try_from_columnar_value!(UInt8Type, u8, UInt8Array);
column_try_from_columnar_value!(UInt16Type, u16, UInt16Array);
column_try_from_columnar_value!(UInt32Type, u32, UInt32Array);
column_try_from_columnar_value!(UInt64Type, u64, UInt64Array);
column_try_from_columnar_value!(Int8Type, i8, Int8Array);
column_try_from_columnar_value!(Int16Type, i16, Int16Array);
column_try_from_columnar_value!(Int32Type, i32, Int32Array);
column_try_from_columnar_value!(Int64Type, i64, Int64Array);

/// ArrayRef and ColumnarValue to GeneralColumn

impl TryFrom<&ArrayRef> for GeneralColumn {
    type Error = ProofError;
    fn try_from(data: &ArrayRef) -> ProofResult<Self> {
        match (&**data).data_type() {
            Boolean => Ok(GeneralColumn::BooleanColumn(Column::try_from(
                data.as_any()
                    .downcast_ref::<BooleanArray>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            Int8 => Ok(GeneralColumn::Int8Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<Int8Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            Int16 => Ok(GeneralColumn::Int16Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<Int16Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            Int32 => Ok(GeneralColumn::Int32Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<Int32Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            Int64 => Ok(GeneralColumn::Int64Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<Int64Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            UInt8 => Ok(GeneralColumn::UInt8Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<UInt8Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            UInt16 => Ok(GeneralColumn::UInt16Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<UInt16Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            UInt32 => Ok(GeneralColumn::UInt32Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<UInt32Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            UInt64 => Ok(GeneralColumn::UInt64Column(Column::try_from(
                data.as_any()
                    .downcast_ref::<UInt64Array>()
                    .ok_or(ProofError::TypeError)?,
            )?)),
            _ => Err(ProofError::TypeError),
        }
    }
}

// Has to be Array because we don't have num_rows
impl TryFrom<&ColumnarValue> for GeneralColumn {
    type Error = ProofError;
    fn try_from(data: &ColumnarValue) -> ProofResult<Self> {
        match data {
            ColumnarValue::Array(a) => GeneralColumn::try_from(a),
            _ => Err(ProofError::TypeError),
        }
    }
}

impl TryFrom<(&ColumnarValue, usize)> for GeneralColumn {
    type Error = ProofError;
    fn try_from(data: (&ColumnarValue, usize)) -> ProofResult<Self> {
        let arr = data.0.clone().into_array(data.1);
        GeneralColumn::try_from(&arr)
    }
}

/// RecordBatch, Vec<ArrayRef> and Vec<ColumnarValue> to Table

impl TryFrom<&RecordBatch> for Table {
    type Error = ProofError;
    // No need to check that all cols have the same length
    fn try_from(data: &RecordBatch) -> ProofResult<Self> {
        Ok(Table {
            data: data
                .columns()
                .iter()
                .map(GeneralColumn::try_from)
                .into_iter()
                .collect::<ProofResult<Vec<GeneralColumn>>>()?,
            num_rows: data.num_rows(),
        })
    }
}

impl TryFrom<&Vec<ArrayRef>> for Table {
    type Error = ProofError;
    fn try_from(data: &Vec<ArrayRef>) -> ProofResult<Self> {
        // From an empt vec it is not clear what the default num_rows is
        if data.is_empty() {
            Err(ProofError::TableColumnLengthError)
        } else {
            let num_rows = data[0].len();
            let table_data = data
                .clone()
                .iter()
                .map(GeneralColumn::try_from)
                .into_iter()
                .collect::<ProofResult<Vec<GeneralColumn>>>()?;
            Table::try_new(table_data, num_rows)
        }
    }
}

impl TryFrom<(&Vec<ArrayRef>, usize)> for Table {
    type Error = ProofError;
    fn try_from(data: (&Vec<ArrayRef>, usize)) -> ProofResult<Self> {
        let num_rows = data.1;
        let table_data = data
            .0
            .clone()
            .iter()
            .map(GeneralColumn::try_from)
            .into_iter()
            .collect::<ProofResult<Vec<GeneralColumn>>>()?;
        Table::try_new(table_data, num_rows)
    }
}

impl TryFrom<&Vec<ColumnarValue>> for Table {
    type Error = ProofError;
    fn try_from(data: &Vec<ColumnarValue>) -> ProofResult<Self> {
        // From an empt vec it is not clear what the default num_rows is
        if data.is_empty() {
            Err(ProofError::TableColumnLengthError)
        } else {
            let table_data = data
                .clone()
                .iter()
                .map(GeneralColumn::try_from)
                .into_iter()
                .collect::<ProofResult<Vec<GeneralColumn>>>()?;
            let num_rows = table_data[0].len();
            Table::try_new(table_data, num_rows)
        }
    }
}

impl TryFrom<(&Vec<ColumnarValue>, usize)> for Table {
    type Error = ProofError;
    fn try_from(data: (&Vec<ColumnarValue>, usize)) -> ProofResult<Self> {
        let num_rows = data.1;
        let table_data = data
            .0
            .clone()
            .iter()
            .map(|c| GeneralColumn::try_from((c, num_rows)))
            .into_iter()
            .collect::<ProofResult<Vec<GeneralColumn>>>()?;
        Table::try_new(table_data, num_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::{
            array::{Float32Array, Int32Array, Int64Array, UInt16Array},
            datatypes::{DataType, Field, Schema},
        },
        scalar::ScalarValue,
    };
    use std::sync::Arc;

    #[test]
    fn test_generalcolumn_length() {
        let general_column = GeneralColumn::Int16Column(Column {
            data: vec![1, 2, 3],
        });
        assert_eq!(general_column.len(), 3);
    }

    #[test]
    fn test_generalcolumn_length_empty() {
        let general_column = GeneralColumn::Int8Column(Column { data: vec![] });
        assert_eq!(general_column.len(), 0);
    }

    #[test]
    fn test_generalcolumn_is_empty_true() {
        let general_column = GeneralColumn::Int64Column(Column { data: vec![] });
        assert_eq!(general_column.is_empty(), true);
    }

    #[test]
    fn test_generalcolumn_is_empty_false() {
        let general_column = GeneralColumn::Int16Column(Column {
            data: vec![-1, -2, -3],
        });
        assert_eq!(general_column.is_empty(), false);
    }

    #[test]
    fn test_table_try_new() {
        let general_column0 = GeneralColumn::Int16Column(Column {
            data: vec![-1, -2, -3],
        });
        let general_column1 = GeneralColumn::Int32Column(Column {
            data: vec![1, 2, 3],
        });
        let general_columns = vec![general_column0, general_column1];
        let actual = Table::try_new(general_columns, 3).unwrap();
        let expected = Table {
            data: vec![
                GeneralColumn::Int16Column(Column {
                    data: vec![-1, -2, -3],
                }),
                GeneralColumn::Int32Column(Column {
                    data: vec![1, 2, 3],
                }),
            ],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_table_try_new() {
        let general_columns: Vec<GeneralColumn> = vec![];
        let actual = Table::try_new(general_columns, 3).unwrap();
        let expected = Table {
            data: vec![],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_table_try_new_failed_incompatible_lengths() {
        let general_column0 = GeneralColumn::Int16Column(Column { data: vec![-1, -2] });
        let general_column1 = GeneralColumn::Int32Column(Column {
            data: vec![1, 2, 3],
        });
        let general_columns = vec![general_column0, general_column1];
        Table::try_new(general_columns, 3).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_table_try_new_failed_wrong_num_rows() {
        let general_column0 = GeneralColumn::Int16Column(Column {
            data: vec![-1, -2, 3],
        });
        let general_column1 = GeneralColumn::Int32Column(Column {
            data: vec![1, 2, 3],
        });
        let general_columns = vec![general_column0, general_column1];
        Table::try_new(general_columns, 2).unwrap();
    }

    #[test]
    fn test_booleanarray_to_column() {
        let arr: BooleanArray = BooleanArray::from(vec![true, false, true, false]);
        let actual: Column<bool> = Column::try_from(&arr).unwrap();
        let expected: Column<bool> = Column {
            data: vec![true, false, true, false],
        };
        assert_eq!(actual, expected);
    }

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
    fn test_booleanarray_columnarvalue_to_column() {
        let arr: BooleanArray = BooleanArray::from(vec![false, false, true, false]);
        let columnar_value = ColumnarValue::Array(Arc::new(arr));
        let actual: Column<bool> = Column::try_from(&columnar_value).unwrap();
        let expected: Column<bool> = Column {
            data: vec![false, false, true, false],
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
    fn test_u32scalar_columnarvalue_to_column_failed() {
        let scalar = ScalarValue::UInt32(Some(20));
        let columnar_value = ColumnarValue::Scalar(scalar);
        let _column: Column<u32> = Column::try_from(&columnar_value).unwrap();
    }

    #[test]
    fn test_i64array_columnarvalue_to_column_with_num_rows() {
        let arr: Int64Array = PrimitiveArray::from_iter_values((0..7).map(|x| x + 3));
        let columnar_value = ColumnarValue::Array(Arc::new(arr));
        let actual: Column<i64> = Column::try_from((&columnar_value, 7)).unwrap();
        let expected: Column<i64> = Column {
            data: vec![3, 4, 5, 6, 7, 8, 9],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_u32scalar_columnarvalue_to_column_with_num_rows() {
        let scalar = ScalarValue::UInt32(Some(20));
        let columnar_value = ColumnarValue::Scalar(scalar);
        let actual: Column<u32> = Column::try_from((&columnar_value, 5)).unwrap();
        let expected: Column<u32> = Column { data: vec![20; 5] };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_array_to_generalcolumn() {
        let arr: Int16Array = PrimitiveArray::from_iter_values((0..7).map(|x| x + 2));
        let arc_arr: ArrayRef = Arc::new(arr);
        let actual: GeneralColumn = GeneralColumn::try_from(&arc_arr).unwrap();
        let expected: GeneralColumn = GeneralColumn::Int16Column(Column {
            data: vec![2, 3, 4, 5, 6, 7, 8],
        });
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_array_to_generalcolumn_failed() {
        let arr = Float32Array::from(vec![1.0, 2.0, 2.5]);
        let arc_arr: ArrayRef = Arc::new(arr);
        GeneralColumn::try_from(&arc_arr).unwrap();
    }

    #[test]
    fn test_columnar_value_to_generalcolumn() {
        let arr: Int16Array = PrimitiveArray::from_iter_values((0..7).map(|x| x + 2));
        let columnar_value = ColumnarValue::Array(Arc::new(arr));
        let actual: GeneralColumn = GeneralColumn::try_from(&columnar_value).unwrap();
        let expected: GeneralColumn = GeneralColumn::Int16Column(Column {
            data: vec![2, 3, 4, 5, 6, 7, 8],
        });
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_columnar_value_to_generalcolumn_failed() {
        let scalar = ScalarValue::Int8(Some(50));
        let columnar_value = ColumnarValue::Scalar(scalar);
        GeneralColumn::try_from(&columnar_value).unwrap();
    }

    #[test]
    fn test_columnar_value_array_to_generalcolumn_with_num_rows() {
        let arr: Int64Array = PrimitiveArray::from_iter_values((0..7).map(|x| x + 2));
        let columnar_value = ColumnarValue::Array(Arc::new(arr));
        let actual: GeneralColumn = GeneralColumn::try_from((&columnar_value, 7)).unwrap();
        let expected: GeneralColumn = GeneralColumn::Int64Column(Column {
            data: vec![2, 3, 4, 5, 6, 7, 8],
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_columnar_value_scalar_to_generalcolumn_with_num_rows() {
        let scalar = ScalarValue::Int8(Some(50));
        let columnar_value = ColumnarValue::Scalar(scalar);
        let actual: GeneralColumn = GeneralColumn::try_from((&columnar_value, 7)).unwrap();
        let expected: GeneralColumn = GeneralColumn::Int8Column(Column { data: vec![50; 7] });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_batch_to_table() {
        // Setup
        let array0 = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1 = Arc::new(Int64Array::from(vec![1, -2, -3]));
        let array2 = Arc::new(BooleanArray::from(vec![true, false, true]));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int64, false),
            Field::new("c", DataType::Boolean, false),
        ]);
        let batch =
            RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1, array2]).unwrap();

        let actual: Table = Table::try_from(&batch).unwrap();
        let expected: Table = Table {
            data: vec![
                GeneralColumn::Int32Column(Column {
                    data: vec![1, 2, 3],
                }),
                GeneralColumn::Int64Column(Column {
                    data: vec![1, -2, -3],
                }),
                GeneralColumn::BooleanColumn(Column {
                    data: vec![true, false, true],
                }),
            ],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_batch_to_table_failed() {
        // Setup
        let array0 = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1 = Arc::new(Float32Array::from(vec![1.5, -2.5, -3.4]));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Float32, false),
        ]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1]).unwrap();

        Table::try_from(&batch).unwrap();
    }

    #[test]
    fn test_vec_arrayref_to_table() {
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Int64Array::from(vec![5, 7, 9]));
        let array2: ArrayRef = Arc::new(BooleanArray::from(vec![false, false, true]));
        let arrays = vec![array0, array1, array2];

        let actual: Table = Table::try_from(&arrays).unwrap();
        let expected: Table = Table {
            data: vec![
                GeneralColumn::Int32Column(Column {
                    data: vec![1, 2, 3],
                }),
                GeneralColumn::Int64Column(Column {
                    data: vec![5, 7, 9],
                }),
                GeneralColumn::BooleanColumn(Column {
                    data: vec![false, false, true],
                }),
            ],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_vec_arrayref_to_table_failed_empty() {
        let arrays: Vec<ArrayRef> = vec![];
        Table::try_from(&arrays).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_vec_arrayref_to_table_failed_incompatible_lengths() {
        // Setup
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Int32Array::from(vec![1, -2]));
        let arrays = vec![array0, array1];

        Table::try_from(&arrays).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_vec_arrayref_to_table_failed_unsupported_type() {
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Float32Array::from(vec![5.4, 7.2, 9.1]));
        let arrays = vec![array0, array1];

        Table::try_from(&arrays).unwrap();
    }

    #[test]
    fn test_vec_arrayref_to_table_with_num_rows() {
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Int64Array::from(vec![5, 7, 9]));
        let array2: ArrayRef = Arc::new(BooleanArray::from(vec![false, false, true]));
        let arrays = vec![array0, array1, array2];

        let actual: Table = Table::try_from((&arrays, 3)).unwrap();
        let expected: Table = Table {
            data: vec![
                GeneralColumn::Int32Column(Column {
                    data: vec![1, 2, 3],
                }),
                GeneralColumn::Int64Column(Column {
                    data: vec![5, 7, 9],
                }),
                GeneralColumn::BooleanColumn(Column {
                    data: vec![false, false, true],
                }),
            ],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_vec_arrayref_to_table_with_num_rows() {
        let arrays: Vec<ArrayRef> = vec![];

        let actual: Table = Table::try_from((&arrays, 3)).unwrap();
        let expected: Table = Table {
            data: vec![],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_vec_arrayref_to_table_with_num_rows_failed_incompatible_lengths() {
        // Setup
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Int32Array::from(vec![1, -2]));
        let arrays = vec![array0, array1];

        Table::try_from((&arrays, 3)).unwrap();
    }

    // Here the &Vec<ArrayRef> itself is fine but the num of rows passed in is wrong
    #[test]
    #[should_panic]
    fn test_vec_arrayref_to_table_with_num_rows_failed_wrong_num_rows() {
        // Setup
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Int32Array::from(vec![1, -2, 3]));
        let arrays = vec![array0, array1];

        Table::try_from((&arrays, 2)).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_vec_arrayref_to_table_with_num_rows_failed_unsupported_type() {
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Float32Array::from(vec![5.4, 7.2, 9.1]));
        let arrays = vec![array0, array1];

        Table::try_from((&arrays, 3)).unwrap();
    }

    #[test]
    fn test_vec_columnar_value_to_table() {
        let array0 = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1 = Arc::new(Int64Array::from(vec![5, 7, -9]));
        let array2 = Arc::new(BooleanArray::from(vec![true, false, false]));
        let columnar_values = vec![
            ColumnarValue::Array(array0),
            ColumnarValue::Array(array1),
            ColumnarValue::Array(array2),
        ];

        let actual: Table = Table::try_from(&columnar_values).unwrap();
        let expected: Table = Table {
            data: vec![
                GeneralColumn::Int32Column(Column {
                    data: vec![1, 2, 3],
                }),
                GeneralColumn::Int64Column(Column {
                    data: vec![5, 7, -9],
                }),
                GeneralColumn::BooleanColumn(Column {
                    data: vec![true, false, false],
                }),
            ],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_vec_columnar_value_to_table_failed_empty() {
        let columnar_values: Vec<ColumnarValue> = vec![];

        Table::try_from(&columnar_values).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_vec_columnar_value_to_table_failed_scalar() {
        let array = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let scalar = ScalarValue::Int64(Some(20));
        let columnar_values = vec![ColumnarValue::Array(array), ColumnarValue::Scalar(scalar)];

        Table::try_from(&columnar_values).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_vec_columnar_value_to_table_failed_incompatible_lengths() {
        // Setup
        let array0 = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1 = Arc::new(Int32Array::from(vec![1, -2]));
        let columnar_values = vec![ColumnarValue::Array(array0), ColumnarValue::Array(array1)];

        Table::try_from(&columnar_values).unwrap();
    }

    #[test]
    fn test_vec_columnar_value_to_table_with_num_rows() {
        let array = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let scalar = ScalarValue::Int64(Some(20));
        let columnar_values = vec![ColumnarValue::Array(array), ColumnarValue::Scalar(scalar)];

        let actual: Table = Table::try_from((&columnar_values, 3)).unwrap();
        let expected: Table = Table {
            data: vec![
                GeneralColumn::Int32Column(Column {
                    data: vec![1, 2, 3],
                }),
                GeneralColumn::Int64Column(Column {
                    data: vec![20, 20, 20],
                }),
            ],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_vec_columnar_value_to_table_with_num_rows() {
        let columnar_values: Vec<ColumnarValue> = vec![];

        let actual: Table = Table::try_from((&columnar_values, 3)).unwrap();
        let expected: Table = Table {
            data: vec![],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_vec_columnar_value_to_table_with_num_rows_failed_incompatible_lengths() {
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Int32Array::from(vec![1, 2]));
        let columnar_values = vec![ColumnarValue::Array(array0), ColumnarValue::Array(array1)];

        Table::try_from((&columnar_values, 3)).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_vec_columnar_value_to_table_with_num_rows_failed_wrong_num_rows() {
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 4]));
        let columnar_values = vec![ColumnarValue::Array(array0), ColumnarValue::Array(array1)];

        Table::try_from((&columnar_values, 2)).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_vec_columnar_value_to_table_with_num_rows_failed_unsupported_type() {
        let array0: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let array1: ArrayRef = Arc::new(Float32Array::from(vec![1.4, 2.2, 4.9]));
        let columnar_values = vec![ColumnarValue::Array(array0), ColumnarValue::Array(array1)];

        Table::try_from((&columnar_values, 2)).unwrap();
    }
}
