use crate::pip::multiplication::proof::*;

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use pedersen::commitments::compute_commitments;
use std::slice;

use crate::base::proof::Commitment;
use crate::base::proof::PIPProof;
use crate::base::proof::Transcript;

#[test]
fn test_create_verify_proof() {
    // create a proof
    let a = vec![Scalar::from(1u64), Scalar::from(7u64), Scalar::from(5u64)];
    let b = vec![Scalar::from(3u64), Scalar::from(10u64), Scalar::from(2u64)];

    
    let mut c_a = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut c_a), &[&a[..]]);
    let commitment_a = Commitment {
        commitment: c_a,
        length: a.len(),
    };
    let mut c_b = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut c_b), &[&b[..]]);
    let commitment_b = Commitment {
        commitment: c_b,
        length: b.len(),
    };


    let mut transcript = Transcript::new(b"multiplicationtest");
    // Note: this is not passing the output $a \boxtimes b$ as a parameter.
    let proof = MultiplicationProof::create(&mut transcript, &[&a, &b], &[], &[commitment_a.clone(), commitment_b.clone()]);

    // verify proof
    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof
        .verify(&mut transcript, &[commitment_a, commitment_b])
        .is_ok());
}
