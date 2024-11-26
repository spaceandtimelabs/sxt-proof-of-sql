//! This is a non-interactive example of using Proof of SQL with a wood types dataset.
//! To run this, use `cargo run --release --example zk_data`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example zk_data --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.
use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use proof_of_sql::{
    base::database::{
        arrow_schema_utility::get_posql_compatible_schema, OwnedTable, OwnedTableTestAccessor,
        TestAccessor,
    },
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{parse::QueryExpr, postprocessing::apply_postprocessing_steps, proof::QueryProof},
};
use rand::{rngs::StdRng, SeedableRng};
use std::fs::File;
use std::{
    io::{stdout, Write},
    time::Instant,
};

// We generate the public parameters and the setups used by the prover and verifier for the Dory PCS.
// The `max_nu` should be set such that the maximum table size is less than `2^(2*max_nu-1)`.
const DORY_SETUP_MAX_NU: usize = 13;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"f3a8d12e6b7c4590a1f2e3d4b5c6a7b8";

/// # Panics
/// Will panic if the query does not parse or the proof fails to verify.
fn prove_and_verify_query(
    sql: &str,
    accessor: &OwnedTableTestAccessor<DynamicDoryEvaluationProof>,
    prover_setup: &ProverSetup,
    verifier_setup: &VerifierSetup,
) {
    // Parse the query:
    let timer = start_timer("NEW!!!!!!!!!!!!!!");
    println!();
    println!("Parsing the query: {sql}...");
    let query_plan = QueryExpr::try_new(
        sql.parse().unwrap(),
        "zk_data".parse().unwrap(),
        accessor,
    )
    .unwrap();
    end_timer(timer);

    // Generate the proof and result:
    let timer = start_timer("Generating proof...");
    let (proof, provable_result) = QueryProof::<DynamicDoryEvaluationProof>::new(
        query_plan.proof_expr(),
        accessor,
        &prover_setup,
    );
    end_timer(timer);

    // Verify the result with the proof:
    let timer = start_timer("Verifying proof...");
    let result = proof
        .verify(
            query_plan.proof_expr(),
            accessor,
            &provable_result,
            &verifier_setup,
        )
        .unwrap();
    let result = apply_postprocessing_steps(result.table, query_plan.postprocessing());
    end_timer(timer);
    println!();
    // Display the result
    println!("Query Result:");
    println!("+++++++++++++++++++++++++++++++++++++++++++++++++++++++");
    println!("{result:?}");
    println!("*******************************************************");
    println!()
}

/// # Panics
///
/// Will panic if flushing the output fails, which can happen due to issues with the underlying output stream.
fn start_timer(message: &str) -> Instant {
    print!("{message}...");
    stdout().flush().unwrap();
    Instant::now()
}
/// # Panics
///
/// This function does not panic under normal circumstances but may panic if the internal printing fails due to issues with the output stream.
fn end_timer(instant: Instant) {
    println!(" {:?} ms.", instant.elapsed().as_secs_f64() * 1000.);
}
fn main() {
    let timer = start_timer("setup about dorycommitment");
    let mut rng = StdRng::from_seed(DORY_SEED);
    let public_parameters = PublicParameters::rand(DORY_SETUP_MAX_NU, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    end_timer(timer);

    let timer = start_timer("Loading data");
    let filename = "./crates/proof-of-sql/examples/zk_data/zk_data.csv";
    let inferred_schema =
        SchemaRef::new(infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap());
    let posql_compatible_schema = get_posql_compatible_schema(&inferred_schema);

    let zk_data_batch = ReaderBuilder::new(posql_compatible_schema)
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
        "zk_data.transactions".parse().unwrap(),
        OwnedTable::try_from(zk_data_batch).unwrap(),
        0,
    );
    end_timer(timer);
    println!();
    

    prove_and_verify_query(
        "SELECT 
    MAX(High) AS MaxPrice,
    MIN(Low) AS MinPrice
    FROM 
    transactions
    WHERE 
    Opentime > timestamp '2000-01-01T00:00:00Z';",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT 
    *
FROM 
    transactions
WHERE 
    Close > Open
ORDER BY Close 
DESC LIMIT 3;",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT COUNT(*) AS total_rows FROM transactions WHERE 
    Opentime > timestamp '2000-01-01T00:00:00Z';",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT Opentime AS new_time , Volume FROM transactions WHERE 
    Close - Open > 20 ORDER BY Volume DESC LIMIT 5;",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT Opentime,SUM(Volume) AS sum_volume
FROM transactions
WHERE (Close - Open > 20 and Close - Open <= 45)
GROUP BY Opentime
ORDER BY Opentime DESC LIMIT 1;",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT count(*) FROM transactions WHERE 
     High > 20;",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );


    
}
