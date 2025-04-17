use crate::{
    base::database::{
        owned_table_utility::{bigint, owned_table},
        CommitmentAccessor, OwnedTableTestAccessor,
    },
    proof_primitive::hyperkzg::{self, HyperKZGCommitment, HyperKZGCommitmentEvaluationProof},
    sql::{
        evm_proof_plan::EVMProofPlan,
        parse::QueryExpr,
        proof::{ProofPlan, VerifiableQueryResult},
        proof_plans::DynProofPlan,
    },
};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use itertools::Itertools;

fn evm_verifier_with_extra_args(
    plan: &DynProofPlan,
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
    accessor: &impl CommitmentAccessor<HyperKZGCommitment>,
    extra_args: &[&'static str],
) -> bool {
    let commitments = plan
        .get_column_references()
        .into_iter()
        .map(|c| accessor.get_commitment(&c.table_ref(), &c.column_id()))
        .flat_map(|c| {
            c.commitment
                .into_affine()
                .xy()
                .map_or(["0".to_string(), "0".to_string()], |(x, y)| {
                    [x.into_bigint().to_string(), y.into_bigint().to_string()]
                })
        })
        .join(",");
    let table_lengths = plan
        .get_table_references()
        .into_iter()
        .map(|t| accessor.get_length(&t).to_string())
        .join(",");

    let bincode_options = bincode::config::standard()
        .with_fixed_int_encoding()
        .with_big_endian();
    let query_bytes =
        bincode::serde::encode_to_vec(EVMProofPlan::new(plan.clone()), bincode_options).unwrap();
    let proof_bytes =
        bincode::serde::encode_to_vec(&verifiable_result.proof, bincode_options).unwrap();
    let result_bytes =
        bincode::serde::encode_to_vec(&verifiable_result.result, bincode_options).unwrap();

    std::process::Command::new("../../solidity/scripts/pre_forge.sh")
        .arg("script")
        .arg("-vvvvv")
        .args(extra_args)
        .args(["--tc", "VerifierTest"])
        .args(["--sig", "verify(bytes,bytes,bytes,uint256[],uint256[])"])
        .arg("./test/verifier/Verifier.t.post.sol")
        .args([
            dbg!(hex::encode(&result_bytes)),
            dbg!(hex::encode(&query_bytes)),
            dbg!(hex::encode(&proof_bytes)),
        ])
        .arg(dbg!(format!("[{table_lengths}]")))
        .arg(dbg!(format!("[{commitments}]")))
        .output()
        .unwrap()
        .status
        .success()
}
fn evm_verifier_all(
    plan: &DynProofPlan,
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
    accessor: &impl CommitmentAccessor<HyperKZGCommitment>,
) -> bool {
    evm_verifier_with_extra_args(plan, verifiable_result, accessor, &[])
        && evm_verifier_with_extra_args(plan, verifiable_result, accessor, &["--via-ir"])
        && evm_verifier_with_extra_args(plan, verifiable_result, accessor, &["--optimize"])
        && evm_verifier_with_extra_args(
            plan,
            verifiable_result,
            accessor,
            &["--optimize", "--via-ir"],
        )
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

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
    let plan = query.proof_expr();

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_with_negative_literal_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

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
    let plan = query.proof_expr();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_filter_with_arithmetic_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

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
        "SELECT a, b FROM table WHERE a + b = a - b"
            .parse()
            .unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let plan = query.proof_expr();

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_complex_filter_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2, 102, 104, 107, 108]),
            bigint("b", [0, 1, 2, 3, 4, 5, 33, 44, 55, 6]),
            bigint("c", [0, 7, 8, 9, 10, 11, 14, 15, 73, 23]),
            bigint("d", [5, 7, 2, 5, 4, 1, 12, 22, 22, 22]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT b,c FROM table WHERE a + b = d and b = a * c"
            .parse()
            .unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let plan = query.proof_expr();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}
