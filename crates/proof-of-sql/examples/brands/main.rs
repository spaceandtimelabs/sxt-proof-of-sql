//! This is a non-interactive example of using Proof of SQL with a brands dataset.
//! To run this, use `cargo run --release --example brands`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example brands --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.
use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{
        arrow_schema_utility::get_posql_compatible_schema, OwnedTable, OwnedTableTestAccessor,
        TableRef, TestAccessor,
    },
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
const DORY_SEED: [u8; 32] = *b"8f3a2e1c5b9d7f0a6e4d2c8b7a9f1e3d";

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
    let query_plan = QueryExpr::try_new(sql.parse().unwrap(), "brands".into(), accessor).unwrap();
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Generate the proof and result:
    print!("Generating proof...");
    let now = Instant::now();
    let verifiable_result = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        query_plan.proof_expr(),
        accessor,
        &prover_setup,
        &[],
    );
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Verify the result with the proof:
    print!("Verifying proof...");
    let now = Instant::now();
    let result = verifiable_result
        .verify(query_plan.proof_expr(), accessor, &verifier_setup, &[])
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

    let filename = "./crates/proof-of-sql/examples/brands/brands.csv";
    let inferred_schema =
        SchemaRef::new(infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap());
    let posql_compatible_schema = get_posql_compatible_schema(&inferred_schema);

    let brands_batch = ReaderBuilder::new(posql_compatible_schema)
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
        TableRef::new("brands", "global_brands"),
        OwnedTable::try_from(brands_batch).unwrap(),
        0,
    );

    // Query 1: Count the total number of brands
    prove_and_verify_query(
        "SELECT COUNT(*) AS total_brands FROM global_brands",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 2: List the names of brands founded before 1950
    prove_and_verify_query(
        "SELECT Name FROM global_brands WHERE Founded < 1950",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 3: List the top 5 countries with the highest total revenue, ordered by total revenue
    prove_and_verify_query(
        "SELECT Country, SUM(Revenue) AS total_revenue FROM global_brands GROUP BY Country ORDER BY total_revenue DESC LIMIT 5",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
