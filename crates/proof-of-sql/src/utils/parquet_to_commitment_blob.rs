use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{
    Array,
    ArrayRef,
    ArrowPrimitiveType,
    BooleanArray,
    Decimal128Array,
    Decimal256Array,
    Int16Array,
    Int32Array,
    Int64Array,
    Int8Array,
    PrimitiveArray,
    RecordBatch,
    StringArray,
    TimestampMicrosecondArray,
    TimestampMillisecondArray,
    TimestampSecondArray,
};
use arrow::compute::concat_batches;
use arrow::datatypes::{DataType, TimeUnit};
use arrow::error::ArrowError;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use postcard::to_allocvec;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::base::commitment::{Commitment, TableCommitment};
use crate::proof_primitive::dory::{
    DoryCommitment,
    DoryProverPublicSetup,
    DynamicDoryCommitment,
    ProverSetup,
};

static PARQUET_FILE_PROOF_ORDER_COLUMN: &str = "META_ROW_NUMBER";

pub struct HasNulls;

/// Performs the following:
/// Reads a collection of parquet files which in aggregate represent a single table of data,
/// Calculates the `TableCommitment` for the table using multiple commitment strategies,
/// Serializes each commitment to a blob, which is saved in the same directory as the original parquet file
///
/// # Panics
///
/// Panics when any part of the process fails
pub fn read_parquet_file_to_commitment_as_blob(
    parquet_files: Vec<PathBuf>,
    output_path_prefix: &str,
    prover_setup: &ProverSetup,
) -> Result<(), HasNulls> {
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 12);
    let mut commitments: Vec<TableCommitment<DoryCommitment>> = parquet_files
        .par_iter()
        .map(|path| {
            println!("Committing to {}..", path.as_path().to_str().unwrap());
            let file = File::open(path).unwrap();
            let reader = ParquetRecordBatchReaderBuilder::try_new(file)
                .unwrap()
                .build()
                .unwrap();
            let record_batch_results: Vec<Result<RecordBatch, ArrowError>> = reader.collect();
            let record_batches: Vec<RecordBatch> = record_batch_results
                .into_iter()
                .map(|record_batch_result| record_batch_result.unwrap())
                .collect();
            let schema = record_batches.first().unwrap().schema();
            let record_batch = concat_batches(&schema, &record_batches).unwrap();

            if batch_has_nulls(&record_batch) {
                println!("found nullable column in table, skipping...");
                Err(HasNulls)?;
            }

            let meta_row_number_column = record_batch
                .column_by_name(PARQUET_FILE_PROOF_ORDER_COLUMN)
                .unwrap()
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap();

            let offset = meta_row_number_column.value(0) - 1;

            let record_batch = replace_nulls_within_record_batch(record_batch);
            let dory_commitment =
                TableCommitment::<DoryCommitment>::try_from_record_batch_with_offset(
                    &record_batch,
                    offset as usize,
                    &dory_prover_setup,
                )
                .unwrap();
            Ok(dory_commitment)
        })
        .collect::<Result<_, HasNulls>>()?;

    println!("done computing per-file commitments, now sorting and aggregating");
    commitments.sort_by(|commitment_a, commitment_b| {
        commitment_a.range().start.cmp(&commitment_b.range().start)
    });

    //aggregate_commitments_to_blob(unzipped.0, format!("{output_path_prefix}-dory-commitment"));
    aggregate_commitments_to_blob(commitments, format!("{output_path_prefix}-dory-commitment"));

    Ok(())
}

fn batch_has_nulls(record_batch: &RecordBatch) -> bool {
    record_batch
        .columns()
        .iter()
        .any(|col| col.null_count() > 0)
}

/// # Panics
///
/// Panics when any part of the process fails
fn aggregate_commitments_to_blob<C: Commitment + Serialize + for<'a> Deserialize<'a>>(
    commitments: Vec<TableCommitment<C>>,
    output_file_base: String,
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

fn write_commitment_to_blob<C: Commitment + Serialize + for<'a> Deserialize<'a>>(
    commitment: &TableCommitment<C>,
    output_file_base: String,
) {
    let bytes: Vec<u8> = to_allocvec(commitment).unwrap();
    let path_extension = "txt";
    let mut output_file = File::create(format!("{output_file_base}.{path_extension}")).unwrap();
    output_file.write_all(&bytes).unwrap();
}

fn replace_nulls_primitive<T: ArrowPrimitiveType>(array: &PrimitiveArray<T>) -> PrimitiveArray<T> {
    PrimitiveArray::from_iter_values(array.iter().map(
        |value: Option<<T as ArrowPrimitiveType>::Native>| value.unwrap_or(T::Native::default()),
    ))
}

fn replace_nulls_within_record_batch(record_batch: RecordBatch) -> RecordBatch {
    let schema = record_batch.schema();
    let new_columns: Vec<_> = record_batch
        .columns()
        .into_iter()
        .map(|column| {
            println!("found nullable column, converting...");
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
                                .downcast_ref::<TimestampMicrosecondArray>()
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
