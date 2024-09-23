use crate::base::{
    database::{OwnedTable, OwnedTableError},
    proof::ProofError,
    scalar::Scalar,
};
#[cfg(feature = "arrow")]
use arrow::{error::ArrowError, record_batch::RecordBatch};
use thiserror::Error;

/// Verifiable query errors
#[derive(Error, Debug)]
pub enum QueryError {
    /// The query result overflowed. This does not mean that the verification failed.
    /// This just means that the database was supposed to respond with a result that was too large.
    #[error("Overflow error")]
    Overflow,
    /// The query result string could not be decoded. This does not mean that the verification failed.
    /// This just means that the database was supposed to respond with a string that was not valid UTF-8.
    #[error("String decode error")]
    InvalidString,
    /// Decoding errors other than overflow and invalid string.
    #[error("Miscellaneous decoding error")]
    MiscellaneousDecodingError,
    /// Indexes are invalid.
    #[error("Invalid indexes")]
    InvalidIndexes,
    /// Miscellaneous evaluation error.
    #[error("Miscellaneous evaluation error")]
    MiscellaneousEvaluationError,
    /// The proof failed to verify.
    #[error(transparent)]
    ProofError {
        /// The underlying source error
        #[from]
        source: ProofError,
    },
    /// The table data was invalid. This should never happen because this should get caught by the verifier before reaching this point.
    #[error(transparent)]
    InvalidTable {
        /// The underlying source error
        #[from]
        source: OwnedTableError,
    },
}

/// The verified results of a query along with metadata produced by verification
pub struct QueryData<S: Scalar> {
    /// We use Apache Arrow's RecordBatch to represent a table
    /// result so as to allow for easy interoperability with
    /// Apache Arrow Flight.
    ///
    /// See `<https://voltrondata.com/blog/apache-arrow-flight-primer/>`
    pub table: OwnedTable<S>,
    /// Additionally, there is a 32-byte verification hash that is included with this table.
    /// This hash provides evidence that the verification has been run.
    pub verification_hash: [u8; 32],
}

impl<S: Scalar> QueryData<S> {
    #[cfg(all(test, feature = "arrow"))]
    pub fn into_record_batch(self) -> RecordBatch {
        self.try_into().unwrap()
    }
}

#[cfg(feature = "arrow")]
impl<S: Scalar> TryFrom<QueryData<S>> for RecordBatch {
    type Error = ArrowError;

    fn try_from(value: QueryData<S>) -> Result<Self, Self::Error> {
        Self::try_from(value.table)
    }
}

/// The result of a query -- either an error or a table.
pub type QueryResult<S> = Result<QueryData<S>, QueryError>;
