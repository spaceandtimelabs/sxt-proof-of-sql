//! This is a non-interactive example of using Proof of SQL with a `tech_gadget_prices` dataset.
//! To run this, use cargo run --release --example `tech_gadget_prices`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run cargo run --release --example `tech_gadget_prices` --no-default-features --features="arrow cpu-perf" instead. It will be slower for proof generation.

use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{OwnedTable, OwnedTableTestAccessor},
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{parse::QueryExpr, proof::VerifiableQueryResult},
};
use rand::{rngs::StdRng, SeedableRng};
use sqlparser::ast::Ident;
use std::{error::Error, fs::File, time::Instant};

const DORY_SETUP_MAX_NU: usize = 8;
const DORY_SEED: [u8; 32] = *b"tech-gadget-prices-dataset-seed!";

fn prove_and_verify_query(
    sql: &str,
    accessor: &OwnedTableTestAccessor<DynamicDoryEvaluationProof>,
    prover_setup: &ProverSetup,
    verifier_setup: &VerifierSetup,
) -> Result<(), Box<dyn Error>> {
    println!("Parsing the query: {sql}...");
    let now = Instant::now();
    let query_plan = QueryExpr::try_new(sql.parse()?, Ident::new("tech_gadget_prices"), accessor)?;
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    print!("Generating proof...");
    let now = Instant::now();
    let verifiable_result = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        query_plan.proof_expr(),
        accessor,
        &prover_setup,
    );
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    print!("Verifying proof...");
    let now = Instant::now();
    let result = verifiable_result.verify(query_plan.proof_expr(), accessor, &verifier_setup)?;
    println!("Verified in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    println!("Query Result:");
    println!("{:?}", result.table);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::from_seed(DORY_SEED);
    let public_parameters = PublicParameters::rand(DORY_SETUP_MAX_NU, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    let filename = "./tech_gadget_prices/tech_gadget_prices.csv";
    let schema = infer_schema_from_files(&[filename.to_string()], b',', None, true)?;
    let data_batch = ReaderBuilder::new(SchemaRef::new(schema))
        .with_header(true)
        .build(File::open(filename)?)?
        .next()
        .ok_or("No data found in CSV file")??;

    let accessor = OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_from_table(
        "tech_gadget_prices.prices".parse()?,
        OwnedTable::try_from(data_batch)?,
        0,
        &prover_setup,
    );

    prove_and_verify_query(
        "SELECT COUNT(*) AS total FROM prices",
        &accessor,
        &prover_setup,
        &verifier_setup,
    )?;
    prove_and_verify_query(
        "SELECT Brand, COUNT(*) AS total FROM prices GROUP BY Brand ORDER BY total",
        &accessor,
        &prover_setup,
        &verifier_setup,
    )?;
    prove_and_verify_query(
        "SELECT Name, Price FROM prices WHERE Category = 'Smartphone' ORDER BY Price DESC LIMIT 3",
        &accessor,
        &prover_setup,
        &verifier_setup,
    )?;
    prove_and_verify_query(
        "SELECT Name, ReleaseYear FROM prices WHERE Price > 500 ORDER BY ReleaseYear DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    )?;
    Ok(())
}
