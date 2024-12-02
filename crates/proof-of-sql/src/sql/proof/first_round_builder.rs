/// Track the result created by a query
pub struct FirstRoundBuilder {
    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,
}

impl Default for FirstRoundBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FirstRoundBuilder {
    pub fn new() -> Self {
        Self {
            num_post_result_challenges: 0,
        }
    }

    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    pub(super) fn num_post_result_challenges(&self) -> usize {
        self.num_post_result_challenges
    }

    /// Request `cnt` more post result challenges.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Note: this must be matched with the same count in the [`CountBuilder`](crate::sql::proof::CountBuilder).
    pub fn request_post_result_challenges(&mut self, cnt: usize) {
        self.num_post_result_challenges += cnt;
    }
}
