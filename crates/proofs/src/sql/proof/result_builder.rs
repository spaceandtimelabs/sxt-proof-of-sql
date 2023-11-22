use super::{Indexes, ProvableQueryResult, ProvableResultColumn};

/// Track the result created by a query
pub struct ResultBuilder<'a> {
    table_length: usize,
    result_index_vector: Indexes,
    result_columns: Vec<Box<dyn ProvableResultColumn + 'a>>,

    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,
}

impl<'a> ResultBuilder<'a> {
    /// Create a new result builder for a table with the given length. For multi table queries, this will likely need to change.
    pub fn new(table_length: usize) -> Self {
        Self {
            table_length,
            result_index_vector: Indexes::default(),
            result_columns: Vec::new(),
            num_post_result_challenges: 0,
        }
    }

    /// Get the length of the table
    pub fn table_length(&self) -> usize {
        self.table_length
    }

    /// Set the indexes of the rows select in the result
    #[tracing::instrument(
        name = "proofs.sql.proof.result_builder.set_result_indexes",
        level = "debug",
        skip_all
    )]
    pub fn set_result_indexes(&mut self, result_index: Indexes) {
        self.result_index_vector = result_index;
    }

    /// Produce an intermediate result column that will be sent to the verifier.
    #[tracing::instrument(
        name = "proofs.sql.proof.result_builder.produce_result_column",
        level = "debug",
        skip_all
    )]
    pub fn produce_result_column(&mut self, col: Box<dyn ProvableResultColumn + 'a>) {
        self.result_columns.push(col);
    }

    /// Construct the intermediate query result to be sent to the verifier.
    #[tracing::instrument(
        name = "proofs.sql.proof.result_builder.make_provable_query_result",
        level = "debug",
        skip_all
    )]
    pub fn make_provable_query_result(&self) -> ProvableQueryResult {
        ProvableQueryResult::new(&self.result_index_vector, &self.result_columns)
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
