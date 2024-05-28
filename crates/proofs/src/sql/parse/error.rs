use crate::base::database::ColumnType;
use proofs_sql::{intermediate_decimal::DecimalError, Identifier, ResourceId};
use thiserror::Error;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConversionError {
    #[error("Column '{0}' was not found in table '{1}'")]
    /// The column is missing in the table
    MissingColumn(Box<Identifier>, Box<ResourceId>),

    #[error("Column '{0}' was not found")]
    /// The column is missing (without table information)
    MissingColumnWithoutTable(Box<Identifier>),

    #[error("Expected '{expected}' but found '{actual}'")]
    /// Invalid data type received
    InvalidDataType {
        /// Expected data type
        expected: ColumnType,
        /// Actual data type found
        actual: ColumnType,
    },

    #[error("Left side has '{1}' type but right side has '{0}' type")]
    /// Data types do not match
    DataTypeMismatch(String, String),

    #[error("Columns have different lengths: {0} != {1}")]
    /// Two columns do not have the same length
    DifferentColumnLength(usize, usize),

    #[error("Multiple result columns with the same alias '{0}' have been found.")]
    /// Duplicate alias in result columns
    DuplicateResultAlias(String),

    #[error("A WHERE clause must has boolean type. It is currently of type '{0}'.")]
    /// WHERE clause is not boolean
    NonbooleanWhereClause(ColumnType),

    #[error("Invalid order by: alias '{0}' does not appear in the result expressions.")]
    /// ORDER BY clause references a non-existent alias
    InvalidOrderBy(String),

    #[error("Invalid group by: column '{0}' must appear in the group by expression.")]
    /// GROUP BY clause references a non-existent column
    InvalidGroupByColumnRef(String),

    #[error("Invalid expression: {0}")]
    /// General error for invalid expressions
    InvalidExpression(String),

    #[error("Unsupported operation: cannot round decimal: {0}")]
    /// Decimal rounding is not supported
    DecimalRoundingError(String),

    #[error("Error while parsing precision from query: {0}")]
    /// Error in parsing precision in a query
    PrecisionParseError(String),

    #[error("Decimal precision is not valid: {0}")]
    /// Decimal precision exceeds the allowed limit
    InvalidPrecision(u8),

    #[error("Encountered parsing error: {0}")]
    /// General parsing error
    ParseError(String),

    #[error("Unsupported operation: cannot round literal: {0}")]
    /// Error when a rounding operation is not supported
    LiteralRoundDownError(String),

    #[error("Query not provable because: {0}")]
    /// Query requires unprovable feature
    Unprovable(String),

    #[error("Invalid decimal format or value: {0}")]
    /// Error when a decimal format or value is incorrect
    InvalidDecimal(String),
}

impl From<DecimalError> for ConversionError {
    fn from(error: DecimalError) -> Self {
        match error {
            DecimalError::ParseError(e) => ConversionError::ParseError(e.to_string()),
            DecimalError::OutOfRange => ConversionError::ParseError(
                "Intermediate decimal cannot be cast to primitive".into(),
            ),
            DecimalError::LossyCast => ConversionError::ParseError(
                "Intermediate decimal has non-zero fractional part".into(),
            ),
            DecimalError::ConversionFailure => {
                ConversionError::ParseError("Could not cast into intermediate decimal.".into())
            }
        }
    }
}

impl From<String> for ConversionError {
    fn from(value: String) -> Self {
        ConversionError::ParseError(value)
    }
}

impl From<ConversionError> for String {
    fn from(error: ConversionError) -> Self {
        error.to_string()
    }
}

impl ConversionError {
    /// Returns a `ConversionError::InvalidExpression` for non-numeric types used in numeric aggregation functions.
    pub fn non_numeric_expr_in_agg<S: Into<String>>(dtype: S, func: S) -> Self {
        ConversionError::InvalidExpression(format!(
            "cannot use expression of type '{}' with numeric aggregation function '{}'",
            dtype.into().to_lowercase(),
            func.into().to_lowercase()
        ))
    }
}

pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
