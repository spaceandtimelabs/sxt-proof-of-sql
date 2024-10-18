//! Binary for computing commitments to many parquet files for many tables.
//!
//! Accepts two positional arguments:
//! 1. the source, a path to the `v0/ETHEREUM/` directory
//! 2. the output_prefix, used when writing commitments to files

use std::env;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use glob::glob;
use proof_of_sql::proof_primitive::dory::{ProverSetup, PublicParameters};
use proof_of_sql::utils::parquet_to_commitment_blob::read_parquet_file_to_commitment_as_blob;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

const TABLE_IDENTIFIERS: [(&str, &str); 8] = [
    ("ETHEREUM", "ERC1155_EVT_TRANSFERBATCH"),
    ("ETHEREUM", "CONTRACTS"),
    ("ETHEREUM", "LOGS"),
    ("ETHEREUM", "NFT_COLLECTIONS"),
    ("ETHEREUM", "ERC1155_EVT_TRANSFERBATCH"),
    ("ETHEREUM", "CONTRACT_EVT_APPROVALFORALL"),
    ("ETHEREUM", "CONTRACT_EVT_OWNERSHIPTRANSFERRED"),
    ("ETHEREUM", "STORAGE_SLOTS"),
];

fn main() {
    let mut args = env::args().skip(1);

    let source: PathBuf = args.next().unwrap().parse().unwrap();
    let output_prefix = args.next().unwrap();

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
        let public_parameters = PublicParameters::rand(16, &mut rng);

        println!("Saving public parameters..");
        public_parameters
            .save_to_file(public_parameters_path)
            .unwrap();

        public_parameters
    };

    println!("Creating prover setup..");
    let prover_setup = ProverSetup::from(&public_parameters);

    println!("Beginning parquet to commitments..");
    TABLE_IDENTIFIERS
        .iter()
        .for_each(|(namespace, table_name)| {
            let parquets_for_table = glob(&format!(
                "{}/SQL_{namespace}_{table_name}/**/**/*.parquet",
                source.as_path().to_str().unwrap()
            ))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

            let full_output_prefix = format!("{output_prefix}-{namespace}-{table_name}");

            read_parquet_file_to_commitment_as_blob(
                parquets_for_table,
                &full_output_prefix,
                &prover_setup,
            )
            .ok();
        });
}
