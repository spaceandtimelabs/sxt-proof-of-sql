/// Track the result created by a query
pub struct ResultBuilder {
    result_table_length: usize,

    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,
}

impl ResultBuilder {
    /// Create a new result builder for a table with the given length. For multi table queries, this will likely need to change.
    pub fn new() -> Self {
        Self {
            result_table_length: 0,
            num_post_result_challenges: 0,
        }
    }

    /// Get the length of the output table
    pub fn result_table_length(&self) -> usize {
        self.result_table_length
    }

    /// Set the length of the output table
    pub fn set_result_table_length(&mut self, result_table_length: usize) {
        self.result_table_length = result_table_length;
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
