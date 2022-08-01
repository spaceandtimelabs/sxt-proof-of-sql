use datafusion::{arrow::record_batch::RecordBatch, physical_plan::ColumnarValue};

/// # Panics
///
/// Panics if `index` is outside of `0..num_columns`.
pub fn batch_column_to_columnar_value(batch: &RecordBatch, index: usize) -> ColumnarValue {
    ColumnarValue::Array(batch.column(index).clone())
}
