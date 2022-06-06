use crate::pip::multiplication::proof::*;
use crate::base::proof::PIPProof;
use crate::base::proof::Commitment;

use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use sha3::Sha3_512;

#[test]
fn test_create_verify_proof() {
    // create a proof
    let a = vec![Scalar::from(1u64), Scalar::from(7u64), Scalar::from(5u64)];
    let b = vec![Scalar::from(3u64), Scalar::from(10u64), Scalar::from(2u64)];
    let mut transcript = Transcript::new(b"multiplicationtest");
    let proof = MultiplicationProof::create(&mut transcript, vec![&a, &b], vec![]);

    // verify proof
    let mut transcript = Transcript::new(b"multiplicationtest");
    let c_a = RistrettoPoint::hash_from_bytes::<Sha3_512>(b"a").compress(); // pretend like this is the commitment of a
    let commitment_a = Commitment{commitment : c_a, length : a.len()};
    let c_b = RistrettoPoint::hash_from_bytes::<Sha3_512>(b"b").compress(); // pretend like this is the commitment of b
    let commitment_b = Commitment{commitment : c_b, length : b.len()};

    assert!(proof.verify(&mut transcript, vec![commitment_a, commitment_b], vec![]).is_ok());
}
