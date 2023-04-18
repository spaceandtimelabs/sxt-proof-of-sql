use arrow::array::{Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use polars::frame::DataFrame;
use polars::prelude::NamedFrom;
use polars::series::Series;
use std::sync::Arc;

/// Convert a RecordBatch to a polars DataFrame
pub fn record_batch_to_dataframe(record_batch: RecordBatch) -> DataFrame {
    let series: Vec<Series> = record_batch
        .schema()
        .fields()
        .iter()
        .zip(record_batch.columns().iter())
        .map(|(f, col)| match f.data_type() {
            arrow::datatypes::DataType::Int64 => {
                let data = col
                    .as_any()
                    .downcast_ref::<arrow::array::Int64Array>()
                    .map(|array| array.values())
                    .unwrap();

                Series::new(f.name(), data)
            }
            arrow::datatypes::DataType::Utf8 => {
                let data = col
                    .as_any()
                    .downcast_ref::<arrow::array::StringArray>()
                    .map(|array| (0..array.len()).map(|i| array.value(i)).collect::<Vec<_>>())
                    .unwrap();

                Series::new(f.name(), data)
            }
            _ => unimplemented!(),
        })
        .collect();

    DataFrame::new(series).unwrap()
}

/// Convert a polars DataFrame to a RecordBatch
pub fn dataframe_to_record_batch(data: DataFrame) -> RecordBatch {
    assert!(!data.is_empty());

    let mut column_fields: Vec<_> = Vec::with_capacity(data.width());
    let mut columns: Vec<Arc<dyn Array>> = Vec::with_capacity(data.width());

    for (field, series) in data.fields().iter().zip(data.get_columns().iter()) {
        let dt = match field.data_type() {
            polars::datatypes::DataType::Int64 => {
                let col = series.i64().unwrap().cont_slice().unwrap();

                columns.push(Arc::new(Int64Array::from(col.to_vec())));

                DataType::Int64
            }
            // This code handles a specific case where a Polars DataFrame has an unsigned 64-bit integer (u64) data type,
            // which only occurs when using the `count` function for aggregation.
            polars::datatypes::DataType::UInt64 => {
                // Retrieve the column as a contiguous slice of u64 values.
                let col = series.u64().unwrap().cont_slice().unwrap();

                // Cast the column to a supported i64 data type.
                // Note that this operation should never overflow
                // unless the database has around 2^64 rows, which is unfeasible.
                let col = col.iter().map(|v| *v as i64).collect::<Vec<_>>();

                columns.push(Arc::new(Int64Array::from(col)));

                DataType::Int64
            }
            polars::datatypes::DataType::Utf8 => {
                let col: Vec<_> = series
                    .utf8()
                    .unwrap()
                    .into_iter()
                    .map(|opt_v| opt_v.unwrap())
                    .collect();

                columns.push(Arc::new(StringArray::from(col)));

                DataType::Utf8
            }
            _ => unimplemented!("Datatype not supported: {:?}", field.data_type()),
        };

        column_fields.push(Field::new(field.name().as_str(), dt, false));
    }

    let schema = Arc::new(Schema::new(column_fields));

    RecordBatch::try_new(schema, columns).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record_batch;

    #[test]
    fn we_can_convert_dataframes_to_record_batches() {
        let dataframe =
            record_batch_to_dataframe(record_batch!("a" => [3214, 34], "bc" => ["a", "fg"]));
        assert_eq!(
            dataframe,
            polars::df!("a" => [3214_i64, 34_i64], "bc" => ["a", "fg"]).unwrap()
        );
    }

    #[test]
    fn we_can_convert_record_batches_to_dataframes() {
        let dataframe = dataframe_to_record_batch(
            polars::df!("a" => [3214_i64, 34_i64], "bc" => ["a", "fg"]).unwrap(),
        );
        assert_eq!(
            dataframe,
            record_batch!("a" => [3214, 34], "bc" => ["a", "fg"])
        );
    }
}
