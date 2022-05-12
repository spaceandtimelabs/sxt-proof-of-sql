use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use sha3::Sha3_512;

use crate::pip::multiplication::MultiplicationProof;

#[test]
fn create_verify_proof() {
    // create a proof
    let a = vec![Scalar::from(1u64), Scalar::from(7u64)];
    let b = vec![Scalar::from(3u64), Scalar::from(10u64)];
    let mut transcript = Transcript::new(b"multiplicationtest");
    let proof = MultiplicationProof::create(&mut transcript, &a, &b);

    // verify proof
    let mut transcript = Transcript::new(b"multiplicationtest");
    let c_a = RistrettoPoint::hash_from_bytes::<Sha3_512>(b"a"); // pretend like this is the commitment of a
    let c_b = RistrettoPoint::hash_from_bytes::<Sha3_512>(b"b"); // pretend like this is the commitment of b

    assert!(proof.verify(&mut transcript, &c_a, &c_b).is_ok());
}
