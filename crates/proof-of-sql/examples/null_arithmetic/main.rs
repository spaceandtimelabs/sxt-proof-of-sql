//! This is a non-interactive example of using Proof of SQL with a null arithmetic dataset.
//! To run this, use `cargo run --release --example null_arithmetic`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example null_arithmetic --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.
use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{OwnedTable, OwnedTableTestAccessor, TableRef, TestAccessor},
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{
        parse::QueryExpr, postprocessing::apply_postprocessing_steps, proof::VerifiableQueryResult,
    },
};
use rand::{rngs::StdRng, SeedableRng};
use std::{fs::File, time::Instant};

// We generate the public parameters and the setups used by the prover and verifier for the Dory PCS.
// The `max_nu` should be set such that the maximum table size is less than `2^(2*max_nu-1)`.
const DORY_SETUP_MAX_NU: usize = 8;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"93c0d245eb104663bfdcd25e36bc3f97";

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
    let query_plan =
        QueryExpr::try_new(sql.parse().unwrap(), "null_arithmetic".into(), accessor).unwrap();
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Generate the proof and result:
    print!("Generating proof...");
    let now = Instant::now();
    let verifiable_result = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        query_plan.proof_expr(),
        accessor,
        &prover_setup,
    );
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Verify the result with the proof:
    print!("Verifying proof...");
    let now = Instant::now();
    let result = verifiable_result
        .verify(query_plan.proof_expr(), accessor, &verifier_setup)
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

    let filename = "./crates/proof-of-sql/examples/null_arithmetic/null_arithmetic.csv";
    let null_arithmetic_batch = ReaderBuilder::new(SchemaRef::new(
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
        TableRef::new("null_arithmetic", "tab"),
        OwnedTable::try_from(null_arithmetic_batch).unwrap(),
        0,
    );

    // Query 1: Show all data
    prove_and_verify_query(
        "SELECT * FROM tab",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 2: Test A + B = 2
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A + B = 2",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 3: Test A + B = 4
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A + B = 4",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 4: Test A + B = 10 (should return empty result)
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A + B = 10",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
} 