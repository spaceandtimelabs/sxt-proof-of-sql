//! Example to use Proof of SQL with a tech gadget prices dataset.
//! To run, use `cargo run --example tech_gadget_prices`.

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
use std::{fs::File, time::Instant};

const DORY_SETUP_MAX_NU: usize = 8;
const DORY_SEED: [u8; 32] = *b"tech-gadget-prices-dataset-seed";
fn prove_and_verify_query(
    sql: &str,
    accessor: &OwnedTableTestAccessor<DynamicDoryEvaluationProof>,
    prover_setup: &ProverSetup,
    verifier_setup: &VerifierSetup,
) {
    println!("Parsing the query: {sql}...");
    let now = Instant::now();
    let query_plan = QueryExpr::<DynamicDoryCommitment>::try_new(
        sql.parse().unwrap(),
        "tech_gadget_prices".parse().unwrap(),
        accessor,
    )
    .unwrap();
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);
    print!("Generating proof...");
    let now = Instant::now();
    let (proof, provable_result) = QueryProof::<DynamicDoryEvaluationProof>::new(
        query_plan.proof_expr(),
        accessor,
        &prover_setup,
    );
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);
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
    println!("Verified in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    println!("Query Result:");
    println!("{:?}", result.table);
}

fn main() {
    let mut rng = StdRng::from_seed(DORY_SEED);
    let public_parameters = PublicParameters::rand(DORY_SETUP_MAX_NU, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    let filename = "./tech_gadget_prices/tech_gadget_prices.csv";
    let data_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
    .with_header(true)
    .build(File::open(filename).unwrap())
    .unwrap()
    .next()
    .unwrap()
    .unwrap();

    let accessor = OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_from_table(
        "tech_gadget_prices.prices".parse().unwrap(),
        OwnedTable::try_from(data_batch).unwrap(),
        0,
        &prover_setup,
    );

    prove_and_verify_query(
        "SELECT COUNT(*) AS total FROM prices",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Brand, COUNT(*) AS total FROM prices GROUP BY Brand ORDER BY total",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Name, Price FROM prices WHERE Category = 'Smartphone' ORDER BY Price DESC LIMIT 3",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Name, ReleaseYear FROM prices WHERE Price > 500 ORDER BY ReleaseYear DESC",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
