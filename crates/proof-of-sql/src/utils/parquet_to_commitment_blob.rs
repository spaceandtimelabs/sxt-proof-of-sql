use crate::{
    base::commitment::{Commitment, TableCommitment},
    proof_primitive::dory::{
        DoryCommitment, DoryProverPublicSetup, DynamicDoryCommitment, ProverSetup, PublicParameters,
    },
};
use arrow::{
    array::{Array, ArrayRef, ArrowPrimitiveType, AsArray, Int32Array, NativeAdapter, PrimitiveArray, RecordBatch},
    compute::concat_batches,
    error::ArrowError,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use postcard::to_allocvec;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, sync::Arc};

pub static PARQUET_FILE_PROOF_ORDER_COLUMN: &str = "META_ROW_NUMBER";

/// Performs the following:
/// Reads a collection of parquet files which in aggregate represent a single table of data,
/// Calculates the `TableCommitment` for the table using multiple commitment strategies,
/// Serializes each commitment to a blob, which is saved in the same directory as the original parquet file
///
/// # Panics
///
/// Panics when any part of the process fails
pub fn read_parquet_file_to_commitment_as_blob(path_bases: Vec<&str>, output_path_prefix: &str) {
    let mut offset: usize = 0;
    let commitments: Vec<(
        TableCommitment<DoryCommitment>,
        TableCommitment<DynamicDoryCommitment>,
    )> = path_bases
        .iter()
        .map(|path_base| {
            let file = File::open(format!("{path_base}.parquet")).unwrap();
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
            let mut record_batch = concat_batches(&schema, &record_batches).unwrap();
            let meta_row_number_column = record_batch
                .column_by_name(PARQUET_FILE_PROOF_ORDER_COLUMN)
                .unwrap()
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap();
            let length = meta_row_number_column.len();
            let new_offset = offset + length;
            let range = ((offset + 1) as i32)..((new_offset + 1) as i32);
            assert_eq!(
                meta_row_number_column,
                &Int32Array::from(range.collect::<Vec<_>>())
            );
            record_batch.remove_column(schema.index_of(PARQUET_FILE_PROOF_ORDER_COLUMN).unwrap());
            let setup_seed = "spaceandtime".to_string();
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
            let public_parameters = PublicParameters::rand(12, &mut rng);
            let prover_setup = ProverSetup::from(&public_parameters);
            let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 20);
            let dory_commitment =
                TableCommitment::<DoryCommitment>::try_from_record_batch_with_offset(
                    &record_batch,
                    offset,
                    &dory_prover_setup,
                )
                .unwrap();
            let dynamic_dory_commitment =
                TableCommitment::<DynamicDoryCommitment>::try_from_record_batch_with_offset(
                    &record_batch,
                    offset,
                    &&prover_setup,
                )
                .unwrap();
            offset = new_offset;
            (dory_commitment, dynamic_dory_commitment)
        })
        .collect();
    let unzipped = commitments.into_iter().unzip();
    aggregate_commitments_to_blob(unzipped.0, format!("{output_path_prefix}-dory-commitment"));
    aggregate_commitments_to_blob(
        unzipped.1,
        format!("{output_path_prefix}-dynamic-dory-commitment"),
    );
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

fn replace_nulls<T: ArrowPrimitiveType>(array: &PrimitiveArray<T>) -> PrimitiveArray<T>
where
    NativeAdapter<T>: From<<T as ArrowPrimitiveType>::Native>,
{
    array
        .iter()
        .map(|value: Option<<T as ArrowPrimitiveType>::Native>| {
            value.unwrap_or(T::Native::default())
        })
        .collect()
}

fn replace_nulls_within_record_batch(record_batch: RecordBatch) -> RecordBatch{
    let schema = record_batch.schema();
    let new_columns: Vec<_> = record_batch.columns().into_iter().map(|column| {
        match column.is_nullable() {
            true => Arc::new(replace_nulls(column.as_primitive())) as ArrayRef,
            false => Arc::new(column.as_primitive()) as ArrayRef
        }
    }).collect();
    RecordBatch::try_new(schema, new_columns).unwrap()
}