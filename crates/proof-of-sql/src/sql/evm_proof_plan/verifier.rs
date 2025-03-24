use crate::proof_primitive::hyperkzg::HyperKZGPublicSetupOwned;
use crate::sql::proof::ProofPlan;
use crate::{
    base::database::{
        owned_table_utility::{bigint, owned_table},
        CommitmentAccessor, OwnedTableTestAccessor,
    },
    proof_primitive::hyperkzg::{
        deserialize_flat_compressed_hyperkzg_public_setup_from_reader,
        nova_commitment_key_to_hyperkzg_public_setup, HyperKZGCommitment,
        HyperKZGCommitmentEvaluationProof, HyperKZGEngine,
    },
    sql::{evm_proof_plan::EVMProofPlan, parse::QueryExpr, proof::VerifiableQueryResult},
};
use alloc::{fmt::Write, string::String};
use ark_bn254::G1Affine;
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField as _;
use ark_serialize::Validate;
use ff::PrimeField;
use itertools::Itertools;
use nova_snark::{
    provider::hyperkzg::{CommitmentEngine, CommitmentKey, EvaluationEngine},
    traits::{commitment::CommitmentEngineTrait, evaluation::EvaluationEngineTrait},
};

fn hex(data: &[u8]) -> String {
    data.iter()
        .fold(String::with_capacity(data.len() * 2), |mut s, c| {
            write!(s, "{c:02x}").unwrap();
            s
        })
}
fn evm_verifier_with_extra_args(
    query: &QueryExpr,
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
    accessor: &impl CommitmentAccessor<HyperKZGCommitment>,
    extra_args: &[&'static str],
) -> bool {
    let commitments = query
        .proof_expr()
        .get_column_references()
        .into_iter()
        .map(|c| accessor.get_commitment(c))
        .flat_map(|c| {
            c.commitment
                .into_affine()
                .xy()
                .map_or(["0".to_string(), "0".to_string()], |(x, y)| {
                    [x.into_bigint().to_string(), y.into_bigint().to_string()]
                })
        })
        .join(",");
    let table_lengths = query
        .proof_expr()
        .get_table_references()
        .into_iter()
        .map(|t| accessor.get_length(&t).to_string())
        .join(",");

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
            dbg!(hex(&result_bytes)),
            dbg!(hex(&query_bytes)),
            dbg!(hex(&proof_bytes)),
        ])
        .arg(dbg!(format!("[{table_lengths}]")))
        .arg(dbg!(format!("[{commitments}]")))
        .output()
        .unwrap()
        .status
        .success()
}
fn evm_verifier_all(
    query: &QueryExpr,
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
    accessor: &impl CommitmentAccessor<HyperKZGCommitment>,
) -> bool {
    evm_verifier_with_extra_args(query, verifiable_result, accessor, &[])
        && evm_verifier_with_extra_args(query, verifiable_result, accessor, &["--via-ir"])
        && evm_verifier_with_extra_args(query, verifiable_result, accessor, &["--optimize"])
        && evm_verifier_with_extra_args(
            query,
            verifiable_result,
            accessor,
            &["--optimize", "--via-ir"],
        )
}

fn load_setups() -> (
    HyperKZGPublicSetupOwned,
    nova_snark::provider::hyperkzg::VerifierKey<HyperKZGEngine>,
) {
    let h: halo2curves::bn256::G1Affine = halo2curves::bn256::G1Affine::generator();
    let tau_H: halo2curves::bn256::G2Affine = halo2curves::bn256::G2Affine {
        x: halo2curves::bn256::Fq2::new(
            halo2curves::bn256::Fq::from_str_vartime(
                "18253511544609001572866960948873128266198935669250718031100637619547827597184",
            )
            .unwrap(),
            halo2curves::bn256::Fq::from_str_vartime(
                "10764647077472957448033591885865458661573660819003350325268673957890498500987",
            )
            .unwrap(),
        ),
        y: halo2curves::bn256::Fq2::new(
            halo2curves::bn256::Fq::from_str_vartime(
                "19756181390911900613508142947142748782977087973617411469215564659012323409872",
            )
            .unwrap(),
            halo2curves::bn256::Fq::from_str_vartime(
                "15207030507740967976352749097256929091435606784526748170016829002013506957017",
            )
            .unwrap(),
        ),
    };
    let (_, vk) = EvaluationEngine::<HyperKZGEngine>::setup(&CommitmentKey::new(vec![], h, tau_H));

    let file = std::fs::File::open("test_assets/ppot_0080_10.bin").unwrap();
    let mut ps =
        deserialize_flat_compressed_hyperkzg_public_setup_from_reader(&file, Validate::Yes)
            .unwrap();

    ps.insert(0, G1Affine::generator());

    (ps, vk)
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_using_the_evm() {
    let (ps, vk) = load_setups();

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

    verifiable_result
        .clone()
        .verify(
            &EVMProofPlan::new(query.proof_expr().clone()),
            &accessor,
            &&vk,
        )
        .unwrap();

    assert!(evm_verifier_all(&query, &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_with_negative_literal_using_the_evm() {
    let (ps, vk) = load_setups();

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

    assert!(evm_verifier_all(&query, &verifiable_result, &accessor));

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
    let (ps, vk) = load_setups();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2, 102, 104, 107, 108]),
            bigint("b", [0, 1, 2, 3, 4, 5, 33, 44, 55, 6]),
            bigint("c", [6, 7, 8, 9, 10, 11, 14, 15, 73, 23]),
            bigint("d", [5, 7, 2, 5, 4, 1, 12, 22, 22, 22]),
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

    assert!(evm_verifier_all(&query, &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(
            &EVMProofPlan::new(query.proof_expr().clone()),
            &accessor,
            &&vk,
        )
        .unwrap();
}
