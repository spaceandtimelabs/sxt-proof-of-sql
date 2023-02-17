use arrow::record_batch::RecordBatch;
use thiserror::Error;

/// Verifiable query errors
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Overflow error")]
    Overflow,
    #[error("String decode error")]
    InvalidString,
}

/// The result of a query -- either an error or a table.
///
/// We use Apache Arrow's RecordBatch to represent a table
/// result so as to allow for easy interoperability with
/// Apache Arrow Flight.
///
/// See `<https://voltrondata.com/blog/apache-arrow-flight-primer/>`
pub type QueryResult = Result<RecordBatch, QueryError>;
