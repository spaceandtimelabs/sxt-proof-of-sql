use crate::base::proof::transcript_protocol::*;

use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use merlin::Transcript;

#[test]
fn test_challenge_scalars() {
    let zero = Scalar::from(0u64);
    let mut transcript = Transcript::new(b"multiplicationtest");
    let mut v: [Scalar; 3] = [zero; 3];
    transcript.challenge_scalars(&mut v, MessageLabel::SumcheckChallenge);
    assert_ne!(v[0], zero);
    assert_ne!(v[1], zero);
    assert_ne!(v[2], zero);
    assert_ne!(v[0], v[1]);
    assert_ne!(v[1], v[2]);
}

#[test]
fn we_can_append_ristretto_points() {
    let mut bytes1 = [0u8; 32];
    bytes1[0] = 1u8;
    let mut bytes2 = [0u8; 32];
    bytes2[0] = 2u8;
    let pts = [CompressedRistretto(bytes1), CompressedRistretto(bytes2)];
    let mut transcript1 = Transcript::new(b"ristrettotest");
    transcript1.append_points(MessageLabel::InnerProduct, &pts[..1]);
    let scalar1 = transcript1.challenge_scalar(MessageLabel::InnerProductChallenge);

    let mut transcript2 = Transcript::new(b"ristrettotest");
    transcript2.append_points(MessageLabel::InnerProduct, &pts);
    let scalar2 = transcript2.challenge_scalar(MessageLabel::InnerProductChallenge);

    assert_ne!(scalar1, scalar2);
}
