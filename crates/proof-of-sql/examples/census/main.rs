//! Example to use Proof of SQL with census datasets
//! To run, use `cargo run --release --example census`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example census --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.

// Note: the census-income.csv was obtained from
// https://github.com/domoritz/vis-examples/blob/master/data/census-income.csv
use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{OwnedTable, OwnedTableTestAccessor},
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{parse::QueryExpr, postprocessing::apply_postprocessing_steps, proof::QueryProof},
};
use rand::{rngs::StdRng, SeedableRng};
use std::{fs::File, time::Instant};

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
        QueryExpr::try_new(sql.parse().unwrap(), "census".parse().unwrap(), accessor).unwrap();
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

    let filename = "./crates/proof-of-sql/examples/census/census-income.csv";
    let census_income_batch = ReaderBuilder::new(SchemaRef::new(
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
        "census.income".parse().unwrap(),
        OwnedTable::try_from(census_income_batch).unwrap(),
        0,
        &prover_setup,
    );

    prove_and_verify_query(
        "SELECT COUNT(*) AS total_geographies FROM income",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Geography, COUNT(*) AS num_geographies FROM income GROUP BY Geography ORDER BY num_geographies",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Geography, COUNT(*) AS num_geographies FROM income WHERE Households_Estimate_total > 2000000 GROUP BY Geography ORDER BY num_geographies DESC LIMIT 5",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
