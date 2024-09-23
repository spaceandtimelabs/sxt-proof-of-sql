use crate::base::{database::ColumnType, math::decimal::DecimalError};
use proof_of_sql_parser::intermediate_ast::{BinaryOperator, UnaryOperator};
use thiserror::Error;

/// Errors from operations on columns.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ColumnOperationError {
    /// Two columns do not have the same length
    #[error("Columns have different lengths: {len_a} != {len_b}")]
    DifferentColumnLength {
        /// The length of the first column
        len_a: usize,
        /// The length of the second column
        len_b: usize,
    },

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

    /// Incorrect `ColumnType` in unary operations
    #[error("{operator:?}(operand: {operand_type:?}) is not supported")]
    UnaryOperationInvalidColumnType {
        /// `UnaryOperator` that caused the error
        operator: UnaryOperator,
        /// `ColumnType` of the operand
        operand_type: ColumnType,
    },

    /// Overflow in integer operations
    #[error("Overflow in integer operation: {error}")]
    IntegerOverflow {
        /// The underlying overflow error
        error: String,
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Errors related to decimal operations
    #[error(transparent)]
    DecimalConversionError {
        /// The underlying source error
        #[from]
        source: DecimalError,
    },
}

/// Result type for column operations
pub type ColumnOperationResult<T> = std::result::Result<T, ColumnOperationError>;
