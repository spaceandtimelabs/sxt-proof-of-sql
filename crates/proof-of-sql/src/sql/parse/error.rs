use crate::base::{
    database::{ColumnOperationError, ColumnType},
    math::decimal::{DecimalError, IntermediateDecimalError},
};
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
};
use core::result::Result;
use proof_of_sql_parser::{posql_time::PoSQLTimestampError, ResourceId};
use snafu::Snafu;
use sqlparser::ast::Ident;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ConversionError {
    #[snafu(display("Column '{identifier}' was not found in table '{resource_id}'"))]
    /// The column is missing in the table
    MissingColumn {
        /// The missing column identifier
        identifier: Box<Ident>,
        /// The table resource id
        resource_id: Box<ResourceId>,
    },

    #[snafu(display("Column '{identifier}' was not found"))]
    /// The column is missing (without table information)
    MissingColumnWithoutTable {
        /// The missing column identifier
        identifier: Box<Ident>,
    },

    #[snafu(display("Expected '{expected}' but found '{actual}'"))]
    /// Invalid data type received
    InvalidDataType {
        /// Expected data type
        expected: ColumnType,
        /// Actual data type found
        actual: ColumnType,
    },

    #[snafu(display("Unsupported expression: {error}"))]
    /// The expression is unsupported
    UnsupportedExpr {
        /// The error for unsupported expression
        error: String,
    },

    #[snafu(display("Invalid precision value: {precision}"))]
    /// Precision value is invalid
    InvalidPrecision {
        /// The invalid precision value
        precision: String,
    },

    #[snafu(display("Invalid scale value: {scale}"))]
    /// Scale value is invalid
    InvalidScale {
        /// The invalid scale value
        scale: String,
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

    #[snafu(display("Unsupported operator: {message}"))]
    /// Unsupported operation
    UnsupportedOperation {
        /// The operator that is unsupported
        message: String,
    },
    /// Errors in converting `Ident` to `Identifier`
    #[snafu(display("Failed to convert `Ident` to `Identifier`: {error}"))]
    IdentifierConversionError {
        /// The underlying error message
        error: String,
    },

    #[snafu(display("Invalid number format: {value:?}"))]
    /// Represents an error due to an invalid number format.
    InvalidNumberFormat {
        /// The invalid number value as a string.
        value: String,
    },

    #[snafu(display(
        "Invalid decimal format: {value:?} with precision {precision} and scale {scale}"
    ))]
    /// Represents an error due to an invalid decimal format.
    InvalidDecimalFormat {
        /// The invalid decimal value as a string.
        value: String,
        /// The precision of the decimal value.
        precision: u8,
        /// The scale of the decimal value.
        scale: i8,
    },

    #[snafu(display("Unsupported literal type: {literal:?}"))]
    /// The literal type is not supported.
    UnsupportedLiteral {
        /// The unsupported literal type as a string.
        literal: String,
    },

    #[snafu(display("Unsupported data type: {data_type:?}"))]
    /// The data type is not supported.
    UnsupportedDataType {
        /// The unsupported data type as a string.
        data_type: String,
    },

    #[snafu(display("Invalid timestamp format: {value:?}"))]
    /// The timestamp format is invalid.
    InvalidTimestampFormat {
        /// The invalid timestamp value as a string.
        value: String,
    },

    #[snafu(display("Timestamp out of range: {value:?}"))]
    /// The timestamp value is out of the allowed range.
    TimestampOutOfRange {
        /// The out-of-range timestamp value as a string.
        value: String,
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

pub type ConversionResult<T> = Result<T, ConversionError>;
