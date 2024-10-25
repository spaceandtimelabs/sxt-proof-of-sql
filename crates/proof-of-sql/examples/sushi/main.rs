//! This is an non-interactive example of using Proof of SQL with some sushi related datasets.
//! To run this, use `cargo run --example sushi`.

//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example space --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.
use arrow::datatypes::SchemaRef;
use arrow_csv::{
	infer_schema_from_files, 
	ReaderBuilder
};
use std::{
	fs::File,
	time::Instant
};
use proof_of_sql::{
    base::database::{
		OwnedTable, 
		OwnedTableTestAccessor,
		TestAccessor
	},
    proof_primitive::dory::{
        DynamicDoryCommitment, 
		DynamicDoryEvaluationProof, 
		ProverSetup, 
		PublicParameters,
        VerifierSetup
    },
    sql::{
		parse::QueryExpr, 
		proof::QueryProof
	}
};
use rand::{
	rngs::StdRng, 
	SeedableRng
};

const DORY_SETUP_MAX_NU: usize = 8;
const DORY_SEED: [u8; 32] = *b"sushi-is-the-best-food-available";

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
        "sushi".parse().unwrap(),
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
    println!("Verified in {} ms.", now.elapsed().as_secs_f64() * 1000.);
    // Display the result
    println!("Query Result:");
    println!("{:?}", result.table);
}

fn main() {
	let mut rng = StdRng::from_seed(DORY_SEED);
    let public_parameters = PublicParameters::rand(DORY_SETUP_MAX_NU, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

	let filename = "./crates/proof-of-sql/examples/sushi/fish.csv";
    let fish_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
		.with_header(true)
		.build(File::open(filename).unwrap())
		.unwrap()
		.next()
		.unwrap()
		.unwrap();
    println!("{fish_batch:?}");

	// Load the table into an "Accessor" so that the prover and verifier can access the data/commitments.
    let mut accessor = OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    accessor.add_table("sushi.fish".parse().unwrap(), OwnedTable::try_from(fish_batch).unwrap(), 0);

	prove_and_verify_query(
        "SELECT * FROM fish",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

	prove_and_verify_query(
        "SELECT COUNT(*) FROM fish WHERE nameEn = 'Tuna'",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

	prove_and_verify_query(
        "SELECT kindEn FROM fish WHERE kindJa = 'Otoro'",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

	prove_and_verify_query(
        "SELECT kindEn FROM fish WHERE kindJa = 'Otoro'",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

	prove_and_verify_query(
        "SELECT * FROM fish WHERE pricePerPound > 25 AND pricePerPound < 75",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

	prove_and_verify_query(
        "SELECT kindJa, COUNT(*) FROM fish GROUP BY kindJa",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}