use crate::base::{database::ColumnType, math::decimal::DecimalError};
use proof_of_sql_parser::intermediate_ast::BinaryOperator;
use thiserror::Error;

/// Errors from operations on columns.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ColumnOperationError {
    /// Two columns do not have the same length
    #[error("Columns have different lengths: {0} != {1}")]
    DifferentColumnLength(usize, usize),

    /// Incorrect `ColumnType` in binary operations
    #[error("{operator:?}(lhs: {left_type:?}, rhs: {right_type:?}) is not supported")]
    BinaryOperationInvalidColumnType {
        /// `BinaryOperator` that caused the error
        operator: BinaryOperator,
        /// `ColumnType` of left operand
        left_type: ColumnType,
        /// `ColumnType` of right operand
        right_type: ColumnType,
    },

    /// Overflow in integer operations
    #[error("Overflow in integer operation: {0}")]
    IntegerOverflow(String),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Errors related to decimal operations
    #[error(transparent)]
    DecimalConversionError(#[from] DecimalError),
}

/// Result type for column operations
pub type ColumnOperationResult<T> = std::result::Result<T, ColumnOperationError>;
