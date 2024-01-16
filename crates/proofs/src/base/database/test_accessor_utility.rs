use crate::base::database::ColumnType;
use arrow::{
    array::{Array, Decimal128Array, Int64Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use std::sync::Arc;

/// Specify what form a randomly generated TestAccessor can take
pub struct RandomTestAccessorDescriptor {
    /// The minimum number of rows in the generated RecordBatch
    pub min_rows: usize,
    /// The maximum number of rows in the generated RecordBatch
    pub max_rows: usize,
    /// The minimum value of the generated data
    pub min_value: i64,
    /// The maximum value of the generated data
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
    cols: &[(&str, ColumnType)],
    descriptor: &RandomTestAccessorDescriptor,
) -> RecordBatch {
    let n = Uniform::new(descriptor.min_rows, descriptor.max_rows + 1).sample(rng);
    let dist = Uniform::new(descriptor.min_value, descriptor.max_value + 1);

    let mut columns: Vec<Arc<dyn Array>> = Vec::with_capacity(n);
    let mut column_fields: Vec<_> = Vec::with_capacity(n);

    for (col_name, col_type) in cols {
        let values: Vec<i64> = dist.sample_iter(&mut *rng).take(n).collect();

        match col_type {
            ColumnType::BigInt => {
                column_fields.push(Field::new(*col_name, DataType::Int64, false));

                columns.push(Arc::new(Int64Array::from(values.to_vec())));
            }
            ColumnType::Int128 => {
                column_fields.push(Field::new(*col_name, DataType::Decimal128(38, 0), false));

                let values: Vec<i128> = values.iter().map(|x| *x as i128).collect();
                columns.push(Arc::new(
                    Decimal128Array::from(values.to_vec())
                        .with_precision_and_scale(38, 0)
                        .unwrap(),
                ));
            }
            ColumnType::VarChar => {
                let col = &values
                    .iter()
                    .map(|v| "s".to_owned() + &v.to_string()[..])
                    .collect::<Vec<String>>()[..];
                let col: Vec<_> = col.iter().map(|v| v.as_str()).collect();

                column_fields.push(Field::new(*col_name, DataType::Utf8, false));

                columns.push(Arc::new(StringArray::from(col)));
            }
            ColumnType::Scalar => unimplemented!("Scalar columns are not supported by arrow"),
        }
    }

    let schema = Arc::new(Schema::new(column_fields));
    RecordBatch::try_new(schema, columns).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record_batch;
    use rand_core::SeedableRng;

    #[test]
    fn we_can_construct_a_random_test_data() {
        let descriptor = RandomTestAccessorDescriptor::default();
        let mut rng = StdRng::from_seed([0u8; 32]);
        let cols = [
            ("a", ColumnType::BigInt),
            ("b", ColumnType::VarChar),
            ("c", ColumnType::Int128),
        ];

        let data1 = make_random_test_accessor_data(&mut rng, &cols, &descriptor);
        let data2 = make_random_test_accessor_data(&mut rng, &cols, &descriptor);
        assert_ne!(data1.num_rows(), data2.num_rows());
    }

    #[test]
    fn we_can_construct_a_random_test_data_with_the_correct_data() {
        let descriptor = RandomTestAccessorDescriptor {
            min_rows: 1,
            max_rows: 1,
            min_value: -2,
            max_value: -2,
        };
        let mut rng = StdRng::from_seed([0u8; 32]);
        let cols = [
            ("b", ColumnType::BigInt),
            ("a", ColumnType::VarChar),
            ("c", ColumnType::Int128),
        ];
        let data = make_random_test_accessor_data(&mut rng, &cols, &descriptor);

        assert_eq!(
            data,
            record_batch!("b" => [-2_i64], "a" => ["s-2"], "c" => [-2_i128])
        );
    }
}
