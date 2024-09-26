use crate::base::{database::ColumnOperationError, math::decimal::DecimalError};
use alloc::string::String;
use core::result::Result;
use snafu::Snafu;

/// Errors from evaluation of `Expression`s.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ExpressionEvaluationError {
    /// Column not found
    #[snafu(display("Column not found: {error}"))]
    ColumnNotFound {
        /// The underlying error
        error: String,
    },
    /// Error in column operation
    #[snafu(transparent)]
    ColumnOperationError {
        /// The underlying source error
        source: ColumnOperationError,
    },
    /// Expression not yet supported
    #[snafu(display("Expression {expression} is not supported yet"))]
    Unsupported {
        /// The unsupported expression
        expression: String,
    },
    /// Error in decimal conversion
    #[snafu(transparent)]
    DecimalConversionError {
        /// The underlying source error
        source: DecimalError,
    },
}

/// Result type for expression evaluation
pub type ExpressionEvaluationResult<T> = Result<T, ExpressionEvaluationError>;
