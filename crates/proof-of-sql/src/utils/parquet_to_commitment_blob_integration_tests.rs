use super::parquet_to_commitment_blob::read_parquet_file_to_commitment_as_blob;
use crate::{
    base::commitment::{Commitment, TableCommitment},
    proof_primitive::dory::{DoryCommitment, DoryProverPublicSetup, DynamicDoryCommitment, ProverSetup, PublicParameters},
};
use arrow::array::{ArrayRef, Int32Array, RecordBatch};
use curve25519_dalek::RistrettoPoint;
use parquet::{arrow::ArrowWriter, basic::Compression, file::properties::WriterProperties};
use postcard::from_bytes;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
    sync::Arc,
};

fn create_mock_file_from_record_batch(path: &str, record_batch: &RecordBatch) {
    let parquet_file = File::create(path).unwrap();
    let writer_properties = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();
    let mut writer =
        ArrowWriter::try_new(parquet_file, record_batch.schema(), Some(writer_properties)).unwrap();
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

fn calculate_dory_commitment(record_batch: &RecordBatch) -> TableCommitment<DoryCommitment> {
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
    TableCommitment::<DoryCommitment>::try_from_record_batch(&record_batch, &dory_prover_setup)
        .unwrap()
}

fn calculate_ristretto_point(record_batch: &RecordBatch) -> TableCommitment<RistrettoPoint> {
    TableCommitment::<RistrettoPoint>::try_from_record_batch(&record_batch, &())
        .unwrap()
}

fn calculate_dynamic_dory_commitment(record_batch: &RecordBatch) -> TableCommitment<DynamicDoryCommitment> {
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
    TableCommitment::<DynamicDoryCommitment>::try_from_record_batch(&record_batch, &&prover_setup)
        .unwrap()
}

fn delete_file_if_exists(path: &str) {
    if Path::new(path).exists() {
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn we_can_retrieve_commitments_and_save_to_file() {
    let parquet_path = "example.parquet";
    let ristretto_point_path = "example_ristretto_point.txt";
    let dory_commitment_path = "example_dory_commitment.txt";
    let dynamic_dory_commitment_path = "example_dynamic_dory_commitment.txt";
    delete_file_if_exists(parquet_path);
    delete_file_if_exists(ristretto_point_path);
    delete_file_if_exists(dory_commitment_path);
    delete_file_if_exists(dynamic_dory_commitment_path);
    let column = Int32Array::from(vec![1, 2, 3, 4]);
    let record_batch =
        RecordBatch::try_from_iter(vec![("id", Arc::new(column) as ArrayRef)]).unwrap();
    create_mock_file_from_record_batch(parquet_path, &record_batch);
    read_parquet_file_to_commitment_as_blob(parquet_path);
    assert_eq!(
        read_commitment_from_blob::<DoryCommitment>(dory_commitment_path),
        calculate_dory_commitment(&record_batch)
    );
    assert_eq!(
        read_commitment_from_blob::<RistrettoPoint>(ristretto_point_path),
        calculate_ristretto_point(&record_batch)
    );
    assert_eq!(
        read_commitment_from_blob::<DynamicDoryCommitment>(dynamic_dory_commitment_path),
        calculate_dynamic_dory_commitment(&record_batch)
    );
    delete_file_if_exists(parquet_path);
    delete_file_if_exists(ristretto_point_path);
    delete_file_if_exists(dory_commitment_path);
    delete_file_if_exists(dynamic_dory_commitment_path);
}
