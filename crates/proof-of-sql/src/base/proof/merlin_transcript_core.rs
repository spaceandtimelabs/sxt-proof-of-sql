impl super::transcript_core::TranscriptCore for merlin::Transcript {
    fn new() -> Self {
        merlin::Transcript::new(b"TranscriptCore::new")
    }
    fn raw_append(&mut self, message: &[u8]) {
        self.append_message(b"TranscriptCore::raw_append", message)
    }
    fn raw_challenge(&mut self) -> [u8; 32] {
        let mut result = [0u8; 32];
        self.challenge_bytes(b"TranscriptCore::raw_challenge", &mut result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::transcript_core::test_util::*;
    #[test]
    fn we_get_equivalent_challenges_with_equivalent_merlin_transcripts() {
        we_get_equivalent_challenges_with_equivalent_transcripts::<merlin::Transcript>()
    }
    #[test]
    fn we_get_different_challenges_with_different_keccak256_transcripts() {
        we_get_different_challenges_with_different_transcripts::<merlin::Transcript>()
    }
    #[test]
    fn we_get_different_nontrivial_consecutive_challenges_from_keccak256_transcript() {
        we_get_different_nontrivial_consecutive_challenges_from_transcript::<merlin::Transcript>()
    }
}
