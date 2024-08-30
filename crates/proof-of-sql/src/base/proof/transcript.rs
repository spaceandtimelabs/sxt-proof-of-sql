use crate::base::scalar::Scalar;
use zerocopy::{AsBytes, FromBytes};

/// A public-coin transcript.
///
/// This trait contains several method for adding prover messages and computing verifier challenges.
///
/// Implementation note: this is intended to be implemented via [super::transcript_core::TranscriptCore] rather than directly.
#[allow(dead_code)]
pub trait Transcript {
    /// Creates a new transcript
    fn new() -> Self;
    /// Appends the provided messages by appending the reversed raw bytes (i.e. assuming the message is bigendian)
    fn extend_as_be<M: FromBytes + AsBytes>(&mut self, messages: impl IntoIterator<Item = M>);
    /// Appends the provided messages by appending the raw bytes (i.e. assuming the message is littleendian)
    fn extend_as_le<'a, M: AsBytes + 'a>(&mut self, messages: impl IntoIterator<Item = &'a M>);
    /// Appends the provided scalars by appending the reversed raw bytes of the canonical value of the scalar (i.e. bigendian form)
    fn extend_scalars_as_be<'a, S: Scalar + 'a>(
        &mut self,
        messages: impl IntoIterator<Item = &'a S>,
    );
    /// Request a scalar challenge. Assumes that the reversed raw bytes are the canonical value of the scalar (i.e. bigendian form)
    fn scalar_challenge_as_be<S: Scalar>(&mut self) -> S;
    #[cfg(test)]
    /// Request a challenge. Returns the raw, unreversed, bytes. (i.e. littleendian form)
    fn challenge_as_le(&mut self) -> [u8; 32];
}
