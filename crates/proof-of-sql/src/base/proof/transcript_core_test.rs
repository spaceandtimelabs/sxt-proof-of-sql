use super::{Keccak256Transcript as T, Transcript, transcript_core::TranscriptCore};
use crate::base::scalar::{Scalar, ScalarExt, test_scalar::TestScalar as S};
use bnum::types::U256;
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
    transcript1.extend_as_le([1u16, 1000, 2]);

    let mut transcript2: T = TranscriptCore::new();
    transcript2.raw_append(&[1, 0]);
    transcript2.raw_append(&[232, 3]);
    transcript2.raw_append(&[2, 0]);

    assert_eq!(transcript1.raw_challenge(), transcript2.raw_challenge());
}

#[test]
fn we_can_add_values_to_the_transcript_in_little_endian_form_from_refs() {
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
    let mut transcript2: T = TranscriptCore::new();

    assert_eq!(
        transcript1.scalar_challenge_as_be::<S>(),
        S::from_wrapping(
            U256::from_be_slice(&transcript2.raw_challenge()).unwrap() & S::CHALLENGE_MASK
        )
    );
}

#[test]
fn we_can_get_challenge_as_little_endian() {
    let mut transcript1: T = TranscriptCore::new();
    let mut transcript2: T = TranscriptCore::new();

    assert_eq!(transcript1.raw_challenge(), transcript2.challenge_as_le());
}
