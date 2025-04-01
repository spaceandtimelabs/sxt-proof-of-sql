use super::HyperKZGCommitment;
use crate::{
    base::{
        commitment::{CommitmentEvaluationProof, VecCommitmentExt},
        database::Column,
        polynomial::compute_evaluation_vector,
        proof::{Keccak256Transcript, Transcript},
        scalar::MontScalar,
    },
    proof_primitive::hyperkzg::{
        public_setup::load_small_setup_for_testing, BNScalar, HyperKZGCommitmentEvaluationProof,
    },
};
use ark_ff::{PrimeField, UniformRand};
use bincode::config::{BigEndian, Configuration, Fixint};
use itertools::Itertools;
use rand::Rng;

const BINCODE_CONFIG: Configuration<BigEndian, Fixint> = bincode::config::standard()
    .with_big_endian()
    .with_fixed_int_encoding();

fn run_evm_verify_hyperkzg(
    commit: HyperKZGCommitment,
    proof: &HyperKZGCommitmentEvaluationProof,
    state: [u8; 32],
    x: &[BNScalar],
    y: BNScalar,
) {
    run_evm_verify_hyperkzg_with_extra_args(commit, proof, state, x, y, &[]);
    run_evm_verify_hyperkzg_with_extra_args(commit, proof, state, x, y, &["--via-ir"]);
    run_evm_verify_hyperkzg_with_extra_args(commit, proof, state, x, y, &["--optimize"]);
    run_evm_verify_hyperkzg_with_extra_args(
        commit,
        proof,
        state,
        x,
        y,
        &["--via-ir", "--optimize"],
    );
}
fn run_evm_verify_hyperkzg_with_extra_args(
    commit: HyperKZGCommitment,
    proof: &HyperKZGCommitmentEvaluationProof,
    state: [u8; 32],
    x: &[BNScalar],
    y: BNScalar,
    extra_args: &[&'static str],
) {
    let commits_bytes = bincode::serde::encode_to_vec(commit, BINCODE_CONFIG).unwrap();
    let proof_bytes = bincode::serde::encode_to_vec(proof, BINCODE_CONFIG).unwrap();
    assert!(
        std::process::Command::new("../../solidity/scripts/pre_forge.sh")
            .arg("script")
            .args(extra_args)
            .args(["--tc", "HyperKZGVerifierTest"])
            .args([
                "--sig",
                "verifyHyperKZG(bytes,uint256[1],uint256[2],uint256[],uint256)"
            ])
            .arg("./test/hyperkzg/HyperKZGVerifier.t.post.sol")
            .arg(hex::encode(&proof_bytes))
            .arg(format!("[0x{}]", hex::encode(state)))
            .arg(format!(
                "[0x{},0x{}]",
                hex::encode(&commits_bytes[..32]),
                hex::encode(&commits_bytes[32..])
            ))
            .arg(format!(
                "[{}]",
                x.iter().map(|x| x.0.into_bigint().to_string()).join(",")
            ))
            .arg(y.0.into_bigint().to_string())
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[ignore = "foundry must be installed in order to run this test"]
#[test]
fn we_can_create_small_valid_proof_for_use_in_solidity_tests() {
    let (ps, vk) = load_small_setup_for_testing();

    let a = [
        BNScalar::from(0),
        BNScalar::from(1),
        BNScalar::from(2),
        BNScalar::from(3),
    ];
    let x = [BNScalar::from(7), BNScalar::from(5)];

    let mut b_vec = vec![BNScalar::default(); a.len()];
    compute_evaluation_vector(&mut b_vec, &x);
    let y: BNScalar = a.iter().zip(b_vec).map(|(a, b)| *a * b).sum();
    let commit = Vec::from_columns_with_offset([Column::Scalar(&a)], 0, &&ps[..])[0];

    let mut transcript = Keccak256Transcript::new();
    let proof = <HyperKZGCommitmentEvaluationProof>::new(&mut transcript, &a, &x, 0, &&ps[..]);

    let mut transcript = Keccak256Transcript::new();
    let r = proof.verify_proof(&mut transcript, &commit, &y, &x, 0, a.len(), &&vk);
    assert!(r.is_ok());

    let mut transcript = Keccak256Transcript::new();
    let state = transcript.challenge_as_le();

    run_evm_verify_hyperkzg(commit, &proof, state, &x, y);
}

#[ignore = "foundry must be installed in order to run this test"]
#[test]
fn we_can_generate_and_verify_random_hyperkzg_proofs() {
    let (ps, vk) = load_small_setup_for_testing();

    let mut rng = ark_std::test_rng();

    for nu in 1..9 {
        let len = 1 << nu;

        let a = (0..len)
            .map(|_| MontScalar(ark_ff::Fp::rand(&mut rng)))
            .collect::<Vec<_>>();
        let x = (0..nu)
            .map(|_| MontScalar(ark_ff::Fp::rand(&mut rng)))
            .collect::<Vec<_>>();

        let mut b_vec = vec![BNScalar::default(); a.len()];
        compute_evaluation_vector(&mut b_vec, &x);
        let y: BNScalar = a.iter().zip(b_vec).map(|(a, b)| *a * b).sum();
        let commit = Vec::from_columns_with_offset([Column::Scalar(&a)], 0, &&ps[..])[0];

        let mut transcript = Keccak256Transcript::new();
        let proof = <HyperKZGCommitmentEvaluationProof>::new(&mut transcript, &a, &x, 0, &&ps[..]);

        let mut transcript = Keccak256Transcript::new();
        let r = proof.verify_proof(&mut transcript, &commit, &y, &x, 0, a.len(), &&vk);
        assert!(r.is_ok());

        let mut transcript = Keccak256Transcript::new();
        let state = transcript.challenge_as_le();

        run_evm_verify_hyperkzg(commit, &proof, state, &x, y);
    }
}

#[ignore = "foundry must be installed in order to run this test"]
#[test]
fn we_can_generate_and_verify_random_hyperkzg_proofs_with_random_length() {
    let (ps, vk) = load_small_setup_for_testing();

    let mut rng = ark_std::test_rng();

    for nu in 1..8 {
        let len = rng.gen_range((1 << (nu - 1)) + 1..=1 << nu);

        let a = (0..len)
            .map(|_| MontScalar(ark_ff::Fp::rand(&mut rng)))
            .collect::<Vec<_>>();
        let x = (0..nu)
            .map(|_| MontScalar(ark_ff::Fp::rand(&mut rng)))
            .collect::<Vec<_>>();

        let mut b_vec = vec![BNScalar::default(); a.len()];
        compute_evaluation_vector(&mut b_vec, &x);
        let y: BNScalar = a.iter().zip(b_vec).map(|(a, b)| *a * b).sum();
        let commit = Vec::from_columns_with_offset([Column::Scalar(&a)], 0, &&ps[..])[0];

        let mut transcript = Keccak256Transcript::new();
        let proof = <HyperKZGCommitmentEvaluationProof>::new(&mut transcript, &a, &x, 0, &&ps[..]);

        let mut transcript = Keccak256Transcript::new();
        let r = proof.verify_proof(&mut transcript, &commit, &y, &x, 0, a.len(), &&vk);
        assert!(r.is_ok());

        let mut transcript = Keccak256Transcript::new();
        let state = transcript.challenge_as_le();

        run_evm_verify_hyperkzg(commit, &proof, state, &x, y);
    }
}
