//! Binary for computing commitments to many parquet files for many tables.
//!
//! Accepts two positional arguments:
//! 1. the source, a path to the `v0/ETHEREUM/` directory
//! 2. the `output_prefix`, used when writing commitments to files

use glob::glob;
use proof_of_sql::{
    proof_primitive::dory::{DoryProverPublicSetup, ProverSetup, PublicParameters},
    utils::{
        parquet_to_commitment_blob::convert_historical_parquet_file_to_commitment_blob,
        parse::find_bigdecimals,
    },
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::{
    env,
    fs::{read_dir, File},
    io::Read,
    panic,
    path::{Path, PathBuf},
};

/// # Panics
fn main() {
    let mut args = env::args().skip(1);

    let source: PathBuf = args.next().unwrap().parse().unwrap();
    let output_prefix = args.next().unwrap();

    let mut sql = String::new();
    File::open("/testnet-parquets/Etherium_ddl_snapshot.sql")
        .unwrap()
        .read_to_string(&mut sql)
        .unwrap();
    let big_decimal_commitments = find_bigdecimals(&sql);

    let table_identifiers: Vec<(String, String)> = read_dir(source.clone())
        .unwrap()
        .map(|entry| {
            let dir_name = entry.unwrap().file_name();

            let table_name = dir_name.to_str().unwrap().to_string();

            let table_name = table_name.strip_prefix("SQL_ETHEREUM_").unwrap();

            ("ETHEREUM".to_string(), table_name.to_string())
        })
        .collect();

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
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 12);

    println!("Beginning parquet to commitments..");
    table_identifiers
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
            let result = panic::catch_unwind(|| {
                convert_historical_parquet_file_to_commitment_blob(
                    &parquets_for_table,
                    &full_output_prefix,
                    &dory_prover_setup,
                    big_decimal_commitments
                        .iter()
                        .find(|(k, _)| {
                            k.to_lowercase() == format!("{namespace}.{table_name}").to_lowercase()
                        })
                        .map(|(_, v)| v)
                        .unwrap(),
                );
            });
            if result.is_err() {
                println!("Table failed: {table_name}");
            }
        });
}
