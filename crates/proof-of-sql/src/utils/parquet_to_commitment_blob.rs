use crate::{
    base::commitment::{Commitment, TableCommitment},
    proof_primitive::dory::{DoryCommitment, DoryProverPublicSetup, DynamicDoryCommitment, ProverSetup, PublicParameters},
};
use arrow::{array::RecordBatch, compute::concat_batches, error::ArrowError};
use curve25519_dalek::RistrettoPoint;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use postcard::to_allocvec;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::Path};

/// Performs the following:
/// Reads a parquet file into a `RecordBatch`,
/// Calculates the `TableCommitment` for the `RecordBatch` using multiple commitment strategies,
/// Serializes the commitment to a blob, which is saved in the same directory as the original parquet file
///
/// # Panics
///
/// Panics when fails any part of the process
pub fn read_parquet_file_to_commitment_as_blob(path: &str) {
    let path_object = Path::new(path);
    read_parquet_file_to_commitment_as_blob_and_write_to_file::<RistrettoPoint>(
        path_object,
        (),
        "ristretto_point".to_string(),
    );
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
        path_object,
        dory_prover_setup,
        "dory_commitment".to_string(),
    );
    read_parquet_file_to_commitment_as_blob_and_write_to_file::<DynamicDoryCommitment>(
        path_object,
        &prover_setup,
        "dynamic_dory_commitment".to_string(),
    );
}

/// # Panics
///
/// Panics when fails any part of the process
fn read_parquet_file_to_commitment_as_blob_and_write_to_file<
    C: Commitment + Serialize + for<'a> Deserialize<'a>,
>(
    path: &Path,
    setup: C::PublicSetup<'_>,
    output_file_suffix: String,
) {
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
    let record_batch: RecordBatch = concat_batches(&schema, &record_batches).unwrap();
    let commitment = TableCommitment::<C>::try_from_record_batch(&record_batch, &setup).unwrap();
    let bytes: Vec<u8> = to_allocvec(&commitment).unwrap();
    let path_base = path.file_stem().unwrap().to_str().unwrap();
    let path_extension = "txt";
    let mut output_file =
        File::create(format!("{path_base}_{output_file_suffix}.{path_extension}")).unwrap();
    output_file.write_all(&bytes).unwrap();
}
