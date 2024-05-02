use thiserror::Error;

/// Errors encountered during the parsing process
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseError {
    #[error("Unable to parse query")]
    /// TODO: add docs
    QueryParseError(String),
    #[error("Unable to parse identifier")]
    /// TODO: add docs
    IdentifierParseError(String),
    #[error("Unable to parse resource_id")]
    /// TODO: add docs
    ResourceIdParseError(String),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
