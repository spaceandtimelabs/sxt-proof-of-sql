use proof_of_sql::{
    proof_primitive::dory::{ProverSetup, PublicParameters},
    utils::parquet_to_commitment_blob::read_parquet_file_to_commitment_as_blob,
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::{env, io};

fn main() {
    let parquet_paths = io::stdin()
        .lines()
        .map(|line| line.unwrap().parse().unwrap())
        .collect();

    let output_prefix = env::args().skip(1).next().unwrap();

    println!("Generating setup..");

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
    let public_parameters = PublicParameters::rand(12, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);

    read_parquet_file_to_commitment_as_blob(parquet_paths, &output_prefix, prover_setup)
}
