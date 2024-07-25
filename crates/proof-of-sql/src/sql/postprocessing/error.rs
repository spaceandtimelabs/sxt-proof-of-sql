use crate::base::database::ColumnType;
use proof_of_sql_parser::intermediate_ast::{BinaryOperator, UnaryOperator};
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
    /// Data Type mismatch in scalar / unary operations
    #[error("Data Type mismatch")]
    UnaryOperationInvalidColumnType {
        /// Unary operator
        operator: UnaryOperator,
        /// ColumnType of the operand
        operand_type: ColumnType,
    },
    /// Data Type mismatch in binary operations
    #[error("Data Type mismatch")]
    BinaryOperationInvalidColumnType {
        /// Binary operator
        operator: BinaryOperator,
        /// ColumnType of left operand
        left_type: ColumnType,
        /// ColumnType of right operand
        right_type: ColumnType,
    },
    /// Errors caused by division by zero
    #[error("Division by zero")]
    DivisionByZero,
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
