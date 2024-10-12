use super::{transcript_core::TranscriptCore, Keccak256Transcript as T, Transcript};
use crate::base::scalar::Curve25519Scalar as S;
use zerocopy::AsBytes;
#[test]
fn we_can_add_values_to_the_transcript_in_big_endian_form() {
    let mut transcript1: T = TranscriptCore::new();
    transcript1.extend_as_be([1u16, 1000, 2]);

    let mut transcript2: T = TranscriptCore::new();
    transcript2.raw_append(&[0, 1]);
    transcript2.raw_append(&[3, 232]);
    transcript2.raw_append(&[0, 2]);

    assert_eq!(transcript1.raw_challenge(), transcript2.raw_challenge());
}

#[test]
fn we_can_add_values_to_the_transcript_in_little_endian_form() {
    let mut transcript1: T = TranscriptCore::new();
    transcript1.extend_as_le_from_refs(&[1u16, 1000, 2]);

    let mut transcript2: T = TranscriptCore::new();
    transcript2.raw_append(&[1, 0]);
    transcript2.raw_append(&[232, 3]);
    transcript2.raw_append(&[2, 0]);

    assert_eq!(transcript1.raw_challenge(), transcript2.raw_challenge());
}

#[test]
fn we_can_add_scalars_to_the_transcript_in_big_endian_form() {
    let mut transcript1: T = TranscriptCore::new();
    transcript1.extend_scalars_as_be(&[S::from(1), S::from(1000), S::from(2)]);

    let mut transcript2: T = TranscriptCore::new();
    transcript2.raw_append(&[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);
    transcript2.raw_append(&[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        3, 232,
    ]);
    transcript2.raw_append(&[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2,
    ]);

    assert_eq!(transcript1.raw_challenge(), transcript2.raw_challenge());
}

#[test]
fn we_can_get_challenge_as_scalar_interpreted_as_big_endian() {
    let mut transcript1: T = TranscriptCore::new();
    let scalar: S = transcript1.scalar_challenge_as_be();

    let mut transcript2: T = TranscriptCore::new();
    let mut bytes = transcript2.raw_challenge();
    bytes.reverse();
    let mut limbs: [u64; 4] = scalar.into();
    limbs.as_bytes_mut().copy_from_slice(&bytes);

    assert_eq!(scalar, limbs.into());
}

#[test]
fn we_can_get_challenge_as_little_endian() {
    let mut transcript1: T = TranscriptCore::new();
    let mut transcript2: T = TranscriptCore::new();

    assert_eq!(transcript1.raw_challenge(), transcript2.challenge_as_le());
}
