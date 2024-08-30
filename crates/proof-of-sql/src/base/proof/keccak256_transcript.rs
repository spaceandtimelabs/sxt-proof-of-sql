use super::{transcript_core::TranscriptCore, Transcript};
use core::mem::replace;
use tiny_keccak::{Hasher, Keccak};

#[allow(dead_code)]
/// Public coin transcript that is easily portable to Solidity.
///
/// Leverages the keccak256 hash function, which has the lowest gas costs on Solidity.
///
/// The public coin transcript consists of alternating prover messages and verifier challenges.
/// In order to multiple verifier challenges in a row, an empty prover message must be sent.
/// In order to send multiple prover messages in a row, the verifier challenge can be discarded.
///
/// The challenges/state are computed as follows:
/// ```pseudo-code
/// challenge_(i+1) = keccak256(challenge_i, message_(i+1))
/// ```
pub struct Keccak256Transcript(Keccak);
impl TranscriptCore for Keccak256Transcript {
    fn new() -> Self {
        Self(Keccak::v256())
    }
    fn raw_append(&mut self, message: &[u8]) {
        self.0.update(message)
    }
    fn raw_challenge(&mut self) -> [u8; 32] {
        let mut result = [0; 32];

        // Replace existing Hasher with a new one, and finalize the old Hasher,
        // getting a hash/the desired challenge:
        replace(self, Transcript::new()).0.finalize(&mut result);

        // Add this challenge to the new Hasher for the next round of messages:
        self.raw_append(&result);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::{super::transcript_core::test_util::*, Keccak256Transcript};
    #[test]
    fn we_get_equivalent_challenges_with_equivalent_keccak256_transcripts() {
        we_get_equivalent_challenges_with_equivalent_transcripts::<Keccak256Transcript>()
    }
    #[test]
    fn we_get_different_challenges_with_different_keccak256_transcripts() {
        we_get_different_challenges_with_different_transcripts::<Keccak256Transcript>()
    }
    #[test]
    fn we_get_different_nontrivial_consecutive_challenges_from_keccak256_transcript() {
        we_get_different_nontrivial_consecutive_challenges_from_transcript::<Keccak256Transcript>()
    }
}
