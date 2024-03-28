use ark_std::test_rng;
use proofs::{
    base::database::{OwnedTableTestAccessor, TestAccessor},
    owned_table,
    proof_primitive::dory::{DoryEvaluationProof, DoryProverPublicSetup},
    sql::{parse::QueryExpr, proof::QueryProof},
};
use wasm_bindgen::prelude::*;

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

fn main() {
    let dory_prover_setup = DoryProverPublicSetup::rand(4, 3, &mut test_rng());
    let dory_verifier_setup = (&dory_prover_setup).into();

    let mut accessor = OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(
        dory_prover_setup.clone(),
    );
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table!("a" => [1i64, 2, 3], "b" => [1i64, 0, 1]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b = 1".parse().unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();
    let (proof, serialized_result) =
        QueryProof::<DoryEvaluationProof>::new(query.proof_expr(), &accessor, &dory_prover_setup);
    let owned_table_result = proof
        .verify(
            query.proof_expr(),
            &accessor,
            &serialized_result,
            &dory_verifier_setup,
        )
        .unwrap()
        .table;
    let expected_result = owned_table!("a" => [1i64, 3], "b" => [1i64, 1]);
    let result_match = owned_table_result == expected_result;

    if cfg!(all(target_arch = "wasm32", target_os = "unknown")) {
        console_log!("Result matches: {}", result_match);
    } else {
        println!("Result matches: {}", result_match);
    }

    assert!(result_match);
}
