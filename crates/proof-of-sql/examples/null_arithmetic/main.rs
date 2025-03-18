//! This is a non-interactive example of using Proof of SQL with a null arithmetic dataset.
//! To run this, use `cargo run --release --example null_arithmetic`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example null_arithmetic --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.
#[cfg(feature = "arrow")]
use arrow::datatypes::SchemaRef;
#[cfg(feature = "arrow")]
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{OwnedTable, OwnedTableTestAccessor, TableRef, TestAccessor},
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{parse::QueryExpr, proof::VerifiableQueryResult},
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
    let result = result.table;
    println!("Verified in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Display the result
    println!("Query Result:");
    println!("{result:?}");
}

#[allow(clippy::too_many_lines)]
#[cfg(feature = "arrow")]
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
        "SELECT * FROM tab WHERE A + B = 0",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 5: Test A - B = 0 (should only return rows where A and B are non-NULL and equal)
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A - B = 0",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 6: Test with a large negative number
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A + B = -999999999998",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 7: Test OR condition with A = 1 OR B = 1
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A = 1 OR B = 1",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 8: Test IS NULL on column A
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A IS NULL",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 9: Test IS NOT NULL on column A
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A IS NOT NULL",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 10: Test IS NULL on column B
    prove_and_verify_query(
        "SELECT * FROM tab WHERE B IS NULL",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 11: Test IS NOT NULL on column B
    prove_and_verify_query(
        "SELECT * FROM tab WHERE B IS NOT NULL",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 12: Test combining IS NULL with other conditions
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A IS NULL AND B = 1",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 13: Test combining IS NOT NULL with other conditions
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A IS NOT NULL AND B IS NULL",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 14: Test greater than (>)
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A > 1",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 15: Test less than (<)
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A < 1",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 16: Test greater than or equal to (>=)
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A >= 1",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 17: Test less than or equal to (<=)
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A <= 1",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 18: Combine comparison with NULL check
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A > 0 AND B IS NOT NULL",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 19: Test B greater than A
    prove_and_verify_query(
        "SELECT * FROM tab WHERE B > A",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 20: Test with expression in comparison
    prove_and_verify_query(
        "SELECT * FROM tab WHERE A + B > 2",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}

#[cfg(not(feature = "arrow"))]
fn main() {
    println!("This example requires the 'arrow' feature to be enabled.");
    println!("Please run with: cargo run --release --example null_arithmetic --features=arrow");
}
