use thiserror::Error;

use proofs_sql::{Identifier, ResourceId};

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConversionError {
    /// This error occurs when a part of the query is of a wrong type (e.g. applying + to booleans)
    #[error("Type error")]
    TypeError(String),
    /// This error occurs when a column doesn't exist
    #[error("Column '{0}' was not found in table '{1}'")]
    MissingColumnError(Box<Identifier>, Box<ResourceId>),
    /// This error occurs when the lhs column has a type different from the rhs literal in the equal expression
    #[error("Left side has '{1}' type but right side has '{0}' type")]
    MismatchTypeError(String, String),
    #[error(
        "The specified column alias '{0}' referenced by the 'order by' clause does not exist."
    )]
    InvalidOrderByError(String),
    #[error("Multiple result columns with the same alias '{0}' have been found.")]
    DuplicateColumnAlias(String),
    #[error("Using aggregation functions with no group by clause specified.")]
    MissingGroupByError,
    #[error("Group by clause requires all non-aggregated result columns to be included in the clause or use an aggregation function.")]
    InvalidGroupByResultColumnError,
    #[error("Cannot aggregate a non-numeric column with function '{0}'.")]
    NonNumericColumnAggregation(&'static str),
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
}

pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
