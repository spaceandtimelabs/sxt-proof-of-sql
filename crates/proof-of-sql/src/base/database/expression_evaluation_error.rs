use crate::base::{database::ColumnOperationError, math::decimal::DecimalError};
use thiserror::Error;

/// Errors from evaluation of `Expression`s.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ExpressionEvaluationError {
    /// Column not found
    #[error("Column not found: {error}")]
    ColumnNotFound {
        /// The underlying error
        error: String,
    },
    /// Error in column operation

    #[error(transparent)]
    ColumnOperationError {
        /// The underlying source error
        #[from]
        source: ColumnOperationError,
    },
    /// Expression not yet supported

    #[error("Expression {expression} is not supported yet")]
    Unsupported {
        /// The unsupported expression
        expression: String,
    },
    /// Error in decimal conversion

    #[error(transparent)]
    DecimalConversionError {
        /// The underlying source error
        #[from]
        source: DecimalError,
    },
}

/// Result type for expression evaluation
pub type ExpressionEvaluationResult<T> = std::result::Result<T, ExpressionEvaluationError>;
