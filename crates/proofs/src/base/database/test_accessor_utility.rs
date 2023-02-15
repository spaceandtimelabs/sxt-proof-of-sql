use crate::base::database::ColumnField;
use crate::base::database::{TestAccessorColumn, TestAccessorColumns};

use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use polars::frame::DataFrame;
use polars::prelude::NamedFrom;
use polars::series::Series;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use std::sync::Arc;

/// Specify what form a randomly generated TestAccessor can take
pub struct RandomTestAccessorDescriptor {
    pub min_rows: usize,
    pub max_rows: usize,
    pub min_value: i64,
    pub max_value: i64,
}

impl Default for RandomTestAccessorDescriptor {
    fn default() -> Self {
        Self {
            min_rows: 0,
            max_rows: 100,
            min_value: -5,
            max_value: 5,
        }
    }
}

/// Generate a DataFrame with random data
pub fn make_random_test_accessor_data(
    rng: &mut StdRng,
    cols: &[&str],
    descriptor: &RandomTestAccessorDescriptor,
) -> DataFrame {
    let n = Uniform::new(descriptor.min_rows, descriptor.max_rows + 1).sample(rng);
    let dist = Uniform::new(descriptor.min_value, descriptor.max_value + 1);
    let mut series = Vec::new();
    for col in cols {
        let v: Vec<i64> = dist.sample_iter(&mut *rng).take(n).collect();
        let v = Series::new(col, &v[..]);
        series.push(v)
    }
    DataFrame::new(series).unwrap()
}

/// Convert a polars DataFrame to a TestAccessorColumns
pub fn data_frame_to_accessors(data: &DataFrame) -> (usize, TestAccessorColumns) {
    assert!(!data.is_empty());

    let mut columns = TestAccessorColumns::new();
    let table_length = data.iter().next().unwrap().len();

    for field_series in data.fields().iter().zip(data.get_columns().iter()) {
        let accessor_col: TestAccessorColumn = field_series.into();
        let field_name = field_series.0.name().parse().unwrap();

        columns.insert(field_name, accessor_col);
    }

    (table_length, columns)
}

/// Convert a polars DataFrame to a RecordBatch
pub fn data_frame_to_record_batch(data: &DataFrame) -> RecordBatch {
    let (_, accessors) = data_frame_to_accessors(data);

    let schema: Vec<_> = accessors
        .iter()
        .map(|(k, v)| {
            let col_type = v.column_type();
            (&ColumnField::new(*k, col_type)).into()
        })
        .collect();

    let res = accessors.iter().map(|(_k, v)| v.to_arrow()).collect();

    RecordBatch::try_new(Arc::new(Schema::new(schema)), res).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::ToScalar;
    use arrow::array::{Array, Int64Array};
    use polars::prelude::*;
    use rand_core::SeedableRng;

    #[test]
    fn we_can_construct_a_random_test_data() {
        let descriptor = RandomTestAccessorDescriptor::default();
        let mut rng = StdRng::from_seed([0u8; 32]);
        let cols = ["a", "b"];

        // zero offset generators
        let data1 = make_random_test_accessor_data(&mut rng, &cols, &descriptor);
        let data2 = make_random_test_accessor_data(&mut rng, &cols, &descriptor);
        assert_ne!(
            data1.iter().next().unwrap().len(),
            data2.iter().next().unwrap().len()
        );
    }

    #[test]
    fn we_can_convert_data_frames_to_accessors() {
        let data_int = vec![1, 2, 3];
        let data_str = vec!["abc", "de", "t"];

        let data = df!(
            "a" => data_int.to_vec(),
            "b" => data_str.to_vec(),
        )
        .unwrap();

        let data_str_scalars: Vec<_> = data_str.iter().map(|v| v.to_scalar()).collect();
        let data_str: Vec<_> = data_str.into_iter().map(String::from).collect();

        let (length, accessors) = data_frame_to_accessors(&data);
        let expected_accessors = TestAccessorColumns::from([
            ("a".parse().unwrap(), TestAccessorColumn::BigInt(data_int)),
            (
                "b".parse().unwrap(),
                TestAccessorColumn::VarChar((data_str, data_str_scalars)),
            ),
        ]);
        assert_eq!(length, 3);
        assert_eq!(accessors, expected_accessors);
    }

    #[test]
    fn we_can_convert_data_frames_to_record_batches() {
        let data_int = vec![1, 2, 3];
        // let data_str = vec!["abc", "de", "t"]; // TODO: add this line when Column::String is supported

        let data = df!(
            "a" => data_int.to_vec(),
            // "b" => data_str.to_vec(), // TODO: add this line when Column::String is supported
        )
        .unwrap();

        let record_batch = data_frame_to_record_batch(&data);
        let columns: Vec<Arc<dyn Array>> = vec![
            Arc::new(Int64Array::from(data_int)),
            // Arc::new(StringArray::from(data_str)) // TODO: add this line when Column::String is supported
        ];
        let column_fields = vec![
            arrow::datatypes::Field::new("a", arrow::datatypes::DataType::Int64, false),
            // arrow::datatypes::Field::new("b", arrow::datatypes::DataType::Utf8, false), // TODO: add this line when Column::String is supported
        ];
        let schema = Arc::new(arrow::datatypes::Schema::new(column_fields));
        let expected_record_batch = RecordBatch::try_new(schema, columns).unwrap();

        assert_eq!(record_batch, expected_record_batch);
    }
}
