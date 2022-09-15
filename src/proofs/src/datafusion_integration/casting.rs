use datafusion::{arrow::record_batch::RecordBatch, physical_plan::ColumnarValue};

/// Panics
///
/// Panics if `index` is outside of `0..num_columns`.
pub fn batch_column_to_columnar_value(batch: &RecordBatch, index: usize) -> ColumnarValue {
    ColumnarValue::Array(batch.column(index).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::{
        array::{ArrayRef, Int64Array, StringArray},
        compute::kernels::{aggregate::min_boolean, comparison::eq_dyn},
        record_batch::RecordBatch,
    };
    use std::sync::Arc;

    #[test]
    fn test_batch_column_convert() {
        let arrs: [ArrayRef; 2] = [
            Arc::new(Int64Array::from(vec![0, 1, 2, 3, 4])),
            Arc::new(StringArray::from(vec![
                None,
                Some("test"),
                Some("space"),
                None,
                Some("time!"),
            ])),
        ];
        let batch =
            RecordBatch::try_from_iter(vec![("col0", arrs[0].clone()), ("col1", arrs[1].clone())])
                .unwrap();
        for i in 0..2 {
            let actual = batch_column_to_columnar_value(&batch, i);
            let expected = ColumnarValue::Array(arrs[i].clone());
            // Check array equality
            match (actual, expected) {
                (ColumnarValue::Array(actual_array), ColumnarValue::Array(expected_array)) => {
                    let min = min_boolean(&eq_dyn(&*actual_array, &*expected_array).unwrap());
                    assert_eq!(Some(true), min);
                }
                _ => panic!("Either the expected ColumnarValue or the actual one is a Scalar!"),
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_invalid_batch_column_convert_bad_index() {
        let arr: ArrayRef = Arc::new(Int64Array::from(vec![0, 1, 2, 3, 4]));
        let batch = RecordBatch::try_from_iter(vec![("col", arr.clone())]).unwrap();
        batch_column_to_columnar_value(&batch, 1);
    }
}
