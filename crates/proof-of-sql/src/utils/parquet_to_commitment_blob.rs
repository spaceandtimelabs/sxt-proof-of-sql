use crate::{
    base::commitment::{Commitment, TableCommitment},
    proof_primitive::dory::{DoryCommitment, DoryProverPublicSetup, DynamicDoryCommitment, ProverSetup}, utils::{decimal_precision::column_clamp_precision, parse_decimals::column_parse_decimals_fallible, record_batch_map::{record_batch_map, record_batch_try_map_with_target_types}},
};
use arrow::{
    array::{
        Array, ArrayRef, Decimal256Array, Decimal256Builder, Int32Array, RecordBatch, StringArray,
    },
    compute::{sort_to_indices, take},
    datatypes::{i256, DataType, Field, Schema},
    error::ArrowError, record_batch,
};
use indexmap::IndexMap;
use core::str::FromStr;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use postcard::to_allocvec;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, sync::Arc};
use sqlparser::ast::{DataType as SqlparserDataType, ExactNumberInfo};

static PARQUET_FILE_PROOF_ORDER_COLUMN: &str = "META_ROW_NUMBER";

/// Performs the following:
/// Reads a collection of parquet files which in aggregate represent a single range from a table of data,
/// Calculates the `TableCommitment` for the table using one or more commitment strategies,
/// Serializes each commitment to a blob
///
/// # Panics
///
/// Panics when any part of the process fails
pub fn convert_historical_parquet_file_to_commitment_blob(
    parquet_files: &Vec<PathBuf>,
    output_path_prefix: &str,
    prover_setup: &ProverSetup,
    target_types: &IndexMap<String, SqlparserDataType>,
) {
    // Compute and collect TableCommitments per RecordBatch per file.
    let mut commitments: Vec<TableCommitment<DynamicDoryCommitment>> = parquet_files
        .par_iter()
        .flat_map(|path| {
            println!("Committing to {}..", path.as_path().to_str().unwrap());

            // Collect RecordBatches from file
            let file = File::open(path).unwrap();
            let reader = ParquetRecordBatchReaderBuilder::try_new(file)
                .unwrap()
                .build()
                .unwrap();
            let record_batch_results: Vec<Result<RecordBatch, ArrowError>> = reader.collect();
            let record_batches: Vec<RecordBatch> = record_batch_results
                .into_iter()
                .map(|record_batch_result| {
                    // Sorting can probably be removed
                    sort_record_batch_by_meta_row_number(&record_batch_result.unwrap())
                })
                .collect();

            // Compute and collect the TableCommitments for each RecordBatch in the file.
            let schema = record_batches.first().unwrap().schema();
            let commitments: Vec<_> = record_batches
                .into_par_iter()
                .map(|mut record_batch| {
                    // We use the proof column only to identify the offset used to compute the commitments. It can be removed afterward.
                    let meta_row_number_column = record_batch
                        .column_by_name(PARQUET_FILE_PROOF_ORDER_COLUMN)
                        .unwrap()
                        .as_any()
                        .downcast_ref::<Int32Array>()
                        .unwrap();
                    let offset = meta_row_number_column.value(0) - 1;
                    record_batch
                        .remove_column(schema.index_of(PARQUET_FILE_PROOF_ORDER_COLUMN).unwrap());

                    // Replace appropriate string columns with decimal columns.
                    let record_batch =
                    record_batch_try_map_with_target_types(
                            record_batch,
                            target_types,
                            column_parse_decimals_fallible
                        ).unwrap();
                    let record_batch = record_batch_map(record_batch, column_clamp_precision);

                    // Calculate and return TableCommitment
                    TableCommitment::<DynamicDoryCommitment>::try_from_record_batch_with_offset(
                        &record_batch,
                        offset as usize,
                        &prover_setup,
                    )
                    .unwrap()
                })
                .collect();
            commitments
        })
        .collect();

    println!("done computing per-file commitments, now sorting and aggregating");

    // We sort the TableCommitment collections in order to avoid non-contiguous errors.
    commitments.sort_by(|commitment_a, commitment_b| {
        commitment_a.range().start.cmp(&commitment_b.range().start)
    });

    // Sum commitments and write commitments to blob
    aggregate_commitments_to_blob(
        commitments,
        &format!("{output_path_prefix}-dynamic-dory-commitment"),
    );
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
fn cast_string_array_to_decimal256_array(string_array: &StringArray, scale: i8) -> Decimal256Array {
    let corrected_precision = 75;
    let mut builder = Decimal256Builder::default()
        .with_data_type(DataType::Decimal256(corrected_precision, scale));

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
fn convert_utf8_to_decimal_75_within_record_batch_as_appropriate(
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
                let string_array: StringArray = column
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .clone();
                big_decimal_columns_lookup
                    .get(&column_name)
                    .map(|(_precision, scale)| {
                        Arc::new(cast_string_array_to_decimal256_array(&string_array, *scale))
                            as ArrayRef
                    })
                    .unwrap_or(Arc::new(string_array))
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

/// # Panics
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
    let path_extension = "bin";
    let mut output_file = File::create(format!("{output_file_base}.{path_extension}")).unwrap();
    output_file.write_all(&bytes).unwrap();
}

#[cfg(test)]
mod tests {
    // use super::cast_string_array_to_decimal256_array;
    // use crate::{
    //     base::commitment::{Commitment, TableCommitment},
    //     proof_primitive::dory::{
    //         DoryCommitment, DoryProverPublicSetup, ProverSetup, PublicParameters,
    //     },
    //     utils::parquet_to_commitment_blob::{
    //         convert_historical_parquet_file_to_commitment_blob, PARQUET_FILE_PROOF_ORDER_COLUMN,
    //     },
    // };
    // use arrow::{
    //     array::{ArrayRef, Decimal256Builder, Int32Array, RecordBatch, StringArray},
    //     datatypes::{i256, DataType},
    // };
    // use parquet::{arrow::ArrowWriter, basic::Compression, file::properties::WriterProperties};
    // use postcard::from_bytes;
    // use rand::SeedableRng;
    // use rand_chacha::ChaCha20Rng;
    // use serde::{Deserialize, Serialize};
    // use std::{
    //     fs::{self, File},
    //     io::Read,
    //     path::Path,
    //     sync::Arc,
    // };

    // fn create_mock_file_from_record_batch(path: &str, record_batch: &RecordBatch) {
    //     let parquet_file = File::create(path).unwrap();
    //     let writer_properties = WriterProperties::builder()
    //         .set_compression(Compression::SNAPPY)
    //         .build();
    //     let mut writer =
    //         ArrowWriter::try_new(parquet_file, record_batch.schema(), Some(writer_properties))
    //             .unwrap();
    //     writer.write(record_batch).unwrap();
    //     writer.close().unwrap();
    // }

    // fn deserialize_commitment_from_file<C: Commitment + Serialize + for<'a> Deserialize<'a>>(
    //     path: &str,
    // ) -> TableCommitment<C> {
    //     let mut blob_file = File::open(path).unwrap();
    //     let mut bytes: Vec<u8> = Vec::new();
    //     blob_file.read_to_end(&mut bytes).unwrap();
    //     from_bytes(&bytes).unwrap()
    // }

    // fn delete_file_if_exists(path: &str) {
    //     if Path::new(path).exists() {
    //         fs::remove_file(path).unwrap();
    //     }
    // }

    // #[test]
    // fn we_can_convert_historical_parquet_file_to_commitment_blob() {
    //     // Purge any old files
    //     let parquet_path_1 = "example-1.parquet";
    //     let parquet_path_2 = "example-2.parquet";
    //     let dory_commitment_path = "example-dory-commitment.txt";
    //     delete_file_if_exists(parquet_path_1);
    //     delete_file_if_exists(parquet_path_2);
    //     delete_file_if_exists(dory_commitment_path);

    //     // ARRANGE

    //     // Prepare prover setup
    //     let setup_seed = "SpaceAndTime".to_string();
    //     let mut rng = {
    //         let seed_bytes = setup_seed
    //             .bytes()
    //             .chain(std::iter::repeat(0u8))
    //             .take(32)
    //             .collect::<Vec<_>>()
    //             .try_into()
    //             .expect("collection is guaranteed to contain 32 elements");
    //         ChaCha20Rng::from_seed(seed_bytes)
    //     };
    //     let public_parameters = PublicParameters::rand(4, &mut rng);
    //     let prover_setup = ProverSetup::from(&public_parameters);
    //     let dory_prover_setup: DoryProverPublicSetup = DoryProverPublicSetup::new(&prover_setup, 3);

    //     // Create two RecordBatches with the same schema
    //     let proof_column_1 = Int32Array::from(vec![1, 2]);
    //     let column_1 = Int32Array::from(vec![2, 1]);
    //     let proof_column_2 = Int32Array::from(vec![3, 4]);
    //     let column_2 = Int32Array::from(vec![3, 4]);
    //     let record_batch_1 = RecordBatch::try_from_iter(vec![
    //         (
    //             PARQUET_FILE_PROOF_ORDER_COLUMN,
    //             Arc::new(proof_column_1) as ArrayRef,
    //         ),
    //         ("column", Arc::new(column_1) as ArrayRef),
    //     ])
    //     .unwrap();
    //     let record_batch_2 = RecordBatch::try_from_iter(vec![
    //         (
    //             PARQUET_FILE_PROOF_ORDER_COLUMN,
    //             Arc::new(proof_column_2) as ArrayRef,
    //         ),
    //         ("column", Arc::new(column_2) as ArrayRef),
    //     ])
    //     .unwrap();

    //     // Write RecordBatches to parquet files
    //     create_mock_file_from_record_batch(parquet_path_1, &record_batch_1);
    //     create_mock_file_from_record_batch(parquet_path_2, &record_batch_2);

    //     // ACT
    //     convert_historical_parquet_file_to_commitment_blob(
    //         &vec![parquet_path_1.into(), parquet_path_2.into()],
    //         "example",
    //         &dory_prover_setup,
    //         &Vec::new(),
    //     );

    //     // ASSERT

    //     // Identify expected commitments
    //     let expected_column = Int32Array::from(vec![2, 1, 3, 4]);
    //     let expected_record_batch =
    //         RecordBatch::try_from_iter(vec![("column", Arc::new(expected_column) as ArrayRef)])
    //             .unwrap();
    //     let expected_commitment = TableCommitment::<DoryCommitment>::try_from_record_batch(
    //         &expected_record_batch,
    //         &dory_prover_setup,
    //     )
    //     .unwrap();

    //     assert_eq!(
    //         deserialize_commitment_from_file::<DoryCommitment>(dory_commitment_path),
    //         expected_commitment
    //     );

    //     // Tear down
    //     delete_file_if_exists(parquet_path_1);
    //     delete_file_if_exists(parquet_path_2);
    //     delete_file_if_exists(dory_commitment_path);
    // }

    // #[test]
    // fn we_can_cast_string_array_to_decimal_75() {
    //     // ARRANGE
    //     let string_array: StringArray =
    //         StringArray::from(vec![Some("123.45"), None, Some("234.56"), Some("789.01")]);

    //     // ACT
    //     let decimal_75_array = cast_string_array_to_decimal256_array(&string_array, 2);

    //     // ASSERT
    //     let mut expected_decimal_75_array =
    //         Decimal256Builder::default().with_data_type(DataType::Decimal256(75, 2));
    //     expected_decimal_75_array.append_value(i256::from(12_345));
    //     expected_decimal_75_array.append_null();
    //     expected_decimal_75_array.append_value(i256::from(23_456));
    //     expected_decimal_75_array.append_value(i256::from(78_901));
    //     assert_eq!(decimal_75_array, expected_decimal_75_array.finish());
    // }
}
