//! This is an non-interactive example of using Proof of SQL with some space related datasets.
//! To run this, use `cargo run --release --example space`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example space --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.

// Note: the space_travellers.csv file was obtained from
// https://www.kaggle.com/datasets/kaushiksinghrawat/humans-to-have-visited-space
// under the Apache 2.0 license.

use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{OwnedTable, OwnedTableTestAccessor},
    proof_primitive::dory::{
        DynamicDoryCommitment, DynamicDoryEvaluationProof, ProverSetup, PublicParameters,
        VerifierSetup,
    },
    sql::{parse::QueryExpr, proof::QueryProof},
};
use rand::{rngs::StdRng, SeedableRng};
use std::fs::File;

// We generate the public parameters and the setups used by the prover and verifier for the Dory PCS.
// The `max_nu` should be set such that the maximum table size is less than `2^(2*max_nu-1)`.
// For a sampling:
// max_nu = 3 => max table size is 32 rows
// max_nu = 4 => max table size is 128 rows
// max_nu = 8 => max table size is 32768 rows
// max_nu = 10 => max table size is 0.5 million rows
// max_nu = 15 => max table size is 0.5 billion rows
// max_nu = 20 => max table size is 0.5 trillion rows
// Note: we will eventually load these from a file.
const DORY_SETUP_MAX_NU: usize = 8;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"len 32 rng seed - Space and Time";

fn main() {
    let mut rng = StdRng::from_seed(DORY_SEED);
    let public_parameters = PublicParameters::rand(DORY_SETUP_MAX_NU, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    let filename = "./crates/proof-of-sql/examples/space/space_travellers.csv";
    let space_travellers_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
    .with_header(true)
    .build(File::open(filename).unwrap())
    .unwrap()
    .next()
    .unwrap()
    .unwrap();

    // Load the table into an "Accessor" so that the prover and verifier can access the data/commitments.
    let accessor = OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_from_table(
        "space.travellers".parse().unwrap(),
        OwnedTable::try_from(space_travellers_batch).unwrap(),
        0,
        &prover_setup,
    );

    // Parse the query:
    let query_plan = QueryExpr::<DynamicDoryCommitment>::try_new(
        "SELECT * FROM travellers".parse().unwrap(),
        "space".parse().unwrap(),
        &accessor,
    )
    .unwrap();

    // Generate the proof and result:
    print!("Generating proof...");
    let (proof, provable_result) = QueryProof::<DynamicDoryEvaluationProof>::new(
        query_plan.proof_expr(),
        &accessor,
        &&prover_setup,
    );
    println!("Done.");

    // Verify the result with the proof:
    print!("Verifying proof...");
    let result = proof
        .verify(
            query_plan.proof_expr(),
            &accessor,
            &provable_result,
            &&verifier_setup,
        )
        .unwrap();
    println!("Verified.");

    // Display the result
    println!("Query Result:");
    println!("{:?}", result.table);
}
