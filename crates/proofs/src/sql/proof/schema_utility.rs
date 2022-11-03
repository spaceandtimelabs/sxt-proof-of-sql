use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::Arc;

/// Helper function to create schemas for arrow RecordBatch's.
///
/// Note: We assume that every column is an int64 and the columns are named
/// with sequential integers, "1", "2", .... These assumptions won't be
/// reasonable for a production version of provable SQL, but they serve our
/// present needs for demoing a POC.
pub fn make_schema(num_columns: usize) -> SchemaRef {
    let mut columns = Vec::with_capacity(num_columns);
    for i in 0..num_columns {
        columns.push(Field::new(&(i + 1).to_string(), DataType::Int64, false));
    }
    Arc::new(Schema::new(columns))
}
