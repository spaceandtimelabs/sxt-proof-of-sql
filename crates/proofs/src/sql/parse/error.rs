use thiserror::Error;

/// Errors from parsing SQL.
///
/// We parse a query into an intermediate AST and then convert it to a provable AST.
/// Errors can happen in both processes and they are stored here.
#[derive(Error, Debug)]
pub enum ParseError {
    /// This error occurs when a part of the query is of a wrong type (e.g. applying + to booleans)
    #[error("Type error")]
    TypeError(String),
    /// This error occurs when a column doesn't exist
    #[error("Column missing")]
    MissingColumnError(String),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
