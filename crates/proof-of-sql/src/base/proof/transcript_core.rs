use super::Transcript;
use crate::base::{ref_into::RefInto, scalar::Scalar};
use zerocopy::{AsBytes, FromBytes};

/// A trait used to facilitate implementation of [Transcript](super::Transcript).
///
/// There is a blanket `impl<T: TranscriptCore> Transcript for T` implementation.
pub(super) trait TranscriptCore {
    /// Creates a new transcript.
    fn new() -> Self;
    /// Appends a slice of bytes (as a message) to the transcript.
    fn raw_append(&mut self, message: &[u8]);
    /// Pulls a challenge from the transcript.
    fn raw_challenge(&mut self) -> [u8; 32];
}

/// private method to facilitate recieving challenges and reversing them. Undefined behavior if the `size_of` `M` is not 32 bytes.
fn receive_challenge_as_be<M: FromBytes>(slf: &mut impl TranscriptCore) -> M {
    debug_assert_eq!(32, core::mem::size_of::<M>());
    let mut bytes = slf.raw_challenge();
    bytes.reverse();
    M::read_from(&bytes).unwrap()
}

impl<T: TranscriptCore> Transcript for T {
    fn new() -> Self {
        TranscriptCore::new()
    }
    fn extend_as_be<M: FromBytes + AsBytes>(&mut self, messages: impl IntoIterator<Item = M>) {
        messages.into_iter().for_each(|mut message| {
            let bytes = message.as_bytes_mut();
            bytes.reverse();
            self.raw_append(bytes)
        })
    }
    fn extend_as_le_from_refs<'a, M: AsBytes + 'a + ?Sized>(
        &mut self,
        messages: impl IntoIterator<Item = &'a M>,
    ) {
        messages
            .into_iter()
            .for_each(|message| self.raw_append(message.as_bytes()))
    }
    fn extend_scalars_as_be<'a, S: Scalar + 'a>(
        &mut self,
        messages: impl IntoIterator<Item = &'a S>,
    ) {
        self.extend_as_be::<[u64; 4]>(messages.into_iter().map(RefInto::ref_into))
    }
    fn scalar_challenge_as_be<S: Scalar>(&mut self) -> S {
        receive_challenge_as_be::<[u64; 4]>(self).into()
    }
    fn challenge_as_le(&mut self) -> [u8; 32] {
        self.raw_challenge()
    }
}

#[cfg(test)]
pub(super) mod test_util {
    use super::TranscriptCore;
    pub fn we_get_equivalent_challenges_with_equivalent_transcripts<T: TranscriptCore>() {
        let mut transcript1: T = TranscriptCore::new();
        transcript1.raw_append(b"message");

        let mut transcript2: T = TranscriptCore::new();
        transcript2.raw_append(b"message");

        assert_eq!(
            transcript1.raw_challenge(),
            transcript2.raw_challenge(),
            "challenges do not match when transcripts are the same"
        );
    }
    pub fn we_get_different_challenges_with_different_transcripts<T: TranscriptCore>() {
        let mut transcript1: T = TranscriptCore::new();
        transcript1.raw_append(b"message1");

        let mut transcript2: T = TranscriptCore::new();
        transcript2.raw_append(b"message2");

        assert_ne!(
            transcript1.raw_challenge(),
            transcript2.raw_challenge(),
            "challenges match even though transcripts are different"
        );
    }
    pub fn we_get_different_nontrivial_consecutive_challenges_from_transcript<T: TranscriptCore>() {
        let mut transcript: T = TranscriptCore::new();
        let challenge1 = transcript.raw_challenge();
        let challenge2 = transcript.raw_challenge();

        assert_ne!(
            challenge1, [0; 32],
            "first challenge in transcript is trivial"
        );
        assert_ne!(
            challenge2, [0; 32],
            "second challenge in transcript is trivial"
        );
        assert_ne!(
            challenge1, challenge2,
            "consequtive challenges match even though transcripts are different"
        );
    }
}
