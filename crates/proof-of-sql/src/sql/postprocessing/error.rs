use crate::base::database::ColumnType;
use proof_of_sql_parser::{
    intermediate_ast::{BinaryOperator, UnaryOperator},
    Identifier,
};
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
    /// Errors in evaluation of `Expression`s
    #[error(transparent)]
    ExpressionEvaluationError(#[from] crate::base::database::ExpressionEvaluationError),
    /// Errors in constructing `OwnedTable`
    #[error(transparent)]
    OwnedTableError(#[from] crate::base::database::OwnedTableError),
    #[error("Invalid group by: column '{0}' must not appear outside aggregate functions or `GROUP BY` clause.")]
    /// GROUP BY clause references a column not in a group by expression outside aggregate functions
    IdentifierNotInAggregateFunctionOrGroupByClause(Identifier),
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
