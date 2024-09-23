use proof_of_sql_parser::Identifier;
use thiserror::Error;

/// Errors in postprocessing
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PostprocessingError {
    /// Error in slicing due to slice index beyond usize
    #[error("Error in slicing due to slice index beyond usize {index}")]
    InvalidSliceIndex {
        /// The overflowing index value
        index: i128,
    },
    /// Column not found
    #[error("Column not found: {column}")]
    ColumnNotFound {
        /// The column which is not found
        column: String,
    },
    /// Errors in evaluation of `Expression`s
    #[error(transparent)]
    ExpressionEvaluationError {
        /// The underlying source error
        #[from]
        source: crate::base::database::ExpressionEvaluationError,
    },
    /// Errors in constructing `OwnedTable`
    #[error(transparent)]
    OwnedTableError {
        /// The underlying source error
        #[from]
        source: crate::base::database::OwnedTableError,
    },
    /// GROUP BY clause references a column not in a group by expression outside aggregate functions
    #[error("Invalid group by: column '{column}' must not appear outside aggregate functions or `GROUP BY` clause.")]
    IdentifierNotInAggregationOperatorOrGroupByClause {
        /// The column identifier
        column: Identifier,
    },
    /// Errors in aggregate columns
    #[error(transparent)]
    AggregateColumnsError {
        /// The underlying source error
        #[from]
        source: crate::base::database::group_by_util::AggregateColumnsError,
    },
    /// Errors in `OwnedColumn`
    #[error(transparent)]
    OwnedColumnError {
        /// The underlying source error
        #[from]
        source: crate::base::database::OwnedColumnError,
    },
    /// Nested aggregation in `GROUP BY` clause
    #[error("Nested aggregation in `GROUP BY` clause: {error}")]
    NestedAggregationInGroupByClause {
        /// The nested aggregation error
        error: String,
    },
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
