use alloc::vec::Vec;
/// Track the result created by a query
pub struct FirstRoundBuilder {
    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,
    /// The extra one evaluation lengths used in the proof.
    one_evaluation_lengths: Vec<usize>,
    // The range_length used in sumcheck which is max of all possible ones.
    range_length: usize,
}

impl FirstRoundBuilder {
    pub fn new(initial_range_length: usize) -> Self {
        Self {
            num_post_result_challenges: 0,
            one_evaluation_lengths: Vec::new(),
            range_length: initial_range_length,
        }
    }

    /// Get the range length used in the proof.
    pub(crate) fn range_length(&self) -> usize {
        self.range_length
    }

    /// Update the range length used in the proof only if the new range is larger than the existing range.
    pub(crate) fn update_range_length(&mut self, new_range_length: usize) {
        if new_range_length > self.range_length {
            self.range_length = new_range_length;
        }
    }

    /// Get the one evaluation lengths used in the proof.
    pub(crate) fn one_evaluation_lengths(&self) -> &[usize] {
        &self.one_evaluation_lengths
    }

    /// Append the length to the list of one evaluation lengths.
    pub(crate) fn produce_one_evaluation_length(&mut self, length: usize) {
        self.update_range_length(length);
        self.one_evaluation_lengths.push(length);
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
    pub fn request_post_result_challenges(&mut self, cnt: usize) {
        self.num_post_result_challenges += cnt;
    }
}
