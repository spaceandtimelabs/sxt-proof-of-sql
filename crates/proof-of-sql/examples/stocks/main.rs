//! This is a non-interactive example of using Proof of SQL with a stocks dataset.
//! To run this, use cargo run --release --example stocks.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run cargo run --release --example stocks --no-default-features --features="arrow cpu-perf" instead. It will be slower for proof generation.

use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{
        arrow_schema_utility::get_posql_compatible_schema, OwnedTable, OwnedTableTestAccessor,
        TestAccessor,
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
// The max_nu should be set such that the maximum table size is less than 2^(2*max_nu-1).
const DORY_SETUP_MAX_NU: usize = 8;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"f9d2e8c1b7a654309cfe81d2b7a3c940";

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
        "stocks".parse().unwrap(),
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

    let filename = "./crates/proof-of-sql/examples/stocks/stocks.csv";
    let schema = get_posql_compatible_schema(&SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ));
    let stocks_batch = ReaderBuilder::new(schema)
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
        "stocks.stocks".parse().unwrap(),
        OwnedTable::try_from(stocks_batch).unwrap(),
        0,
    );

    // Query 1: Calculate total market cap and count of stocks
    prove_and_verify_query(
        "SELECT SUM(MarketCap) as total_market_cap, COUNT(*) as c FROM stocks",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 2: Find technology stocks with PE ratio under 30 and dividend yield > 0
    prove_and_verify_query(
        "SELECT Symbol, Company, PE_Ratio, DividendYield 
         FROM stocks 
         WHERE Sector = 'Technology' AND PE_Ratio < 30 AND DividendYield > 0 
         ORDER BY PE_Ratio DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 3: Average market cap by sector (using SUM/COUNT instead of AVG)
    prove_and_verify_query(
        "SELECT Sector, SUM(MarketCap)/COUNT(*) as avg_market_cap, COUNT(*) as c 
         FROM stocks 
         GROUP BY Sector 
         ORDER BY avg_market_cap DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 4: High value stocks with significant volume and dividend yield
    prove_and_verify_query(
        "SELECT Symbol, Company, Price, Volume, DividendYield 
         FROM stocks 
         WHERE Volume > 20000000 AND DividendYield > 0 AND Price > 100 
         ORDER BY Volume DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
