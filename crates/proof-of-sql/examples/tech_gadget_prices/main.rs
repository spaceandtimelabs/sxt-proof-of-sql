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
