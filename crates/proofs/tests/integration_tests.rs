#![cfg(feature = "test")]
use ark_std::test_rng;
use proofs::{
    base::{
        commitment::InnerProductProof,
        database::{OwnedTableTestAccessor, TestAccessor},
    },
    owned_table,
    proof_primitive::dory::{DoryEvaluationProof, DoryProverPublicSetup},
    sql::{parse::QueryExpr, proof::QueryProof},
};

#[test]
fn we_can_prove_a_basic_query() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
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
        QueryProof::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = proof
        .verify(query.proof_expr(), &accessor, &serialized_result, &())
        .unwrap()
        .table;
    let expected_result = owned_table!("a" => [1i64, 3], "b" => [1i64, 1]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_basic_query_with_dory() {
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
    assert_eq!(owned_table_result, expected_result);
}
