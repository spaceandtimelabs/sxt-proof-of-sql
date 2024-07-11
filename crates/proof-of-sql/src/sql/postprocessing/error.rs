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
    /// Errors related to decimal operations
    #[error(transparent)]
    DecimalConversionError(#[from] crate::base::math::decimal::DecimalError),
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
