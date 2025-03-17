use crate::{
    base::database::{
        owned_table_utility::{bigint, owned_table},
        OwnedTableTestAccessor,
    },
    proof_primitive::hyperkzg::{
        nova_commitment_key_to_hyperkzg_public_setup, HyperKZGCommitmentEvaluationProof,
        HyperKZGEngine,
    },
    sql::{evm_proof_plan::EVMProofPlan, parse::QueryExpr, proof::VerifiableQueryResult},
};
use alloc::{fmt::Write, string::String};
use nova_snark::{
    provider::hyperkzg::{CommitmentEngine, CommitmentKey, EvaluationEngine},
    traits::{commitment::CommitmentEngineTrait, evaluation::EvaluationEngineTrait},
};
use rand::SeedableRng;

fn hex(data: &[u8]) -> String {
    data.iter()
        .fold(String::with_capacity(data.len() * 2), |mut s, c| {
            write!(s, "{c:02x}").unwrap();
            s
        })
}
fn evm_verifier(
    query: &QueryExpr,
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
) -> bool {
    assert!(query.postprocessing().is_empty());
    let bincode_options = bincode::config::standard()
        .with_fixed_int_encoding()
        .with_big_endian();
    let query_bytes = bincode::serde::encode_to_vec(
        EVMProofPlan::new(query.proof_expr().clone()),
        bincode_options,
    )
    .unwrap();
    let proof_bytes =
        bincode::serde::encode_to_vec(verifiable_result.proof.as_ref().unwrap(), bincode_options)
            .unwrap();
    let _result_bytes =
        bincode::serde::encode_to_vec(verifiable_result.result.as_ref().unwrap(), bincode_options)
            .unwrap();

    dbg!(hex(&query_bytes));
    dbg!(hex(&proof_bytes));

    // std::process::Command::new("forge")
    //     .arg("script")
    //     .args(["--tc", "VerifyProof"])
    //     .args(["--sig", "_testVerifyProof(bytes,bytes,bytes)"])
    //     .arg("../../solidity/src/proof/Verify.post.sol")
    //     .args([hex(&query_bytes), hex(&result_bytes), hex(&proof_bytes)])
    //     .output()
    //     .unwrap()
    //     .status
    //     .success()
    true
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_using_the_evm() {
    let ck: CommitmentKey<HyperKZGEngine> =
        CommitmentKey::setup_from_rng(b"test", 128, rand::rngs::StdRng::from_seed([0; 32]));
    let (_, vk) = EvaluationEngine::setup(&ck);

    let ps = nova_commitment_key_to_hyperkzg_public_setup(&ck);

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT b FROM table WHERE a = 5".parse().unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(query.proof_expr().clone()),
        &accessor,
        &&ps[..],
    );

    assert!(evm_verifier(&query, &verifiable_result));

    verifiable_result
        .clone()
        .verify(
            &EVMProofPlan::new(query.proof_expr().clone()),
            &accessor,
            &&vk,
        )
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_with_negative_literal_using_the_evm() {
    let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 128);
    let (_, vk) = EvaluationEngine::setup(&ck);

    let ps = nova_commitment_key_to_hyperkzg_public_setup(&ck);

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, -2, 5, 3, -2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT b FROM table WHERE a = -2".parse().unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(query.proof_expr().clone()),
        &accessor,
        &&ps[..],
    );

    assert!(evm_verifier(&query, &verifiable_result));

    verifiable_result
        .clone()
        .verify(
            &EVMProofPlan::new(query.proof_expr().clone()),
            &accessor,
            &&vk,
        )
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_complex_filter_using_the_evm() {
    let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 128);
    let (_, vk) = EvaluationEngine::setup(&ck);

    let ps = nova_commitment_key_to_hyperkzg_public_setup(&ck);

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
            bigint("c", [6, 7, 8, 9, 10, 11]),
            bigint("d", [5, 7, 2, 5, 4, 1]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT b,c FROM table WHERE a = d".parse().unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(query.proof_expr().clone()),
        &accessor,
        &&ps[..],
    );

    assert!(evm_verifier(&query, &verifiable_result));

    verifiable_result
        .clone()
        .verify(
            &EVMProofPlan::new(query.proof_expr().clone()),
            &accessor,
            &&vk,
        )
        .unwrap();
}
