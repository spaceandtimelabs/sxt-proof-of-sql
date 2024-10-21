use crate::{
    base::commitment::{Commitment, TableCommitment},
    proof_primitive::dory::{DoryCommitment, DoryProverPublicSetup},
};
use arrow::{
    array::{
        Array, ArrayRef, ArrowPrimitiveType, BooleanArray, Decimal128Array, Decimal256Array,
        Decimal256Builder, Int16Array, Int32Array, Int64Array, Int8Array, PrimitiveArray,
        RecordBatch, StringArray, TimestampMicrosecondArray, TimestampMillisecondArray,
        TimestampNanosecondArray, TimestampSecondArray,
    },
    compute::{sort_to_indices, take},
    datatypes::{i256, DataType, Field, Schema, TimeUnit},
    error::ArrowError,
};
use core::str::FromStr;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use postcard::to_allocvec;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, sync::Arc};

static PARQUET_FILE_PROOF_ORDER_COLUMN: &str = "META_ROW_NUMBER";

/// Performs the following:
/// Reads a collection of parquet files which in aggregate represent a single table of data,
/// Calculates the `TableCommitment` for the table using multiple commitment strategies,
/// Serializes each commitment to a blob, which is saved in the same directory as the original parquet file
///
/// # Panics
///
/// Panics when any part of the process fails
pub fn read_parquet_file_to_commitment_as_blob(
    parquet_files: &Vec<PathBuf>,
    output_path_prefix: &str,
    prover_setup: &DoryProverPublicSetup,
    big_decimal_columns: &[(String, u8, i8)],
) {
    let mut commitments: Vec<TableCommitment<DoryCommitment>> = parquet_files
        .par_iter()
        .flat_map(|path| {
            println!("Committing to {}..", path.as_path().to_str().unwrap());
            let file = File::open(path).unwrap();
            let reader = ParquetRecordBatchReaderBuilder::try_new(file)
                .unwrap()
                .build()
                .unwrap();
            let record_batch_results: Vec<Result<RecordBatch, ArrowError>> = reader.collect();
            let record_batches: Vec<RecordBatch> = record_batch_results
                .into_iter()
                .map(|record_batch_result| {
                    sort_record_batch_by_meta_row_number(&record_batch_result.unwrap())
                })
                .collect();
            let schema = record_batches.first().unwrap().schema();
            println!(
                "File row COUNT: {}",
                record_batches
                    .iter()
                    .map(RecordBatch::num_rows)
                    .sum::<usize>()
            );
            let commitments: Vec<_> = record_batches
                .into_par_iter()
                .map(|mut unmodified_record_batch| {
                    let meta_row_number_column = unmodified_record_batch
                        .column_by_name(PARQUET_FILE_PROOF_ORDER_COLUMN)
                        .unwrap()
                        .as_any()
                        .downcast_ref::<Int32Array>()
                        .unwrap();

                    let offset = meta_row_number_column.value(0) - 1;
                    unmodified_record_batch
                        .remove_column(schema.index_of(PARQUET_FILE_PROOF_ORDER_COLUMN).unwrap());
                    let record_batch = replace_nulls_within_record_batch(&correct_utf8_fields(
                        &unmodified_record_batch,
                        big_decimal_columns.to_vec(),
                    ));
                    TableCommitment::<DoryCommitment>::try_from_record_batch_with_offset(
                        &record_batch,
                        offset as usize,
                        prover_setup,
                    )
                    .unwrap()
                })
                .collect();
            println!("Commitments generated");
            commitments
        })
        .collect();

    println!("done computing per-file commitments, now sorting and aggregating");
    commitments.sort_by(|commitment_a, commitment_b| {
        commitment_a.range().start.cmp(&commitment_b.range().start)
    });

    //aggregate_commitments_to_blob(unzipped.0, format!("{output_path_prefix}-dory-commitment"));
    aggregate_commitments_to_blob(
        commitments,
        &format!("{output_path_prefix}-dory-commitment"),
    );
}

/// # Panics
///
/// Panics when any part of the process fails
fn aggregate_commitments_to_blob<C: Commitment + Serialize + for<'a> Deserialize<'a>>(
    commitments: Vec<TableCommitment<C>>,
    output_file_base: &str,
) {
    let commitment = commitments
        .into_iter()
        .fold(
            None,
            |aggregate_commitment: Option<TableCommitment<C>>, next_commitment| {
                match aggregate_commitment {
                    Some(agg) => Some(agg.try_add(next_commitment).unwrap()),
                    None => Some(next_commitment),
                }
            },
        )
        .unwrap();
    write_commitment_to_blob(&commitment, output_file_base);
}

/// # Panics
fn write_commitment_to_blob<C: Commitment + Serialize + for<'a> Deserialize<'a>>(
    commitment: &TableCommitment<C>,
    output_file_base: &str,
) {
    let bytes: Vec<u8> = to_allocvec(commitment).unwrap();
    let path_extension = "txt";
    let mut output_file = File::create(format!("{output_file_base}.{path_extension}")).unwrap();
    output_file.write_all(&bytes).unwrap();
}

fn replace_nulls_primitive<T: ArrowPrimitiveType>(array: &PrimitiveArray<T>) -> PrimitiveArray<T> {
    PrimitiveArray::from_iter_values(
        array
            .iter()
            .map(|value: Option<<T as ArrowPrimitiveType>::Native>| value.unwrap_or_default()),
    )
}

/// # Panics
fn replace_nulls_within_record_batch(record_batch: &RecordBatch) -> RecordBatch {
    let schema = record_batch.schema();
    let new_columns: Vec<_> = record_batch
        .columns()
        .iter()
        .map(|column| {
            if column.is_nullable() {
                let column_type = column.data_type();
                let column: ArrayRef = match column_type {
                    DataType::Int8 => Arc::new(replace_nulls_primitive(
                        column.as_any().downcast_ref::<Int8Array>().unwrap(),
                    )),
                    DataType::Int16 => Arc::new(replace_nulls_primitive(
                        column.as_any().downcast_ref::<Int16Array>().unwrap(),
                    )),
                    DataType::Int32 => Arc::new(replace_nulls_primitive(
                        column.as_any().downcast_ref::<Int32Array>().unwrap(),
                    )),
                    DataType::Int64 => Arc::new(replace_nulls_primitive(
                        column.as_any().downcast_ref::<Int64Array>().unwrap(),
                    )),

                    DataType::Decimal128(precision, scale) => Arc::new(
                        replace_nulls_primitive(
                            column.as_any().downcast_ref::<Decimal128Array>().unwrap(),
                        )
                        .with_precision_and_scale(*precision, *scale)
                        .unwrap(),
                    ),
                    DataType::Decimal256(precision, scale) => Arc::new(
                        replace_nulls_primitive(
                            column.as_any().downcast_ref::<Decimal256Array>().unwrap(),
                        )
                        .with_precision_and_scale(*precision, *scale)
                        .unwrap(),
                    ),
                    DataType::Timestamp(TimeUnit::Second, timezone) => Arc::new(
                        replace_nulls_primitive(
                            column
                                .as_any()
                                .downcast_ref::<TimestampSecondArray>()
                                .unwrap(),
                        )
                        .with_timezone_opt(timezone.clone()),
                    ),
                    DataType::Timestamp(TimeUnit::Millisecond, timezone) => Arc::new(
                        replace_nulls_primitive(
                            column
                                .as_any()
                                .downcast_ref::<TimestampMillisecondArray>()
                                .unwrap(),
                        )
                        .with_timezone_opt(timezone.clone()),
                    ),
                    DataType::Timestamp(TimeUnit::Microsecond, timezone) => Arc::new(
                        replace_nulls_primitive(
                            column
                                .as_any()
                                .downcast_ref::<TimestampMicrosecondArray>()
                                .unwrap(),
                        )
                        .with_timezone_opt(timezone.clone()),
                    ),
                    DataType::Timestamp(TimeUnit::Nanosecond, timezone) => Arc::new(
                        replace_nulls_primitive(
                            column
                                .as_any()
                                .downcast_ref::<TimestampNanosecondArray>()
                                .unwrap(),
                        )
                        .with_timezone_opt(timezone.clone()),
                    ),
                    DataType::Boolean => Arc::new(
                        column
                            .as_any()
                            .downcast_ref::<BooleanArray>()
                            .unwrap()
                            .iter()
                            .map(|element| Some(element.unwrap_or(false)))
                            .collect::<BooleanArray>(),
                    ),
                    DataType::Utf8 => Arc::new(StringArray::from_iter_values(
                        column
                            .as_any()
                            .downcast_ref::<StringArray>()
                            .unwrap()
                            .iter()
                            .map(|element| element.unwrap_or("")),
                    )),
                    _ => unimplemented!(),
                };

                column
            } else {
                column.clone()
            }
        })
        .collect();
    RecordBatch::try_new(schema, new_columns).unwrap()
}

/// # Panics
fn sort_record_batch_by_meta_row_number(record_batch: &RecordBatch) -> RecordBatch {
    let schema = record_batch.schema();
    let indices = sort_to_indices(
        record_batch
            .column_by_name(PARQUET_FILE_PROOF_ORDER_COLUMN)
            .unwrap(),
        None,
        None,
    )
    .unwrap();
    let columns = record_batch
        .columns()
        .iter()
        .map(|c| take(c, &indices, None).unwrap())
        .collect();
    RecordBatch::try_new(schema, columns).unwrap()
}

/// # Panics
fn cast_string_array_to_decimal256_array(
    string_array: &[Option<String>],
    precision: u8,
    scale: i8,
) -> Decimal256Array {
    let mut builder =
        Decimal256Builder::default().with_data_type(DataType::Decimal256(precision, scale));

    string_array.iter().for_each(|value| match value {
        Some(v) => {
            let decimal_value = f64::from_str(v).expect("Invalid number");
            let scaled_value = decimal_value * 10f64.powi(i32::from(scale));
            builder.append_value(i256::from_f64(scaled_value).unwrap());
        }
        None => builder.append_null(),
    });

    builder.finish()
}

/// # Panics
fn correct_utf8_fields(
    record_batch: &RecordBatch,
    big_decimal_columns: Vec<(String, u8, i8)>,
) -> RecordBatch {
    let big_decimal_columns_lookup: HashMap<String, (u8, i8)> = big_decimal_columns
        .into_iter()
        .map(|(key, precision, scale)| (key, (precision, scale)))
        .collect();
    let schema = record_batch.schema();

    // Replace StringArray columns as appropriate
    let columns: Vec<Arc<dyn Array>> = record_batch
        .columns()
        .iter()
        .zip(schema.fields().iter())
        .map(|(pointer_column, field)| {
            let column = pointer_column.clone();
            let column_name = field.name().to_lowercase();
            if field.data_type() == &DataType::Utf8 {
                let string_vec: Vec<Option<String>> = column
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .into_iter()
                    .map(|s| s.map(|st| st.replace("\0", "")))
                    .collect();
                big_decimal_columns_lookup
                    .get(&column_name)
                    .map(|(precision, scale)| {
                        Arc::new(cast_string_array_to_decimal256_array(
                            &string_vec,
                            *precision,
                            *scale,
                        )) as ArrayRef
                    })
                    .unwrap_or(Arc::new(StringArray::from(string_vec)))
            } else {
                Arc::new(column)
            }
        })
        .collect();

    // Replace Utf8 fields with Decimal256 for the big_decimal columns
    let fields: Vec<Arc<Field>> = schema
        .fields()
        .iter()
        .map(|field| {
            if field.data_type() == &DataType::Utf8 {
                big_decimal_columns_lookup
                    .get(&field.name().to_lowercase())
                    .map(|(precision, scale)| {
                        Arc::new(Field::new(
                            field.name(),
                            DataType::Decimal256(*precision, *scale),
                            field.is_nullable(),
                        ))
                    })
                    .unwrap_or(field.clone())
            } else {
                field.clone()
            }
        })
        .collect();
    let new_schema = Schema::new(fields);
    RecordBatch::try_new(new_schema.into(), columns).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::{
        base::commitment::{Commitment, TableCommitment},
        proof_primitive::dory::{
            DoryCommitment, DoryProverPublicSetup, ProverSetup, PublicParameters,
        },
        utils::parquet_to_commitment_blob::{
            correct_utf8_fields, read_parquet_file_to_commitment_as_blob,
            replace_nulls_within_record_batch, PARQUET_FILE_PROOF_ORDER_COLUMN,
        },
    };
    use arrow::{
        array::{
            ArrayRef, ArrowPrimitiveType, BooleanArray, Decimal128Array, Decimal256Builder,
            Int16Array, Int32Array, Int64Array, Int8Array, RecordBatch, StringArray,
            TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
            TimestampSecondArray,
        },
        datatypes::{
            i256, DataType, Decimal128Type, Field, Int16Type, Int32Type, Int64Type, Int8Type,
            Schema, TimestampMicrosecondType, TimestampMillisecondType, TimestampNanosecondType,
            TimestampSecondType,
        },
    };
    use parquet::{arrow::ArrowWriter, basic::Compression, file::properties::WriterProperties};
    use postcard::from_bytes;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use serde::{Deserialize, Serialize};
    use std::{
        fs::{self, File},
        io::Read,
        panic,
        path::Path,
        sync::Arc,
    };

    fn create_mock_file_from_record_batch(path: &str, record_batch: &RecordBatch) {
        let parquet_file = File::create(path).unwrap();
        let writer_properties = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();
        let mut writer =
            ArrowWriter::try_new(parquet_file, record_batch.schema(), Some(writer_properties))
                .unwrap();
        writer.write(record_batch).unwrap();
        writer.close().unwrap();
    }

    fn read_commitment_from_blob<C: Commitment + Serialize + for<'a> Deserialize<'a>>(
        path: &str,
    ) -> TableCommitment<C> {
        let mut blob_file = File::open(path).unwrap();
        let mut bytes: Vec<u8> = Vec::new();
        blob_file.read_to_end(&mut bytes).unwrap();
        from_bytes(&bytes).unwrap()
    }

    fn delete_file_if_exists(path: &str) {
        if Path::new(path).exists() {
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn we_can_replace_nulls() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("utf8", DataType::Utf8, true),
            Field::new("boolean", DataType::Boolean, true),
            Field::new(
                "timestamp_second",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Second, None),
                true,
            ),
            Field::new(
                "timestamp_millisecond",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None),
                true,
            ),
            Field::new(
                "timestamp_microsecond",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None),
                true,
            ),
            Field::new(
                "timestamp_nanosecond",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new("decimal128", DataType::Decimal128(38, 10), true),
            Field::new("int64", DataType::Int64, true),
            Field::new("int32", DataType::Int32, true),
            Field::new("int16", DataType::Int16, true),
            Field::new("int8", DataType::Int8, true),
        ]));

        let utf8 = Arc::new(StringArray::from(vec![
            Some("a"),
            None,
            Some("c"),
            Some("d"),
            None,
        ])) as ArrayRef;
        let utf8_denulled = Arc::new(StringArray::from(vec![
            Some("a"),
            Some(""),
            Some("c"),
            Some("d"),
            Some(""),
        ])) as ArrayRef;

        let boolean = Arc::new(BooleanArray::from(vec![
            Some(true),
            None,
            Some(false),
            Some(true),
            None,
        ])) as ArrayRef;
        let boolean_denulled = Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(false),
            Some(false),
            Some(true),
            Some(false),
        ])) as ArrayRef;

        let timestamp_second = Arc::new(TimestampSecondArray::from(vec![
            Some(1_627_846_260),
            None,
            Some(1_627_846_262),
            Some(1_627_846_263),
            None,
        ])) as ArrayRef;
        let timestamp_second_denulled = Arc::new(TimestampSecondArray::from(vec![
            Some(1_627_846_260),
            Some(TimestampSecondType::default_value()),
            Some(1_627_846_262),
            Some(1_627_846_263),
            Some(TimestampSecondType::default_value()),
        ])) as ArrayRef;

        let timestamp_millisecond = Arc::new(TimestampMillisecondArray::from(vec![
            Some(1_627_846_260_000),
            None,
            Some(1_627_846_262_000),
            Some(1_627_846_263_000),
            None,
        ])) as ArrayRef;
        let timestamp_millisecond_denulled = Arc::new(TimestampMillisecondArray::from(vec![
            Some(1_627_846_260_000),
            Some(TimestampMillisecondType::default_value()),
            Some(1_627_846_262_000),
            Some(1_627_846_263_000),
            Some(TimestampMillisecondType::default_value()),
        ])) as ArrayRef;

        let timestamp_microsecond = Arc::new(TimestampMicrosecondArray::from(vec![
            Some(1_627_846_260_000_000),
            None,
            Some(1_627_846_262_000_000),
            Some(1_627_846_263_000_000),
            None,
        ])) as ArrayRef;
        let timestamp_microsecond_denulled = Arc::new(TimestampMicrosecondArray::from(vec![
            Some(1_627_846_260_000_000),
            Some(TimestampMicrosecondType::default_value()),
            Some(1_627_846_262_000_000),
            Some(1_627_846_263_000_000),
            Some(TimestampMicrosecondType::default_value()),
        ])) as ArrayRef;

        let timestamp_nanosecond = Arc::new(TimestampNanosecondArray::from(vec![
            Some(1_627_846_260_000_000_000),
            None,
            Some(1_627_846_262_000_000_000),
            Some(1_627_846_263_000_000_000),
            None,
        ])) as ArrayRef;
        let timestamp_nanosecond_denulled = Arc::new(TimestampNanosecondArray::from(vec![
            Some(1_627_846_260_000_000_000),
            Some(TimestampNanosecondType::default_value()),
            Some(1_627_846_262_000_000_000),
            Some(1_627_846_263_000_000_000),
            Some(TimestampNanosecondType::default_value()),
        ])) as ArrayRef;

        let decimal128 = Arc::new(Decimal128Array::from(vec![
            Some(12_345_678_901_234_567_890_i128),
            None,
            Some(23_456_789_012_345_678_901_i128),
            Some(34_567_890_123_456_789_012_i128),
            None,
        ])) as ArrayRef;
        let decimal128_denulled = Arc::new(Decimal128Array::from(vec![
            Some(12_345_678_901_234_567_890_i128),
            Some(Decimal128Type::default_value()),
            Some(23_456_789_012_345_678_901_i128),
            Some(34_567_890_123_456_789_012_i128),
            Some(Decimal128Type::default_value()),
        ])) as ArrayRef;

        let int64 = Arc::new(Int64Array::from(vec![
            Some(1),
            None,
            Some(3),
            Some(4),
            None,
        ])) as ArrayRef;
        let int64_denulled = Arc::new(Int64Array::from(vec![
            Some(1),
            Some(Int64Type::default_value()),
            Some(3),
            Some(4),
            Some(Int64Type::default_value()),
        ])) as ArrayRef;

        let int32 = Arc::new(Int32Array::from(vec![
            Some(1),
            None,
            Some(3),
            Some(4),
            None,
        ])) as ArrayRef;
        let int32_denulled = Arc::new(Int32Array::from(vec![
            Some(1),
            Some(Int32Type::default_value()),
            Some(3),
            Some(4),
            Some(Int32Type::default_value()),
        ])) as ArrayRef;

        let int16 = Arc::new(Int16Array::from(vec![
            Some(1),
            None,
            Some(3),
            Some(4),
            None,
        ])) as ArrayRef;
        let int16_denulled = Arc::new(Int16Array::from(vec![
            Some(1),
            Some(Int16Type::default_value()),
            Some(3),
            Some(4),
            Some(Int16Type::default_value()),
        ])) as ArrayRef;

        let int8 =
            Arc::new(Int8Array::from(vec![Some(1), None, Some(3), Some(4), None])) as ArrayRef;
        let int8_denulled = Arc::new(Int8Array::from(vec![
            Some(1),
            Some(Int8Type::default_value()),
            Some(3),
            Some(4),
            Some(Int8Type::default_value()),
        ])) as ArrayRef;

        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                utf8,
                boolean,
                timestamp_second,
                timestamp_millisecond,
                timestamp_microsecond,
                timestamp_nanosecond,
                decimal128,
                int64,
                int32,
                int16,
                int8,
            ],
        )
        .unwrap();
        let record_batch_denulled = RecordBatch::try_new(
            schema,
            vec![
                utf8_denulled,
                boolean_denulled,
                timestamp_second_denulled,
                timestamp_millisecond_denulled,
                timestamp_microsecond_denulled,
                timestamp_nanosecond_denulled,
                decimal128_denulled,
                int64_denulled,
                int32_denulled,
                int16_denulled,
                int8_denulled,
            ],
        )
        .unwrap();

        let null_replaced_batch = replace_nulls_within_record_batch(&record_batch);
        assert_eq!(null_replaced_batch, record_batch_denulled);
    }

    #[test]
    fn we_can_correct_utf8_columns() {
        let original_schema = Arc::new(Schema::new(vec![
            Arc::new(Field::new("nullable_regular_string", DataType::Utf8, true)),
            Arc::new(Field::new("nullable_big_decimal", DataType::Utf8, true)),
            Arc::new(Field::new("not_null_regular_string", DataType::Utf8, false)),
            Arc::new(Field::new("not_null_big_decimal", DataType::Utf8, false)),
            Arc::new(Field::new("nullable_int", DataType::Int32, true)),
            Arc::new(Field::new("not_null_int", DataType::Int32, false)),
        ]));
        let corrected_schema = Arc::new(Schema::new(vec![
            Arc::new(Field::new("nullable_regular_string", DataType::Utf8, true)),
            Arc::new(Field::new(
                "nullable_big_decimal",
                DataType::Decimal256(25, 4),
                true,
            )),
            Arc::new(Field::new("not_null_regular_string", DataType::Utf8, false)),
            Arc::new(Field::new(
                "not_null_big_decimal",
                DataType::Decimal256(25, 4),
                false,
            )),
            Arc::new(Field::new("nullable_int", DataType::Int32, true)),
            Arc::new(Field::new("not_null_int", DataType::Int32, false)),
        ]));

        let original_nullable_regular_string_array: ArrayRef = Arc::new(StringArray::from(vec![
            None,
            Some("Bob"),
            Some("Char\0lie"),
            None,
            Some("Eve"),
        ]));
        let corrected_nullable_regular_string_array: ArrayRef = Arc::new(StringArray::from(vec![
            None,
            Some("Bob"),
            Some("Charlie"),
            None,
            Some("Eve"),
        ]));
        let original_nullable_big_decimal_array: ArrayRef = Arc::new(StringArray::from(vec![
            Some("1234.56"),
            None,
            Some("45321E6"),
            Some("123e4"),
            None,
        ]));
        let mut corrected_nullable_big_decimal_array_builder =
            Decimal256Builder::default().with_data_type(DataType::Decimal256(25, 4));
        corrected_nullable_big_decimal_array_builder.append_option(Some(i256::from(12_345_600)));
        corrected_nullable_big_decimal_array_builder.append_null();
        corrected_nullable_big_decimal_array_builder
            .append_option(Some(i256::from(453_210_000_000_000i64)));
        corrected_nullable_big_decimal_array_builder
            .append_option(Some(i256::from(12_300_000_000i64)));
        corrected_nullable_big_decimal_array_builder.append_null();
        let corrected_nullable_big_decimal_array: ArrayRef =
            Arc::new(corrected_nullable_big_decimal_array_builder.finish());
        let original_not_null_regular_string_array: ArrayRef =
            Arc::new(StringArray::from(vec!["A", "B", "C\0", "D", "E"]));
        let corrected_not_null_regular_string_array: ArrayRef =
            Arc::new(StringArray::from(vec!["A", "B", "C", "D", "E"]));
        let original_not_null_big_decimal_array: ArrayRef =
            Arc::new(StringArray::from(vec!["1", "2.34", "5e6", "12", "1E4"]));
        let mut corrected_not_null_big_decimal_array_builder =
            Decimal256Builder::default().with_data_type(DataType::Decimal256(25, 4));
        corrected_not_null_big_decimal_array_builder.append_value(i256::from(10_000));
        corrected_not_null_big_decimal_array_builder.append_value(i256::from(23_400));
        corrected_not_null_big_decimal_array_builder.append_value(i256::from(50_000_000_000i64));
        corrected_not_null_big_decimal_array_builder.append_value(i256::from(120_000));
        corrected_not_null_big_decimal_array_builder.append_value(i256::from(100_000_000));
        let corrected_not_null_big_decimal_array: ArrayRef =
            Arc::new(corrected_not_null_big_decimal_array_builder.finish());

        let nullable_int_array: ArrayRef = Arc::new(Int32Array::from(vec![
            Some(10),
            None,
            Some(30),
            Some(40),
            None,
        ]));
        let not_null_int_array: ArrayRef = Arc::new(Int32Array::from(vec![1, 2, 3, 4, 5]));

        let original_record_batch = RecordBatch::try_new(
            original_schema,
            vec![
                original_nullable_regular_string_array,
                original_nullable_big_decimal_array,
                original_not_null_regular_string_array,
                original_not_null_big_decimal_array,
                nullable_int_array.clone(),
                not_null_int_array.clone(),
            ],
        )
        .unwrap();

        let expected_corrected_record_batch = RecordBatch::try_new(
            corrected_schema,
            vec![
                corrected_nullable_regular_string_array,
                corrected_nullable_big_decimal_array,
                corrected_not_null_regular_string_array,
                corrected_not_null_big_decimal_array,
                nullable_int_array,
                not_null_int_array,
            ],
        )
        .unwrap();

        let big_decimal_columns = vec![
            ("nullable_big_decimal".to_string(), 25, 4),
            ("not_null_big_decimal".to_string(), 25, 4),
        ];
        let corrected_record_batch =
            correct_utf8_fields(&original_record_batch, big_decimal_columns);

        assert_eq!(corrected_record_batch, expected_corrected_record_batch);
    }

    #[test]
    fn we_can_fail_if_datatype_of_big_decimal_column_is_not_string() {
        let err = panic::catch_unwind(|| {
            let string_array: ArrayRef = Arc::new(StringArray::from(vec![
                None,
                Some("123"),
                Some("345"),
                None,
                Some("567"),
            ]));
            let schema = Arc::new(Schema::new(vec![Arc::new(Field::new(
                "nullable_big_decimal",
                DataType::Int16,
                true,
            ))]));
            let record_batch = RecordBatch::try_new(schema, vec![string_array]).unwrap();
            let big_decimal_columns = vec![("nullable_big_decimal".to_string(), 25, 4)];
            let _test = correct_utf8_fields(&record_batch, big_decimal_columns);
        });
        assert!(err.is_err());
    }

    #[test]
    fn we_can_fail_if_big_decimal_column_is_not_castable() {
        let err = panic::catch_unwind(|| {
            let string_array: ArrayRef = Arc::new(StringArray::from(vec![
                None,
                Some("Bob"),
                Some("Charlie"),
                None,
                Some("Eve"),
            ]));
            let schema = Arc::new(Schema::new(vec![Arc::new(Field::new(
                "nullable_regular_string",
                DataType::Utf8,
                true,
            ))]));
            let record_batch = RecordBatch::try_new(schema, vec![string_array]).unwrap();
            let big_decimal_columns = vec![("nullable_regular_string".to_string(), 25, 4)];
            let _test = correct_utf8_fields(&record_batch, big_decimal_columns);
        });
        assert!(err.is_err());
    }

    #[test]
    fn we_can_retrieve_commitments_and_save_to_file() {
        let parquet_path_1 = "example-1.parquet";
        let parquet_path_2 = "example-2.parquet";
        let dory_commitment_path = "example-dory-commitment.txt";
        delete_file_if_exists(parquet_path_1);
        delete_file_if_exists(parquet_path_2);
        delete_file_if_exists(dory_commitment_path);
        let proof_column_1 = Int32Array::from(vec![1, 2]);
        let column_1 = Int32Array::from(vec![2, 1]);
        let proof_column_2 = Int32Array::from(vec![3, 4]);
        let column_2 = Int32Array::from(vec![3, 4]);
        let column = Int32Array::from(vec![2, 1, 3, 4]);
        let record_batch_1 = RecordBatch::try_from_iter(vec![
            (
                PARQUET_FILE_PROOF_ORDER_COLUMN,
                Arc::new(proof_column_1) as ArrayRef,
            ),
            ("column", Arc::new(column_1) as ArrayRef),
        ])
        .unwrap();
        let record_batch_2 = RecordBatch::try_from_iter(vec![
            (
                PARQUET_FILE_PROOF_ORDER_COLUMN,
                Arc::new(proof_column_2) as ArrayRef,
            ),
            ("column", Arc::new(column_2) as ArrayRef),
        ])
        .unwrap();
        let record_batch =
            RecordBatch::try_from_iter(vec![("column", Arc::new(column) as ArrayRef)]).unwrap();
        create_mock_file_from_record_batch(parquet_path_1, &record_batch_1);
        create_mock_file_from_record_batch(parquet_path_2, &record_batch_2);
        let setup_seed = "SpaceAndTime".to_string();
        let mut rng = {
            // Convert the seed string to bytes and create a seeded RNG
            let seed_bytes = setup_seed
                .bytes()
                .chain(std::iter::repeat(0u8))
                .take(32)
                .collect::<Vec<_>>()
                .try_into()
                .expect("collection is guaranteed to contain 32 elements");
            ChaCha20Rng::from_seed(seed_bytes) // Seed ChaChaRng
        };
        let public_parameters = PublicParameters::rand(4, &mut rng);
        let prover_setup = ProverSetup::from(&public_parameters);
        let dory_prover_setup: DoryProverPublicSetup = DoryProverPublicSetup::new(&prover_setup, 3);
        read_parquet_file_to_commitment_as_blob(
            &vec![parquet_path_1.into(), parquet_path_2.into()],
            "example",
            &dory_prover_setup,
            &Vec::new(),
        );
        assert_eq!(
            read_commitment_from_blob::<DoryCommitment>(dory_commitment_path),
            TableCommitment::<DoryCommitment>::try_from_record_batch(
                &record_batch,
                &dory_prover_setup
            )
            .unwrap()
        );
        delete_file_if_exists(parquet_path_1);
        delete_file_if_exists(parquet_path_2);
        delete_file_if_exists(dory_commitment_path);
    }
}
