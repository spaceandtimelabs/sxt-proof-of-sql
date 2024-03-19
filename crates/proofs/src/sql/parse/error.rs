use proofs_sql::{Identifier, ResourceId};
use thiserror::Error;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConversionError {
    #[error("Column '{0}' was not found in table '{1}'")]
    MissingColumn(Box<Identifier>, Box<ResourceId>),
    #[error("Left side has '{1}' type but right side has '{0}' type")]
    DataTypeMismatch(String, String),
    #[error("Multiple result columns with the same alias '{0}' have been found.")]
    DuplicateResultAlias(String),
    #[error("Invalid order by: alias '{0}' does not appear in the result expressions.")]
    InvalidOrderBy(String),
    #[error("Invalid group by: column '{0}' must appear in the group by expression.")]
    InvalidGroupByColumnRef(String),
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
    #[error("Error while parsing precision from query: {0}")]
    PrecisionParseError(String),
    #[error("Encountered parsing error: {0}")]
    ParseError(String),
    #[error("Unsupported operation: cannot round literal: {0}")]
    LiteralRoundDownError(String),
}

impl From<String> for ConversionError {
    fn from(value: String) -> Self {
        ConversionError::ParseError(value)
    }
}

impl ConversionError {
    pub fn non_numeric_expr_in_agg<S: Into<String>>(dtype: S, func: S) -> Self {
        ConversionError::InvalidExpression(format!(
            "cannot use expression of type '{}' with numeric aggregation function '{}'",
            dtype.into().to_lowercase(),
            func.into().to_lowercase()
        ))
    }
}

pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
