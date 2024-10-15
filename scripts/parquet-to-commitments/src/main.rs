//! Accepts a list of parquet files from stdin, a output-file prefix as an env arg, then produces
//! commitment files starting with that prefix.

use proof_of_sql::{
    proof_primitive::dory::{ProverSetup, PublicParameters},
    utils::parquet_to_commitment_blob::read_parquet_file_to_commitment_as_blob,
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::{env, io, path::Path};

fn main() {
    let parquet_paths = io::stdin()
        .lines()
        .map(|line| line.unwrap().parse().unwrap())
        .collect();

    let output_prefix = env::args().skip(1).next().unwrap();

    let public_parameters_path = Path::new("public-parameters");

    let public_parameters = if public_parameters_path.exists() {
        println!("Loading public parameters..");
        PublicParameters::load_from_file(public_parameters_path).unwrap()
    } else {
        println!("Generating public parameters..");
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

        println!("Saving public parameters..");
        public_parameters
            .save_to_file(public_parameters_path)
            .unwrap();

        public_parameters
    };

    println!("Creating prover setup..");
    let prover_setup = ProverSetup::from(&public_parameters);

    println!("Beginning parquet to commitments..");
    read_parquet_file_to_commitment_as_blob(parquet_paths, &output_prefix, prover_setup)
}
