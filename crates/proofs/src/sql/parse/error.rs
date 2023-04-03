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
    #[error("The specified column alias '{0}' referenced by the 'order by' clause does not exist in the '{1}' table.")]
    InvalidOrderByError(String, String),
    #[error("The 'order by' clause '{0}' is ambiguous because multiple result columns with the same alias have been found.")]
    AmbiguousOrderByError(String),
}

pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
