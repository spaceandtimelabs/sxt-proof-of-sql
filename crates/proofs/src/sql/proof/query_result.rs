use crate::base::proof::ProofError;
use arrow::record_batch::RecordBatch;
use thiserror::Error;

/// Verifiable query errors
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Overflow error")]
    Overflow,
    #[error("String decode error")]
    InvalidString,
    #[error(transparent)]
    ProofError(#[from] ProofError),
}

/// The verified results of a query along with metadata produced by verification
pub struct QueryData {
    /// We use Apache Arrow's RecordBatch to represent a table
    /// result so as to allow for easy interoperability with
    /// Apache Arrow Flight.
    ///
    /// See `<https://voltrondata.com/blog/apache-arrow-flight-primer/>`
    pub record_batch: RecordBatch,
    /// Additionally, there is a 32-byte verification hash that is included with this table.
    /// This hash provides evidence that the verification has been run.
    pub verification_hash: [u8; 32],
}

/// The result of a query -- either an error or a table.
pub type QueryResult = Result<QueryData, QueryError>;
