use thiserror::Error;

/// Errors from converting an intermediate AST into a provable AST.
#[derive(Error, Debug)]
pub enum ConversionError {
    /// This error occurs when a part of the query is of a wrong type (e.g. applying + to booleans)
    #[error("Type error")]
    TypeError(String),
    /// This error occurs when a column doesn't exist
    #[error("Column missing")]
    MissingColumnError(String),
    /// This error occurs when the lhs column has a type different from the rhs literal in the equal expression
    #[error("Column type mismatch from query expression")]
    MismatchTypeError(String),
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
}

pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
