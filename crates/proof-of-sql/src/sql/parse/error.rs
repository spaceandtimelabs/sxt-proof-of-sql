use crate::base::{
    database::{ColumnOperationError, ColumnType},
    math::{DecimalError, InvalidPrecisionError},
};
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
};
use bigdecimal::ParseBigDecimalError;
use core::result::Result;
use proof_of_sql_parser::{posql_time::PoSQLTimestampError, Identifier, ResourceId};
use snafu::Snafu;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ConversionError {
    #[snafu(display("Column '{identifier}' was not found in table '{resource_id}'"))]
    /// The column is missing in the table
    MissingColumn {
        /// The missing column identifier
        identifier: Box<Identifier>,
        /// The table resource id
        resource_id: Box<ResourceId>,
    },

    #[snafu(display("Column '{identifier}' was not found"))]
    /// The column is missing (without table information)
    MissingColumnWithoutTable {
        /// The missing column identifier
        identifier: Box<Identifier>,
    },

    #[snafu(display("Expected '{expected}' but found '{actual}'"))]
    /// Invalid data type received
    InvalidDataType {
        /// Expected data type
        expected: ColumnType,
        /// Actual data type found
        actual: ColumnType,
    },

    #[snafu(display("Left side has '{left_type}' type but right side has '{right_type}' type"))]
    /// Data types do not match
    DataTypeMismatch {
        /// The left side datatype
        left_type: String,
        /// The right side datatype
        right_type: String,
    },

    #[snafu(display("Columns have different lengths: {len_a} != {len_b}"))]
    /// Two columns do not have the same length
    DifferentColumnLength {
        /// The length of the first column
        len_a: usize,
        /// The length of the second column
        len_b: usize,
    },

    #[snafu(display("Multiple result columns with the same alias '{alias}' have been found."))]
    /// Duplicate alias in result columns
    DuplicateResultAlias {
        /// The duplicate alias
        alias: String,
    },

    #[snafu(display(
        "A WHERE clause must has boolean type. It is currently of type '{datatype}'."
    ))]
    /// WHERE clause is not boolean
    NonbooleanWhereClause {
        /// The actual datatype of the WHERE clause
        datatype: ColumnType,
    },

    #[snafu(display(
        "Invalid order by: alias '{alias}' does not appear in the result expressions."
    ))]
    /// ORDER BY clause references a non-existent alias
    InvalidOrderBy {
        /// The non-existent alias in the ORDER BY clause
        alias: String,
    },

    #[snafu(display(
        "Invalid group by: column '{column}' must appear in the group by expression."
    ))]
    /// GROUP BY clause references a non-existent column
    InvalidGroupByColumnRef {
        /// The non-existent column in the GROUP BY clause
        column: String,
    },

    #[snafu(display("Invalid expression: {expression}"))]
    /// General error for invalid expressions
    InvalidExpression {
        /// The invalid expression error
        expression: String,
    },

    #[snafu(display("Encountered parsing error: {error}"))]
    /// General parsing error
    ParseError {
        /// The underlying error
        error: String,
    },

    #[snafu(transparent)]
    /// Errors related to decimal operations
    DecimalConversionError {
        /// The underlying source error
        source: DecimalError,
    },

    /// Errors related to timestamp parsing
    #[snafu(context(false), display("Timestamp conversion error: {source}"))]
    TimestampConversionError {
        /// The underlying source error
        source: PoSQLTimestampError,
    },

    /// Errors related to column operations
    #[snafu(transparent)]
    ColumnOperationError {
        /// The underlying source error
        source: ColumnOperationError,
    },

    /// Errors related to postprocessing
    #[snafu(transparent)]
    PostprocessingError {
        /// The underlying source error
        source: crate::sql::postprocessing::PostprocessingError,
    },

    #[snafu(display("Query not provable because: {error}"))]
    /// Query requires unprovable feature
    Unprovable {
        /// The underlying error
        error: String,
    },
}

impl From<InvalidPrecisionError> for ConversionError {
    fn from(value: InvalidPrecisionError) -> Self {
        ConversionError::from(Into::<DecimalError>::into(value))
    }
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

impl From<ParseBigDecimalError> for ConversionError {
    fn from(err: ParseBigDecimalError) -> ConversionError {
        ConversionError::DecimalConversionError {
            source: DecimalError::ParseError { error: err },
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

pub type ConversionResult<T> = Result<T, ConversionError>;
