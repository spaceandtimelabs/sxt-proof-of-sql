use super::array::print_long_array;
use super::raw_pointer::RawPtrBox;
use datafusion::arrow::{
    array::{Array, ArrayData, FixedSizeListArray, JsonEqual},
    buffer::{Buffer, MutableBuffer},
    datatypes::DataType,
    error::{ArrowError, Result},
    util::bit_util,
};
use hex::FromHex;
use serde_json::value::Value::{Null as JNull, String as JString};
use serde_json::Value;
use std::{any::Any, fmt};

/// An array where each element consists of \mathbb{Z}_p elements.
///
/// # Examples
///
/// Create an array from an iterable argument of byte slices.
///
/// ```no_run
///    use datafusion::arrow::array::Array;
///    use proofs::base::arrow::ScalarArray;
///    let input_arg = vec![ vec![1; 32], vec![3; 32], vec![5; 32] ];
///    let arr = ScalarArray::try_from_iter(input_arg.into_iter()).unwrap();
///
///    assert_eq!(3, arr.len());
///
/// ```
/// Create an array from an iterable argument of sparse byte slices.
/// Sparsity means that the input argument can contain `None` items.
/// ```no_run
///    use datafusion::arrow::array::Array;
///    use proofs::base::arrow::ScalarArray;
///    let input_arg = vec![ None, Some(vec![7; 32]), Some(vec![9; 32]), None, Some(vec![15; 32]) ];
///    let arr = ScalarArray::try_from_sparse_iter(input_arg.into_iter()).unwrap();
///    assert_eq!(5, arr.len())
///
/// ```
///

pub struct ScalarArray {
    data: ArrayData,
    value_data: RawPtrBox<u8>,
}

impl ScalarArray {
    /// Returns the element at index `i` as a byte slice.
    pub fn value(&self, i: usize) -> &[u8] {
        assert!(i < self.data.len(), "ScalarArray out of bounds access");
        let offset = i + self.data.offset();
        unsafe {
            let pos = self.value_offset_at(offset);
            std::slice::from_raw_parts(
                self.value_data.as_ptr().offset(pos as isize),
                (self.value_offset_at(offset + 1) - pos) as usize,
            )
        }
    }

    /// Returns the element at index `i` as a byte slice.
    /// # Safety
    /// Caller is responsible for ensuring that the index is within the bounds of the array
    pub unsafe fn value_unchecked(&self, i: usize) -> &[u8] {
        let offset = i + self.data.offset();
        let pos = self.value_offset_at(offset);
        std::slice::from_raw_parts(
            self.value_data.as_ptr().offset(pos as isize),
            (self.value_offset_at(offset + 1) - pos) as usize,
        )
    }

    /// Returns the offset for the element at index `i`.
    ///
    /// Note this doesn't do any bound checking, for performance reason.
    #[inline]
    pub fn value_offset(&self, i: usize) -> i32 {
        self.value_offset_at(self.data.offset() + i)
    }

    /// Returns the length for an element in as u8 arrays.
    ///
    /// All elements have the same length as the array is a fixed size.
    #[inline]
    pub fn value_length(&self) -> i32 {
        32
    }

    /// Returns a clone of the value data buffer
    pub fn value_data(&self) -> Buffer {
        self.data.buffers()[0].clone()
    }

    /// Create an array from an iterable argument of sparse byte slices.
    /// Sparsity means that items returned by the iterator are optional, i.e input argument can
    /// contain `None` items.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use proofs::base::arrow::ScalarArray;
    /// let input_arg = vec![
    ///     None,
    ///     Some(vec![7; 32]),
    ///     Some(vec![9; 32]),
    ///     None,
    ///     Some(vec![13; 32]),
    ///     None,
    /// ];
    /// let array = ScalarArray::try_from_sparse_iter(input_arg.into_iter()).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if argument has length zero, or sizes of nested slices don't match.
    pub fn try_from_sparse_iter<T, U>(mut iter: T) -> Result<Self>
    where
        T: Iterator<Item = Option<U>>,
        U: AsRef<[u8]>,
    {
        let mut len = 0;
        let mut byte = 0;
        let mut null_buf = MutableBuffer::from_len_zeroed(0);
        let mut buffer = MutableBuffer::from_len_zeroed(0);
        let mut prepend = 0;
        let mut initialized = false;
        iter.try_for_each(|item| -> Result<()> {
            // extend null bitmask by one byte per each 8 items
            if byte == 0 {
                null_buf.push(0u8);
                byte = 8;
            }
            byte -= 1;

            if let Some(slice) = item {
                let slice = slice.as_ref();
                if initialized {
                    if slice.len() != 32 {
                        return Err(ArrowError::InvalidArgumentError(format!(
                            "Nested array size mismatch: it should be 32 but is {}",
                            slice.len()
                        )));
                    }
                } else {
                    initialized = true;
                    buffer.extend_zeros(32 * prepend);
                }
                bit_util::set_bit(null_buf.as_slice_mut(), len);
                buffer.extend_from_slice(slice);
            } else if initialized {
                buffer.extend_zeros(32);
            } else {
                prepend += 1;
            }

            len += 1;

            Ok(())
        })?;

        if len == 0 {
            return Err(ArrowError::InvalidArgumentError(
                "Input iterable argument has no data".to_owned(),
            ));
        }

        let array_data = unsafe {
            ArrayData::new_unchecked(
                DataType::FixedSizeBinary(32),
                len,
                None,
                Some(null_buf.into()),
                0,
                vec![buffer.into()],
                vec![],
            )
        };
        Ok(ScalarArray::from(array_data))
    }

    /// Create an array from an iterable argument of byte slices.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use proofs::base::arrow::ScalarArray;
    /// let input_arg = vec![
    ///     vec![1; 32],
    ///     vec![3; 32],
    ///     vec![5; 32],
    /// ];
    /// let array = ScalarArray::try_from_iter(input_arg.into_iter()).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if argument has length zero, or sizes of nested slices don't match.
    pub fn try_from_iter<T, U>(mut iter: T) -> Result<Self>
    where
        T: Iterator<Item = U>,
        U: AsRef<[u8]>,
    {
        let mut len = 0;
        let mut buffer = MutableBuffer::from_len_zeroed(0);
        iter.try_for_each(|item| -> Result<()> {
            let slice = item.as_ref();
            if slice.len() != 32 {
                return Err(ArrowError::InvalidArgumentError(format!(
                    "Nested array size mismatch: it should be 32 but is {}",
                    slice.len()
                )));
            }
            buffer.extend_from_slice(slice);

            len += 1;

            Ok(())
        })?;

        if len == 0 {
            return Err(ArrowError::InvalidArgumentError(
                "Input iterable argument has no data".to_owned(),
            ));
        }

        let array_data = ArrayData::builder(DataType::FixedSizeBinary(32))
            .len(len)
            .add_buffer(buffer.into());
        let array_data = unsafe { array_data.build_unchecked() };
        Ok(ScalarArray::from(array_data))
    }

    #[inline]
    fn value_offset_at(&self, i: usize) -> i32 {
        32 * i as i32
    }
}

impl From<ArrayData> for ScalarArray {
    fn from(data: ArrayData) -> Self {
        assert_eq!(
            data.buffers().len(),
            1,
            "ScalarArray data should contain 1 buffer only (values)"
        );
        let value_data = data.buffers()[0].as_ptr();
        match data.data_type() {
            DataType::FixedSizeBinary(32) => (),
            _ => panic!(
                "Expected data type to be FixedSizeBinary with length of each element \
            32"
            ),
        };
        Self {
            data,
            value_data: unsafe { RawPtrBox::new(value_data) },
        }
    }
}

/// Creates a `ScalarArray` from `FixedSizeList<u8>` array
impl From<FixedSizeListArray> for ScalarArray {
    fn from(v: FixedSizeListArray) -> Self {
        assert_eq!(
            v.data_ref().child_data()[0].child_data().len(),
            0,
            "ScalarArray can only be created from list array of u8 values \
             (i.e. FixedSizeList<PrimitiveArray<u8>>)."
        );
        assert_eq!(
            v.data_ref().child_data()[0].data_type(),
            &DataType::UInt8,
            "ScalarArray can only be created from FixedSizeList<u8> arrays, mismatched data types."
        );
        assert_eq!(
            v.value_length(),
            32,
            "ScalarArray can only be created from FixedSizeList<u8> arrays \
             with the length of each element equal to 32."
        );

        let builder = ArrayData::builder(DataType::FixedSizeBinary(v.value_length()))
            .len(v.len())
            .add_buffer(v.data_ref().child_data()[0].buffers()[0].clone())
            .null_bit_buffer(v.data_ref().null_buffer().cloned());

        let data = unsafe { builder.build_unchecked() };
        Self::from(data)
    }
}

impl From<Vec<Option<&[u8; 32]>>> for ScalarArray {
    fn from(v: Vec<Option<&[u8; 32]>>) -> Self {
        Self::try_from_sparse_iter(v.into_iter()).unwrap()
    }
}

impl From<Vec<&[u8; 32]>> for ScalarArray {
    fn from(v: Vec<&[u8; 32]>) -> Self {
        Self::try_from_iter(v.into_iter()).unwrap()
    }
}

impl fmt::Debug for ScalarArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ScalarArray<{}>\n[\n", self.value_length())?;
        print_long_array(self, f, |array, index, f| {
            fmt::Debug::fmt(&array.value(index), f)
        })?;
        write!(f, "]")
    }
}

impl Array for ScalarArray {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn data(&self) -> &ArrayData {
        &self.data
    }
}

impl JsonEqual for ScalarArray {
    fn equals_json(&self, json: &[&Value]) -> bool {
        if self.len() != json.len() {
            return false;
        }

        (0..self.len()).all(|i| match json[i] {
            JString(s) => {
                // binary data is sometimes hex encoded, this checks if bytes are equal,
                // and if not converting to hex is attempted
                self.is_valid(i)
                    && (s.as_str().as_bytes() == self.value(i)
                        || Vec::from_hex(s.as_str()) == Ok(self.value(i).to_vec()))
            }
            JNull => self.is_null(i),
            _ => false,
        })
    }
}

impl PartialEq<Value> for ScalarArray {
    fn eq(&self, json: &Value) -> bool {
        match json {
            Value::Array(json_array) => self.equals_json_values(json_array),
            _ => false,
        }
    }
}

impl PartialEq<ScalarArray> for Value {
    fn eq(&self, arrow: &ScalarArray) -> bool {
        match self {
            Value::Array(json_array) => arrow.equals_json_values(json_array),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::datatypes::Field;

    #[test]
    fn test_scalar_array() {
        let mut values = Vec::with_capacity(96);
        values.extend_from_slice(&[1; 32]);
        values.extend_from_slice(&[2; 32]);
        values.extend_from_slice(&[3; 32]);

        let array_data = ArrayData::builder(DataType::FixedSizeBinary(32))
            .len(3)
            .add_buffer(Buffer::from(&values[..]))
            .build()
            .unwrap();
        let scalar_array = ScalarArray::from(array_data);
        assert_eq!(3, scalar_array.len());
        assert_eq!(0, scalar_array.null_count());
        assert_eq!([1; 32], scalar_array.value(0));
        assert_eq!([2; 32], scalar_array.value(1));
        assert_eq!([3; 32], scalar_array.value(2));
        assert_eq!(32, scalar_array.value_length());
        assert_eq!(64, scalar_array.value_offset(2));
        for i in 0..3 {
            assert!(scalar_array.is_valid(i));
            assert!(!scalar_array.is_null(i));
        }

        // Test binary array with offset
        let array_data = ArrayData::builder(DataType::FixedSizeBinary(32))
            .len(2)
            .offset(1)
            .add_buffer(Buffer::from(&values[..]))
            .build()
            .unwrap();
        let scalar_array = ScalarArray::from(array_data);
        assert_eq!([2; 32], scalar_array.value(0));
        assert_eq!([3; 32], scalar_array.value(1));
        assert_eq!(2, scalar_array.len());
        assert_eq!(32, scalar_array.value_offset(0));
        assert_eq!(32, scalar_array.value_length());
        assert_eq!(64, scalar_array.value_offset(1));
    }

    #[test]
    #[should_panic(expected = "ScalarArray can only be created from FixedSizeList<u8> arrays")]
    fn test_scalar_array_from_list_array_with_incorrect_datatype() {
        let values: [u32; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let values_data = ArrayData::builder(DataType::UInt32)
            .len(12)
            .add_buffer(Buffer::from_slice_ref(&values))
            .build()
            .unwrap();

        let array_data = unsafe {
            ArrayData::builder(DataType::FixedSizeList(
                Box::new(Field::new("item", DataType::Binary, false)),
                4,
            ))
            .len(3)
            .add_child_data(values_data)
            .build_unchecked()
        };
        let list_array = FixedSizeListArray::from(array_data);
        drop(ScalarArray::from(list_array));
    }

    #[test]
    #[should_panic(
        expected = "ScalarArray can only be created from FixedSizeList<u8> arrays with the length of each element equal to 32."
    )]
    fn test_scalar_array_from_list_array_with_incorrect_length() {
        let values: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let values_data = ArrayData::builder(DataType::UInt8)
            .len(12)
            .add_buffer(Buffer::from_slice_ref(&values))
            .build()
            .unwrap();

        let array_data = unsafe {
            ArrayData::builder(DataType::FixedSizeList(
                Box::new(Field::new("item", DataType::Binary, false)),
                4,
            ))
            .len(3)
            .add_child_data(values_data)
            .build_unchecked()
        };
        let list_array = FixedSizeListArray::from(array_data);
        drop(ScalarArray::from(list_array));
    }

    #[test]
    fn test_scalar_array_from_iter() {
        let input_arg = vec![vec![1; 32], vec![3; 32], vec![5; 32]];
        let arr = ScalarArray::try_from_iter(input_arg.into_iter()).unwrap();

        assert_eq!(32, arr.value_length());
        assert_eq!(3, arr.len())
    }

    #[test]
    fn test_all_none_scalar_array_from_sparse_iter() {
        let none_option: Option<[u8; 32]> = None;
        let input_arg = vec![none_option, none_option, none_option];
        let arr = ScalarArray::try_from_sparse_iter(input_arg.into_iter()).unwrap();
        assert_eq!(32, arr.value_length());
        assert_eq!(3, arr.len())
    }

    #[test]
    fn test_scalar_array_from_sparse_iter() {
        let input_arg = vec![
            None,
            Some(vec![7; 32]),
            Some(vec![9; 32]),
            None,
            Some(vec![13; 32]),
        ];
        let arr = ScalarArray::try_from_sparse_iter(input_arg.into_iter()).unwrap();
        assert_eq!(32, arr.value_length());
        assert_eq!(5, arr.len())
    }

    #[test]
    fn test_scalar_array_from_vec() {
        let values = vec![&[12_u8; 32]; 4];
        let array = ScalarArray::from(values);
        assert_eq!(array.len(), 4);
        assert_eq!(array.null_count(), 0);
        for i in 0..4 {
            assert_eq!(array.value(i), [12_u8; 32]);
            assert!(!array.is_null(i));
        }
    }

    #[test]
    fn test_scalar_array_from_opt_vec() {
        let values = vec![
            Some(&[1_u8; 32]),
            Some(&[4_u8; 32]),
            None,
            Some(&[12_u8; 32]),
            Some(&[7_u8; 32]),
        ];
        let array = ScalarArray::from(values);
        assert_eq!(array.len(), 5);
        assert_eq!(array.value(0), [1_u8; 32]);
        assert_eq!(array.value(1), [4_u8; 32]);
        assert_eq!(array.value(3), [12_u8; 32]);
        assert_eq!(array.value(4), [7_u8; 32]);
        assert!(!array.is_null(0));
        assert!(!array.is_null(1));
        assert!(array.is_null(2));
        assert!(!array.is_null(3));
        assert!(!array.is_null(4));
    }
}
