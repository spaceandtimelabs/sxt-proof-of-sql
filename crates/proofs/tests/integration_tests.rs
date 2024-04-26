#![cfg(feature = "test")]
use ark_std::test_rng;
use arrow::record_batch::RecordBatch;
#[cfg(feature = "blitzar")]
use proofs::base::commitment::InnerProductProof;
use proofs::{
    base::{
        database::{OwnedTable, OwnedTableTestAccessor, TestAccessor},
        scalar::Curve25519Scalar,
    },
    owned_table,
    proof_primitive::dory::{DoryEvaluationProof, DoryProverPublicSetup},
    record_batch,
    sql::{
        parse::QueryExpr,
        proof::{QueryProof, TransformExpr},
    },
};

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_equality_query_with_curve25519() {
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
fn we_can_prove_a_basic_equality_query_with_dory() {
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

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_inequality_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table!("a" => [1i64, 2, 3], "b" => [1i64, 0, 2]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b >= 1".parse().unwrap(),
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
    let expected_result = owned_table!("a" => [1i64, 3], "b" => [1i64, 2]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_basic_inequality_query_with_dory() {
    let dory_prover_setup = DoryProverPublicSetup::rand(4, 3, &mut test_rng());
    let dory_verifier_setup = (&dory_prover_setup).into();

    let mut accessor = OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(
        dory_prover_setup.clone(),
    );
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table!("a" => [1i64, 2, 3], "b" => [1i64, 0, 4]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b <= 0".parse().unwrap(),
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
    let expected_result = owned_table!("a" => [2i64], "b" => [0i64]);
    assert_eq!(owned_table_result, expected_result);
}

//TODO: This test uses postprocessing now. Check proofs results once PROOF-765 is done.
#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_minimal_group_by_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table!("a" => [1i64, 1, 2, 2, 3], "b" => [1i64, 0, 2, 3, 4]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a, count(*) as c FROM table group by a"
            .parse()
            .unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();
    let (proof, serialized_result) =
        QueryProof::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result: OwnedTable<Curve25519Scalar> = proof
        .verify(query.proof_expr(), &accessor, &serialized_result, &())
        .unwrap()
        .table;
    let transformed_result: RecordBatch = query
        .result()
        .transform_results(owned_table_result.clone().try_into().unwrap());
    let expected_result: RecordBatch = record_batch!("a" => [1i64, 2, 3], "c" => [2i64, 2, 1]);
    assert_eq!(transformed_result, expected_result);
}

#[test]
fn we_can_prove_a_basic_group_by_query_with_dory() {
    let dory_prover_setup = DoryProverPublicSetup::rand(4, 3, &mut test_rng());
    let dory_verifier_setup = (&dory_prover_setup).into();

    let mut accessor = OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(
        dory_prover_setup.clone(),
    );
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table!("a" => [1i64, 1, 2, 3, 2], "b" => [1i64, 0, 4, 2, 3], "c" => [-2i64, 2, 1, 0, 1]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a, sum(b) as d, count(*) as e FROM table WHERE c >= 0 group by a"
            .parse()
            .unwrap(),
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
    let expected_result =
        owned_table!("a" => [1i64, 2, 3], "d" => [0i64, 7, 2], "e" => [1i64, 2, 1]);
    assert_eq!(owned_table_result, expected_result);
}
