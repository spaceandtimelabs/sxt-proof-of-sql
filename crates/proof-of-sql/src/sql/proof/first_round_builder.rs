/// Track the result created by a query
pub struct FirstRoundBuilder {
    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,

    /// Used to determine the indices of generators we use
    range_length: usize,
}

impl FirstRoundBuilder {
    pub fn new(range_length: usize) -> Self {
        Self {
            num_post_result_challenges: 0,
            range_length,
        }
    }

    pub fn range_length(&self) -> usize {
        self.range_length
    }

    /// Used if a `ProofPlan` can cause output `table_length` to be larger
    /// than the largest of the input ones e.g. unions and joins since it will
    /// force us to update `range_length`.
    pub fn update_range_length(&mut self, table_length: usize) {
        if table_length > self.range_length {
            self.range_length = table_length;
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
