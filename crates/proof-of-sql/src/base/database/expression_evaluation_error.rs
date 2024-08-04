use crate::base::{database::ColumnOperationError, math::decimal::DecimalError};
use thiserror::Error;

/// Errors from evaluation of `Expression`s.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ExpressionEvaluationError {
    /// Column not found
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
    /// Error in column operation
    #[error(transparent)]
    ColumnOperationError(#[from] ColumnOperationError),
    /// Expression not yet supported
    #[error("Expression {0} is not supported yet")]
    Unsupported(String),
    /// Error in decimal conversion
    #[error(transparent)]
    DecimalConversionError(#[from] DecimalError),
}

/// Result type for expression evaluation
pub type ExpressionEvaluationResult<T> = std::result::Result<T, ExpressionEvaluationError>;
