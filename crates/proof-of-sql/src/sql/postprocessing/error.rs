use alloc::string::String;
use proof_of_sql_parser::Identifier;
use snafu::Snafu;

/// Errors in postprocessing
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum PostprocessingError {
    /// Error in slicing due to slice index beyond usize
    #[snafu(display("Error in slicing due to slice index beyond usize {index}"))]
    InvalidSliceIndex {
        /// The overflowing index value
        index: i128,
    },
    /// Column not found
    #[snafu(display("Column not found: {column}"))]
    ColumnNotFound {
        /// The column which is not found
        column: String,
    },
    /// Errors in evaluation of `Expression`s
    #[snafu(transparent)]
    ExpressionEvaluationError {
        /// The underlying source error
        source: crate::base::database::ExpressionEvaluationError,
    },
    /// Errors in constructing `OwnedTable`
    #[snafu(transparent)]
    OwnedTableError {
        /// The underlying source error
        source: crate::base::database::OwnedTableError,
    },
    /// GROUP BY clause references a column not in a group by expression outside aggregate functions
    #[snafu(display("Invalid group by: column '{column}' must not appear outside aggregate functions or `GROUP BY` clause."))]
    IdentifierNotInAggregationOperatorOrGroupByClause {
        /// The column identifier
        column: Identifier,
    },
    /// Errors in aggregate columns
    #[snafu(transparent)]
    AggregateColumnsError {
        /// The underlying source error
        source: crate::base::database::group_by_util::AggregateColumnsError,
    },
    /// Errors in `OwnedColumn`
    #[snafu(transparent)]
    OwnedColumnError {
        /// The underlying source error
        source: crate::base::database::OwnedColumnError,
    },
    /// Nested aggregation in `GROUP BY` clause
    #[snafu(display("Nested aggregation in `GROUP BY` clause: {error}"))]
    NestedAggregationInGroupByClause {
        /// The nested aggregation error
        error: String,
    },
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
