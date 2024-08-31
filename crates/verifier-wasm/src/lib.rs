use proof_of_sql::{
    base::{
        commitment::{CommitmentEvaluationProof, QueryCommitments},
        database::OwnedTable,
    },
    proof_primitive::dory::{
        DoryCommitment, DoryEvaluationProof, DoryScalar, DoryVerifierPublicSetup, VerifierSetup,
    },
    sql::{
        parse::QueryExpr,
        proof::{ProvableQueryResult, QueryProof},
    },
};
use wasm_bindgen::prelude::wasm_bindgen;

const EXIT_STATUS_VERIFICATION_SUCCESS: i32 = 0;
const EXIT_STATUS_VERIFICATION_FAIL: i32 = 1;
const EXIT_STATUS_BAD_PARAM: i32 = 2;

// Define a console_log macro as a replacement for println,
// which doesn't work on the wasm32-unknown-unknown target.
// Code for this macro was copied from the examples section
// of the wasm-bindgen documentation.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: &str);
}
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn run_verification<CP: CommitmentEvaluationProof>(
    query: &str,
    default_schema: &str,
    query_commitments: &QueryCommitments<CP::Commitment>,
    proof: &QueryProof<CP>,
    serialized_result: &ProvableQueryResult,
    verifier_setup: &CP::VerifierPublicSetup<'_>,
) -> Option<OwnedTable<CP::Scalar>> {
    let query_expr = QueryExpr::try_new(
        query.parse().ok()?,
        default_schema.parse().ok()?,
        query_commitments,
    )
    .ok()?;
    Some(
        proof
            .verify(
                query_expr.proof_expr(),
                query_commitments,
                serialized_result,
                verifier_setup,
            )
            .ok()?
            .table,
    )
}

/// This method verifies a proof for a given query and returns the result if the proof is valid.
/// This method will return `None` if the results cannot be verified.
///
/// The inputs are:
///     - `query`: The SQL query to verify.
///     - `default_schema`: The default schema to use for the query.
///     - `query_commitments`: The commitments to the columns of data used in the query.
///     - `proof`: The proof of the query result.
///     - `serialized_result`: The serialized result of the query.
///     - `verifier_setup`: The public setup for the verifier.
pub fn run_dory_verification(
    query: &str,
    default_schema: &str,
    query_commitments: &QueryCommitments<DoryCommitment>,
    proof: &QueryProof<DoryEvaluationProof>,
    serialized_result: &ProvableQueryResult,
    verifier_setup: &DoryVerifierPublicSetup,
) -> Option<OwnedTable<DoryScalar>> {
    run_verification(
        query,
        default_schema,
        query_commitments,
        proof,
        serialized_result,
        verifier_setup,
    )
}

#[wasm_bindgen]
pub fn verify(
    query: &str,
    schema: &str,
    query_commitments_ser: &[u8],
    proof_ser: &[u8],
    serialized_result_ser: &[u8],
    verifier_setup_ser: &[u8],
    sigma: usize,
) -> i32 {
    console_log!("Wasm verifier start");

    // Deserialize arguments
    let query_commitments: QueryCommitments<DoryCommitment> =
        match bincode::deserialize(query_commitments_ser) {
            Ok(val) => val,
            Err(_) => return EXIT_STATUS_BAD_PARAM,
        };

    let proof: QueryProof<DoryEvaluationProof> = match bincode::deserialize(proof_ser) {
        Ok(val) => val,
        Err(_) => return EXIT_STATUS_BAD_PARAM,
    };

    let serialized_result: ProvableQueryResult = match bincode::deserialize(serialized_result_ser) {
        Ok(val) => val,
        Err(_) => return EXIT_STATUS_BAD_PARAM,
    };

    let verifier_setup: VerifierSetup = match bincode::deserialize(verifier_setup_ser) {
        Ok(val) => val,
        Err(_) => return EXIT_STATUS_BAD_PARAM,
    };

    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, sigma);

    // Run verifier
    let res = run_dory_verification(
        query,
        schema,
        &query_commitments,
        &proof,
        &serialized_result,
        &dory_verifier_setup,
    );

    match res {
        Some(owned_table_result) => {
            console_log!("Result: {:?}", owned_table_result);
            EXIT_STATUS_VERIFICATION_SUCCESS
        }
        None => EXIT_STATUS_VERIFICATION_FAIL,
    }
}
