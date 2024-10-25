//! This is a non-interactive example of using Proof of SQL with a sports dataset.
//! To run this, use `cargo run --release --example sports`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example sports --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.

use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{OwnedTable, OwnedTableTestAccessor, TestAccessor},
    proof_primitive::dory::{
        DynamicDoryCommitment, DynamicDoryEvaluationProof, ProverSetup, PublicParameters,
        VerifierSetup,
    },
    sql::{parse::QueryExpr, postprocessing::apply_postprocessing_steps, proof::QueryProof},
};
use rand::{rngs::StdRng, SeedableRng};
use std::{fs::File, time::Instant};

// We generate the public parameters and the setups used by the prover and verifier for the Dory PCS.
// The `max_nu` should be set such that the maximum table size is less than `2^(2*max_nu-1)`.
const DORY_SETUP_MAX_NU: usize = 8;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"8b5d523c69a1d1e4f2c3b6a9d8e7f012";

/// # Panics
/// Will panic if the query does not parse or the proof fails to verify.
fn prove_and_verify_query(
    sql: &str,
    accessor: &OwnedTableTestAccessor<DynamicDoryEvaluationProof>,
    prover_setup: &ProverSetup,
    verifier_setup: &VerifierSetup,
) {
    // Parse the query:
    println!("Parsing the query: {sql}...");
    let now = Instant::now();
    let query_plan = QueryExpr::<DynamicDoryCommitment>::try_new(
        sql.parse().unwrap(),
        "sports".parse().unwrap(),
        accessor,
    )
    .unwrap();
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Generate the proof and result:
    print!("Generating proof...");
    let now = Instant::now();
    let (proof, provable_result) = QueryProof::<DynamicDoryEvaluationProof>::new(
        query_plan.proof_expr(),
        accessor,
        &prover_setup,
    );
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Verify the result with the proof:
    print!("Verifying proof...");
    let now = Instant::now();
    let result = proof
        .verify(
            query_plan.proof_expr(),
            accessor,
            &provable_result,
            &verifier_setup,
        )
        .unwrap();
    let result = apply_postprocessing_steps(result.table, query_plan.postprocessing());
    println!("Verified in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Display the result
    println!("Query Result:");
    println!("{result:?}");
}

fn main() {
    let mut rng = StdRng::from_seed(DORY_SEED);
    let public_parameters = PublicParameters::rand(DORY_SETUP_MAX_NU, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    let filename = "./crates/proof-of-sql/examples/sports/sports.csv";
    let sports_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
    .with_header(true)
    .build(File::open(filename).unwrap())
    .unwrap()
    .next()
    .unwrap()
    .unwrap();

    // Load the table into an "Accessor" so that the prover and verifier can access the data/commitments.
    let mut accessor =
        OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    accessor.add_table(
        "sports.sports".parse().unwrap(),
        OwnedTable::try_from(sports_batch).unwrap(),
        0,
    );

    // Count total number of sports
    prove_and_verify_query(
        "SELECT COUNT(*) AS total_sports FROM sports",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Group sports by type with counts 
    prove_and_verify_query(
        "SELECT Type, COUNT(*) as count FROM sports GROUP BY Type ORDER BY count DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Filter team sports and order by name
    prove_and_verify_query(
        "SELECT \"Sport Name\" FROM sports WHERE \"Team or Individual\" = 'Team' ORDER BY \"Sport Name\"",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Filter by multiple conditions
    prove_and_verify_query(
        "SELECT \"Sport Name\" FROM sports WHERE Type = 'Indoor' AND \"Team or Individual\" = 'Individual'",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Group by with multiple dimensions
    prove_and_verify_query(
        "SELECT Type, \"Team or Individual\", COUNT(*) as count FROM sports GROUP BY Type, \"Team or Individual\" ORDER BY count DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}