use crate::base::scalar::Scalar;
use alloc::vec::Vec;
use zerocopy::{AsBytes, FromBytes};

/// A public-coin transcript.
///
/// This trait contains several method for adding prover messages and computing verifier challenges.
///
/// Implementation note: this is intended to be implemented via [`super::transcript_core::TranscriptCore`] rather than directly.
#[allow(dead_code)]
pub trait Transcript {
    /// Creates a new transcript
    fn new() -> Self;
    /// Appends the provided messages by appending the reversed raw bytes (i.e. assuming the message is bigendian)
    fn extend_as_be<M: FromBytes + AsBytes>(&mut self, messages: impl IntoIterator<Item = M>);
    /// Appends the provided messages by appending the reversed raw bytes (i.e. assuming the message is bigendian)
    fn extend_as_be_from_refs<'a, M: FromBytes + AsBytes + 'a + Copy>(
        &mut self,
        messages: impl IntoIterator<Item = &'a M>,
    ) {
        self.extend_as_be(messages.into_iter().copied());
    }
    /// Appends the provided messages by appending the raw bytes (i.e. assuming the message is littleendian)
    fn extend_as_le<M: AsBytes>(&mut self, messages: impl IntoIterator<Item = M>);
    /// Appends the provided messages by appending the raw bytes (i.e. assuming the message is littleendian)
    fn extend_as_le_from_refs<'a, M: AsBytes + 'a + ?Sized>(
        &mut self,
        messages: impl IntoIterator<Item = &'a M>,
    );
    /// Appends the provided scalars by appending the reversed raw bytes of the canonical value of the scalar (i.e. bigendian form)
    fn extend_scalars_as_be<'a, S: Scalar + 'a>(
        &mut self,
        messages: impl IntoIterator<Item = &'a S>,
    );
    /// Request a scalar challenge. Assumes that the reversed raw bytes are the canonical value of the scalar (i.e. bigendian form)
    fn scalar_challenge_as_be<S: Scalar>(&mut self) -> S;
    /// Request a challenge. Returns the raw, unreversed, bytes. (i.e. littleendian form)
    fn challenge_as_le(&mut self) -> [u8; 32];

    /// Appends a type that implements [`serde::Serialize`] by appending the raw bytes (i.e. assuming the message is littleendian)
    ///
    /// # Panics
    /// - Panics if `postcard::to_allocvec(message)` fails to serialize the message.
    fn extend_serialize_as_le(&mut self, message: &(impl serde::Serialize + ?Sized)) {
        self.extend_as_le_from_refs([postcard::to_allocvec(message).unwrap().as_slice()]);
    }
    /// Appends a type that implements [`ark_serialize::CanonicalSerialize`] by appending the raw bytes (i.e. assuming the message is littleendian)
    ///
    /// # Panics
    /// - Panics if `message.serialize_compressed(&mut buf)` fails to serialize the message.
    fn extend_canonical_serialize_as_le(
        &mut self,
        message: &(impl ark_serialize::CanonicalSerialize + ?Sized),
    ) {
        let mut buf = Vec::with_capacity(message.compressed_size());
        message.serialize_compressed(&mut buf).unwrap();
        self.extend_as_le_from_refs([buf.as_slice()]);
    }
    /// "Lift" a function so that it can be applied to an `impl Transcript` of (possibly) different type than self.
    /// This allows for interopability between transcript types.
    fn wrap_transcript<T: Transcript, R>(&mut self, op: impl FnOnce(&mut T) -> R) -> R {
        let mut transcript = T::new();
        transcript.extend_as_le([self.challenge_as_le()]);
        let result = op(&mut transcript);
        self.extend_as_le([transcript.challenge_as_le()]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::Transcript;
    use crate::base::proof::Keccak256Transcript;
    use alloc::{string::ToString, vec};

    #[test]
    fn we_can_extend_transcript_with_serialize() {
        let mut transcript1: Keccak256Transcript = Transcript::new();
        let mut transcript2: Keccak256Transcript = Transcript::new();

        transcript1.extend_serialize_as_le(&(123, vec!["hi", "there"]));
        transcript2.extend_serialize_as_le(&(123, vec!["hi", "there"]));

        assert_eq!(transcript1.challenge_as_le(), transcript2.challenge_as_le());

        transcript2.extend_serialize_as_le(&234.567);

        assert_ne!(transcript1.challenge_as_le(), transcript2.challenge_as_le());
    }

    #[test]
    fn we_can_extend_transcript_with_canonical_serialize() {
        let mut transcript1: Keccak256Transcript = Transcript::new();
        let mut transcript2: Keccak256Transcript = Transcript::new();

        transcript1.extend_canonical_serialize_as_le(&(
            123_u16,
            vec!["hi".to_string(), "there".to_string()],
        ));
        transcript2.extend_canonical_serialize_as_le(&(
            123_u16,
            vec!["hi".to_string(), "there".to_string()],
        ));

        assert_eq!(transcript1.challenge_as_le(), transcript2.challenge_as_le());

        transcript2.extend_canonical_serialize_as_le(&ark_bls12_381::FQ_ONE);

        assert_ne!(transcript1.challenge_as_le(), transcript2.challenge_as_le());
    }

    #[test]
    fn we_can_extend_transcript_with_wrapped_transcript() {
        let mut transcript1: Keccak256Transcript = Transcript::new();
        let mut transcript2: Keccak256Transcript = Transcript::new();

        let result1 = transcript1.wrap_transcript(|transcript: &mut merlin::Transcript| {
            transcript.append_u64(b"test", 320);
            let mut result = vec![0; 3];
            transcript.challenge_bytes(b"test2", &mut result);
            result
        });
        let result2 = transcript2.wrap_transcript(|transcript: &mut merlin::Transcript| {
            transcript.append_u64(b"test", 320);
            let mut result = vec![0; 3];
            transcript.challenge_bytes(b"test2", &mut result);
            result
        });

        assert_eq!(result1, result2);
        assert_eq!(transcript1.challenge_as_le(), transcript2.challenge_as_le());

        transcript1.wrap_transcript(|transcript: &mut merlin::Transcript| {
            let mut result = vec![0; 32];
            transcript.challenge_bytes(b"test3", &mut result);
            result
        });

        assert_ne!(transcript1.challenge_as_le(), transcript2.challenge_as_le());
    }

    #[test]
    fn we_can_extend_transcript_with_extend_as_be_from_refs() {
        let mut transcript1: Keccak256Transcript = Transcript::new();
        let mut transcript2: Keccak256Transcript = Transcript::new();

        let messages: Vec<u32> = vec![1, 2, 3, 4];
        transcript1.extend_as_be_from_refs(&messages);
        transcript2.extend_as_be_from_refs(&messages);

        assert_eq!(transcript1.challenge_as_le(), transcript2.challenge_as_le());

        let more_messages: Vec<u32> = vec![5, 6, 7, 8];
        transcript2.extend_as_be_from_refs(&more_messages);

        assert_ne!(transcript1.challenge_as_le(), transcript2.challenge_as_le());
    }
}
