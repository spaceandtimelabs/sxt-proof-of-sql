use thiserror::Error;

/// Errors in postprocessing
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PostprocessingError {
    /// Error in slicing due to slice index beyond usize
    #[error("Error in slicing due to slice index beyond usize {0}")]
    InvalidSliceIndex(i128),
    /// Column not found
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
