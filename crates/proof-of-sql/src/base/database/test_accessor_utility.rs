use crate::base::database::ColumnType;
use arrow::{
    array::{
        Array, BooleanArray, Decimal128Array, Decimal256Array, Int16Array, Int32Array, Int64Array,
        Int8Array, StringArray, TimestampMicrosecondArray, TimestampMillisecondArray,
        TimestampNanosecondArray, TimestampSecondArray,
    },
    datatypes::{i256, DataType, Field, Schema, TimeUnit},
    record_batch::RecordBatch,
};
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use std::sync::Arc;

/// Specify what form a randomly generated `TestAccessor` can take
pub struct RandomTestAccessorDescriptor {
    /// The minimum number of rows in the generated `RecordBatch`
    pub min_rows: usize,
    /// The maximum number of rows in the generated `RecordBatch`
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

/// Generate a `DataFrame` with random data
///
/// # Panics
///
/// This function may panic in the following cases:
/// - If `Precision::new(7)` fails when creating a `Decimal75` column type, which would occur
///   if the precision is invalid.
/// - When calling `.unwrap()` on the result of `RecordBatch::try_new(schema, columns)`, which
///   will panic if the schema and columns do not align correctly or if there are any other
///   underlying errors.
#[allow(dead_code)]
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
            ColumnType::Boolean => {
                column_fields.push(Field::new(*col_name, DataType::Boolean, false));
                let boolean_values: Vec<bool> = values.iter().map(|x| x % 2 != 0).collect();
                columns.push(Arc::new(BooleanArray::from(boolean_values)));
            }
            ColumnType::TinyInt => {
                column_fields.push(Field::new(*col_name, DataType::Int8, false));
                let values: Vec<i8> = values
                    .iter()
                    .map(|x| ((*x >> 56) as i8)) // Shift right to align the lower 8 bits
                    .collect();
                columns.push(Arc::new(Int8Array::from(values)));
            }
            ColumnType::SmallInt => {
                column_fields.push(Field::new(*col_name, DataType::Int16, false));
                let values: Vec<i16> = values
                    .iter()
                    .map(|x| ((*x >> 48) as i16)) // Shift right to align the lower 16 bits
                    .collect();
                columns.push(Arc::new(Int16Array::from(values)));
            }
            ColumnType::Int => {
                column_fields.push(Field::new(*col_name, DataType::Int32, false));
                let values: Vec<i32> = values
                    .iter()
                    .map(|x| ((*x >> 32) as i32)) // Shift right to align the lower 32 bits
                    .collect();
                columns.push(Arc::new(Int32Array::from(values)));
            }
            ColumnType::BigInt => {
                column_fields.push(Field::new(*col_name, DataType::Int64, false));
                let values: Vec<i64> = values.clone();
                columns.push(Arc::new(Int64Array::from(values)));
            }
            ColumnType::Int128 => {
                column_fields.push(Field::new(*col_name, DataType::Decimal128(38, 0), false));

                let values: Vec<i128> = values.iter().map(|x| i128::from(*x)).collect();
                columns.push(Arc::new(
                    Decimal128Array::from(values.clone())
                        .with_precision_and_scale(38, 0)
                        .unwrap(),
                ));
            }
            ColumnType::Decimal75(precision, scale) => {
                column_fields.push(Field::new(
                    *col_name,
                    DataType::Decimal256(precision.value(), *scale),
                    false,
                ));

                let values: Vec<i256> = values.iter().map(|x| i256::from(*x)).collect();
                columns.push(Arc::new(
                    Decimal256Array::from(values.clone())
                        .with_precision_and_scale(precision.value(), *scale)
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
            ColumnType::TimestampTZ(tu, tz) => {
                column_fields.push(Field::new(
                    *col_name,
                    DataType::Timestamp(
                        match tu {
                            PoSQLTimeUnit::Second => TimeUnit::Second,
                            PoSQLTimeUnit::Millisecond => TimeUnit::Millisecond,
                            PoSQLTimeUnit::Microsecond => TimeUnit::Microsecond,
                            PoSQLTimeUnit::Nanosecond => TimeUnit::Nanosecond,
                        },
                        Some(Arc::from(tz.to_string())),
                    ),
                    false,
                ));
                // Create the correct timestamp array based on the time unit
                let timestamp_array: Arc<dyn Array> = match tu {
                    PoSQLTimeUnit::Second => Arc::new(TimestampSecondArray::from(values.clone())),
                    PoSQLTimeUnit::Millisecond => {
                        Arc::new(TimestampMillisecondArray::from(values.clone()))
                    }
                    PoSQLTimeUnit::Microsecond => {
                        Arc::new(TimestampMicrosecondArray::from(values.clone()))
                    }
                    PoSQLTimeUnit::Nanosecond => {
                        Arc::new(TimestampNanosecondArray::from(values.clone()))
                    }
                };
                columns.push(timestamp_array);
            }
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
            ("d", ColumnType::SmallInt),
            ("e", ColumnType::Int),
            ("f", ColumnType::TinyInt),
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
