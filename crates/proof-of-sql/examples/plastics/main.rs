//! This is a non-interactive example of using Proof of SQL with a plastics dataset.
//! To run this, use `cargo run --release --example plastics`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example plastics --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.

use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::{
        arrow::arrow_schema_utility::get_posql_compatible_schema,
        database::{OwnedTable, OwnedTableTestAccessor, TestAccessor},
    },
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
const DORY_SEED: [u8; 32] = *b"32f7f321c4ab1234d5e6f7a8b9c0d1e2";

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
        "plastics".parse().unwrap(),
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

    let filename = "./crates/proof-of-sql/examples/plastics/plastics.csv";
    let schema = get_posql_compatible_schema(&SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ));
    let plastics_batch = ReaderBuilder::new(schema)
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
        "plastics.types".parse().unwrap(),
        OwnedTable::try_from(plastics_batch).unwrap(),
        0,
    );

    // Query 1: Count total number of plastic types
    prove_and_verify_query(
        "SELECT COUNT(*) AS total_types FROM types",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 2: List names of biodegradable plastics
    prove_and_verify_query(
        "SELECT Name FROM types WHERE Biodegradable = TRUE ORDER BY Name",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 3: Show average density of plastics by recycling code
    prove_and_verify_query(
        "SELECT Code, SUM(Density)/COUNT(*) as avg_density FROM types GROUP BY Code ORDER BY Code",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 4: List plastics with density greater than 1.0 g/cm³
    prove_and_verify_query(
        "SELECT Name, Density FROM types WHERE Density > 1.0 ORDER BY Density DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
