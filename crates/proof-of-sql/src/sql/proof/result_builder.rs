use super::{Indexes, ProvableQueryResult, ProvableResultColumn};
use crate::base::{database::Column, scalar::Scalar};

/// Track the result created by a query
pub struct ResultBuilder {
    table_length: usize,
    result_index_vector: Indexes,

    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,
}

impl ResultBuilder {
    /// Create a new result builder for a table with the given length. For multi table queries, this will likely need to change.
    pub fn new(table_length: usize) -> Self {
        Self {
            table_length,
            result_index_vector: Indexes::default(),
            num_post_result_challenges: 0,
        }
    }

    /// Get the length of the table
    pub fn table_length(&self) -> usize {
        self.table_length
    }

    /// Set the indexes of the rows select in the result
    pub fn set_result_indexes(&mut self, result_index: Indexes) {
        self.result_index_vector = result_index;
    }

    /// Construct the intermediate query result to be sent to the verifier.
    pub fn make_provable_query_result<S: Scalar>(
        &self,
        cols: &[Column<'_, S>],
    ) -> ProvableQueryResult {
        let boxed: Vec<Box<dyn ProvableResultColumn>> = cols
            .iter()
            .map(|c| -> Box<dyn ProvableResultColumn> {
                Box::new(*c) as Box<dyn ProvableResultColumn>
            })
            .collect::<Vec<_>>();
        ProvableQueryResult::new(&self.result_index_vector, &boxed)
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
    /// Note: this must be matched with the same count in the CountBuilder.
    pub fn request_post_result_challenges(&mut self, cnt: usize) {
        self.num_post_result_challenges += cnt;
    }
}
