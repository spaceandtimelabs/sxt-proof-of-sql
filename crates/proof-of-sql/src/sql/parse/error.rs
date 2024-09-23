use crate::base::{
    database::{ColumnOperationError, ColumnType},
    math::decimal::DecimalError,
};
use proof_of_sql_parser::{
    intermediate_decimal::IntermediateDecimalError, posql_time::PoSQLTimestampError, Identifier,
    ResourceId,
};
use thiserror::Error;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConversionError {
    #[error("Column '{identifier}' was not found in table '{resource_id}'")]
    /// The column is missing in the table
    MissingColumn {
        /// The missing column identifier
        identifier: Box<Identifier>,
        /// The table resource id
        resource_id: Box<ResourceId>,
    },

    #[error("Column '{identifier}' was not found")]
    /// The column is missing (without table information)
    MissingColumnWithoutTable {
        /// The missing column identifier
        identifier: Box<Identifier>,
    },

    #[error("Expected '{expected}' but found '{actual}'")]
    /// Invalid data type received
    InvalidDataType {
        /// Expected data type
        expected: ColumnType,
        /// Actual data type found
        actual: ColumnType,
    },

    #[error("Left side has '{left_type}' type but right side has '{right_type}' type")]
    /// Data types do not match
    DataTypeMismatch {
        /// The left side datatype
        left_type: String,
        /// The right side datatype
        right_type: String,
    },

    #[error("Columns have different lengths: {len_a} != {len_b}")]
    /// Two columns do not have the same length
    DifferentColumnLength {
        /// The length of the first column
        len_a: usize,
        /// The length of the second column
        len_b: usize,
    },

    #[error("Multiple result columns with the same alias '{alias}' have been found.")]
    /// Duplicate alias in result columns
    DuplicateResultAlias {
        /// The duplicate alias
        alias: String,
    },

    #[error("A WHERE clause must has boolean type. It is currently of type '{datatype}'.")]
    /// WHERE clause is not boolean
    NonbooleanWhereClause {
        /// The actual datatype of the WHERE clause
        datatype: ColumnType,
    },

    #[error("Invalid order by: alias '{alias}' does not appear in the result expressions.")]
    /// ORDER BY clause references a non-existent alias
    InvalidOrderBy {
        /// The non-existent alias in the ORDER BY clause
        alias: String,
    },

    #[error("Invalid group by: column '{column}' must appear in the group by expression.")]
    /// GROUP BY clause references a non-existent column
    InvalidGroupByColumnRef {
        /// The non-existent column in the GROUP BY clause
        column: String,
    },

    #[error("Invalid expression: {expression}")]
    /// General error for invalid expressions
    InvalidExpression {
        /// The invalid expression error
        expression: String,
    },

    #[error("Encountered parsing error: {error}")]
    /// General parsing error
    ParseError {
        /// The underlying error
        error: String,
    },

    #[error(transparent)]
    /// Errors related to decimal operations
    DecimalConversionError {
        /// The underlying source error
        #[from]
        source: DecimalError,
    },

    /// Errors related to timestamp parsing
    #[error("Timestamp conversion error: {source}")]
    TimestampConversionError {
        /// The underlying source error
        #[from]
        source: PoSQLTimestampError,
    },

    /// Errors related to column operations
    #[error(transparent)]
    ColumnOperationError {
        /// The underlying source error
        #[from]
        source: ColumnOperationError,
    },

    /// Errors related to postprocessing
    #[error(transparent)]
    PostprocessingError {
        /// The underlying source error
        #[from]
        source: crate::sql::postprocessing::PostprocessingError,
    },

    #[error("Query not provable because: {error}")]
    /// Query requires unprovable feature
    Unprovable {
        /// The underlying error
        error: String,
    },
}

impl From<String> for ConversionError {
    fn from(value: String) -> Self {
        ConversionError::ParseError { error: value }
    }
}

impl From<ConversionError> for String {
    fn from(error: ConversionError) -> Self {
        error.to_string()
    }
}

impl From<IntermediateDecimalError> for ConversionError {
    fn from(err: IntermediateDecimalError) -> ConversionError {
        ConversionError::DecimalConversionError {
            source: DecimalError::IntermediateDecimalConversionError { source: err },
        }
    }
}

impl ConversionError {
    /// Returns a `ConversionError::InvalidExpression` for non-numeric types used in numeric aggregation functions.
    pub fn non_numeric_expr_in_agg<S: Into<String>>(dtype: S, func: S) -> Self {
        ConversionError::InvalidExpression {
            expression: format!(
                "cannot use expression of type '{}' with numeric aggregation function '{}'",
                dtype.into().to_lowercase(),
                func.into().to_lowercase()
            ),
        }
    }
}

pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
