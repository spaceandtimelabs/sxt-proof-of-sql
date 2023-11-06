use crate::base::{proof::transcript_protocol::*, scalar::ArkScalar};
use curve25519_dalek::ristretto::CompressedRistretto;
use merlin::Transcript;

#[test]
fn test_challenge_ark_scalars() {
    let zero = ArkScalar::from(0u64);
    let mut transcript = Transcript::new(b"multiplicationtest");
    let mut v = [zero; 3];
    transcript.challenge_ark_scalars(&mut v, MessageLabel::SumcheckChallenge);
    assert_ne!(v[0], zero);
    assert_ne!(v[1], zero);
    assert_ne!(v[2], zero);
    assert_ne!(v[0], v[1]);
    assert_ne!(v[1], v[2]);
}

#[test]
fn test_challenge_ark_group_elements() {
    let zero = ark_bls12_381::G1Affine::identity();
    let mut transcript = Transcript::new(b"multiplicationtest");
    let mut v = [zero; 3];
    transcript.challenge_ark(&mut v, MessageLabel::SumcheckChallenge);
    assert_ne!(v[0], zero);
    assert_ne!(v[1], zero);
    assert_ne!(v[2], zero);
    assert_ne!(v[0], v[1]);
    assert_ne!(v[1], v[2]);
}

#[test]
fn we_get_different_results_with_different_transcripts() {
    let zero = ArkScalar::from(0u64);
    let mut transcript = Transcript::new(b"same");
    let mut transcript2 = Transcript::new(b"different");
    let mut v = [zero; 3];
    let mut w = [zero; 3];
    transcript.challenge_ark_scalars(&mut v, MessageLabel::SumcheckChallenge);
    transcript2.challenge_ark_scalars(&mut w, MessageLabel::SumcheckChallenge);
    assert_ne!(v[0], w[0]);
    assert_ne!(v[1], w[1]);
    assert_ne!(v[2], w[2]);
}

#[test]
fn we_get_equivalent_results_with_equivalent_transcripts() {
    let zero = ArkScalar::from(0u64);
    let mut transcript = Transcript::new(b"same");
    let mut transcript2 = Transcript::new(b"same");
    let mut v = [zero; 3];
    let mut w = [zero; 3];
    transcript.challenge_ark_scalars(&mut v, MessageLabel::SumcheckChallenge);
    transcript2.challenge_ark_scalars(&mut w, MessageLabel::SumcheckChallenge);
    assert_eq!(v, w);
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
    let scalar1 = transcript1.challenge_ark_scalar(MessageLabel::InnerProductChallenge);

    let mut transcript2 = Transcript::new(b"ristrettotest");
    transcript2.append_points(MessageLabel::InnerProduct, &pts);
    let scalar2 = transcript2.challenge_ark_scalar(MessageLabel::InnerProductChallenge);

    assert_ne!(scalar1, scalar2);
}

#[test]
fn we_can_append_ark_group_elements() {
    let mut rng: ark_std::rand::rngs::StdRng = ark_std::rand::SeedableRng::from_seed([1; 32]);
    let pts: [ark_bls12_381::G1Affine; 2] = [
        ark_std::UniformRand::rand(&mut rng),
        ark_std::UniformRand::rand(&mut rng),
    ];
    let mut transcript1 = Transcript::new(b"arktest");
    transcript1.append_canonical_serialize(MessageLabel::InnerProduct, &pts[..1]);
    let scalar1 = transcript1.challenge_ark_scalar(MessageLabel::InnerProductChallenge);

    let mut transcript2 = Transcript::new(b"arktest");
    transcript2.append_canonical_serialize(MessageLabel::InnerProduct, &pts);
    let scalar2 = transcript2.challenge_ark_scalar(MessageLabel::InnerProductChallenge);

    assert_ne!(scalar1, scalar2);
}
