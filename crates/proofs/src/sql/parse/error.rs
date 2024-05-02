use crate::base::database::ColumnType;
use proofs_sql::{Identifier, ResourceId};
use thiserror::Error;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConversionError {
    #[error("Column '{0}' was not found in table '{1}'")]
    /// TODO: add docs
    MissingColumn(Box<Identifier>, Box<ResourceId>),
    #[error("Expected '{expected}' but found '{actual}'")]
    /// TODO: add docs
    InvalidDataType {
        /// TODO: add docs
        expected: ColumnType,
        /// TODO: add docs
        actual: ColumnType,
    },
    #[error("Left side has '{1}' type but right side has '{0}' type")]
    /// TODO: add docs
    DataTypeMismatch(String, String),
    #[error("Multiple result columns with the same alias '{0}' have been found.")]
    /// TODO: add docs
    DuplicateResultAlias(String),
    #[error("Invalid order by: alias '{0}' does not appear in the result expressions.")]
    /// TODO: add docs
    InvalidOrderBy(String),
    #[error("Invalid group by: column '{0}' must appear in the group by expression.")]
    /// TODO: add docs
    InvalidGroupByColumnRef(String),
    #[error("Invalid expression: {0}")]
    /// TODO: add docs
    InvalidExpression(String),
    #[error("Error while parsing precision from query: {0}")]
    /// TODO: add docs
    PrecisionParseError(String),
    #[error("Encountered parsing error: {0}")]
    /// TODO: add docs
    ParseError(String),
    #[error("Unsupported operation: cannot round literal: {0}")]
    /// TODO: add docs
    LiteralRoundDownError(String),
    #[error("Query not provable because: {0}")]
    /// TODO: add docs
    Unprovable(String),
}

impl From<String> for ConversionError {
    fn from(value: String) -> Self {
        ConversionError::ParseError(value)
    }
}

impl ConversionError {
    /// TODO: add docs
    pub fn non_numeric_expr_in_agg<S: Into<String>>(dtype: S, func: S) -> Self {
        ConversionError::InvalidExpression(format!(
            "cannot use expression of type '{}' with numeric aggregation function '{}'",
            dtype.into().to_lowercase(),
            func.into().to_lowercase()
        ))
    }
}

pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
