use crate::{
    base::commitment::{Commitment, TableCommitment},
    proof_primitive::dory::{
        DoryCommitment, DoryProverPublicSetup, DynamicDoryCommitment, ProverSetup, PublicParameters,
    },
};
use arrow::{
    array::RecordBatch,
    compute::{concat_batches, sort_to_indices, take},
    error::ArrowError,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use postcard::to_allocvec;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};

/// Performs the following:
/// Reads a parquet file into a `RecordBatch`,
/// Calculates the `TableCommitment` for the `RecordBatch` using multiple commitment strategies,
/// Serializes the commitment to a blob, which is saved in the same directory as the original parquet file
///
/// # Panics
///
/// Panics when fails any part of the process
pub fn read_parquet_file_to_commitment_as_blob(paths: Vec<&str>, output_path_prefix: &str) {
    let unsorted_record_batches_with_unmodified_schema: Vec<RecordBatch> = paths
        .iter()
        .map(|path| {
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
            concat_batches(&schema, &record_batches).unwrap()
        })
        .collect();
    let schema = unsorted_record_batches_with_unmodified_schema
        .first()
        .unwrap()
        .schema();
    let unsorted_record_batch_with_unmodified_schema =
        concat_batches(&schema, &unsorted_record_batches_with_unmodified_schema).unwrap();
    let indices = sort_to_indices(
        unsorted_record_batch_with_unmodified_schema
            .column_by_name("SXTMETA_ROW_NUMBER")
            .unwrap(),
        None,
        None,
    )
    .unwrap();
    let index = schema.index_of("SXTMETA_ROW_NUMBER").unwrap();
    let columns = unsorted_record_batch_with_unmodified_schema
        .columns()
        .iter()
        .map(|c| take(&*c, &indices, None).unwrap())
        .collect();
    let mut record_batch = RecordBatch::try_new(
        unsorted_record_batch_with_unmodified_schema.schema(),
        columns,
    )
    .unwrap();
    record_batch.remove_column(index);

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
    let public_parameters = PublicParameters::rand(4, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    read_parquet_file_to_commitment_as_blob_and_write_to_file::<DoryCommitment>(
        &record_batch,
        dory_prover_setup,
        format!("{output_path_prefix}_dory_commitment"),
    );
    read_parquet_file_to_commitment_as_blob_and_write_to_file::<DynamicDoryCommitment>(
        &record_batch,
        &prover_setup,
        format!("{output_path_prefix}_dynamic_dory_commitment"),
    );
}

/// # Panics
///
/// Panics when fails any part of the process
fn read_parquet_file_to_commitment_as_blob_and_write_to_file<
    C: Commitment + Serialize + for<'a> Deserialize<'a>,
>(
    record_batch: &RecordBatch,
    setup: C::PublicSetup<'_>,
    output_file_base: String,
) {
    let commitment = TableCommitment::<C>::try_from_record_batch(&record_batch, &setup).unwrap();
    let bytes: Vec<u8> = to_allocvec(&commitment).unwrap();
    let path_extension = "txt";
    let mut output_file = File::create(format!("{output_file_base}.{path_extension}")).unwrap();
    output_file.write_all(&bytes).unwrap();
}
