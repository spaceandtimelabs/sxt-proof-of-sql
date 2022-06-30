use crate::pip::hadamard::proof::*;

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use pedersen::commitments::compute_commitments;
use rand_core::SeedableRng;
use std::slice;

use crate::base::proof::{Commitment, PIPProof, Transcript};

fn test_helper_create(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(123);

    // create a proof
    let a_vec: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
    let b_vec: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
    let ab_vec: Vec<Scalar> = a_vec.iter().zip(b_vec.iter()).map(|(a, b)| a * b).collect();

    let mut c_a = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut c_a), &[&a_vec[..]]);
    let commitment_a = Commitment {
        commitment: c_a,
        length: a_vec.len(),
    };
    let mut c_b = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut c_b), &[&b_vec[..]]);
    let commitment_b = Commitment {
        commitment: c_b,
        length: b_vec.len(),
    };

    let mut transcript = Transcript::new(b"hadamardtest");
    let proof = HadamardProof::create(
        &mut transcript,
        &[&a_vec, &b_vec],
        &[&ab_vec],
        &[commitment_a, commitment_b],
    );

    // verify proof
    let mut transcript = Transcript::new(b"hadamardtest");
    assert!(proof
        .verify(&mut transcript, &[commitment_a, commitment_b])
        .is_ok());

    // verify fails if the wrong transcript is used
    if n > 1 {
        let mut transcript = Transcript::new(b"invalid");
        assert!(proof
            .verify(&mut transcript, &[commitment_a, commitment_b])
            .is_err());
    }

    // verify fails if commit_a doesn't match
    let mut transcript = Transcript::new(b"hadamardtest");
    let not_commitment_a = Commitment {
        commitment: CompressedRistretto::identity(),
        length: a_vec.len(),
    };
    assert!(proof
        .verify(&mut transcript, &[not_commitment_a, commitment_b])
        .is_err());

    // verify fails if commit_b doesn't match
    let mut transcript = Transcript::new(b"hadamardtest");
    let not_commitment_b = Commitment {
        commitment: CompressedRistretto::identity(),
        length: b_vec.len(),
    };
    assert!(proof
        .verify(&mut transcript, &[commitment_a, not_commitment_b])
        .is_err());

    // verify fails if commit_ab doesn't match
    let mut bad_proof = proof.clone();
    bad_proof.commit_ab.commitment = CompressedRistretto::identity();
    let mut transcript = Transcript::new(b"hadamardtest");
    assert!(bad_proof
        .verify(&mut transcript, &[commitment_a, commitment_b])
        .is_err());

    // verify fails if f_a doesn't match
    let mut bad_proof = proof.clone();
    bad_proof.f_a = Scalar::one();
    let mut transcript = Transcript::new(b"hadamardtest");
    assert!(bad_proof
        .verify(&mut transcript, &[commitment_a, commitment_b])
        .is_err());

    // verify fails if f_b doesn't match
    let mut bad_proof = proof.clone();
    bad_proof.f_b = Scalar::one();
    let mut transcript = Transcript::new(b"hadamardtest");
    assert!(bad_proof
        .verify(&mut transcript, &[commitment_a, commitment_b])
        .is_err());

    // verify fails if the hadamard product is wrong
    let mut transcript = Transcript::new(b"hadamardtest");
    let bad_proof = HadamardProof::create(
        &mut transcript,
        &[&a_vec, &b_vec],
        &[&a_vec],
        &[commitment_a, commitment_b],
    );
    let mut transcript = Transcript::new(b"hadamardtest");
    assert!(bad_proof
        .verify(&mut transcript, &[commitment_a, commitment_b])
        .is_err());
}

#[test]
fn test_zero_proof() {
    let n = 1;
    let a_vec = vec![Scalar::zero(); n];

    let commitment = Commitment {
        commitment: CompressedRistretto::identity(),
        length: a_vec.len(),
    };

    let mut transcript = Transcript::new(b"hadamardtest");
    let proof = HadamardProof::create(
        &mut transcript,
        &[&a_vec, &a_vec],
        &[&a_vec],
        &[commitment, commitment],
    );

    // verify proof
    let mut transcript = Transcript::new(b"hadamardtest");
    assert!(proof
        .verify(&mut transcript, &[commitment, commitment])
        .is_ok());
}

#[test]
fn make_proof_1() {
    test_helper_create(1);
}

#[test]
fn make_proof_2() {
    test_helper_create(2);
}

#[test]
fn make_proof_3() {
    test_helper_create(3);
}

#[test]
fn make_proof_16() {
    test_helper_create(16);
}
