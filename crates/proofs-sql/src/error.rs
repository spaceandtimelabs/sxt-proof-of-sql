use thiserror::Error;

/// Errors encountered during the parsing process
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseError {
    #[error("Unable to parse query")]
    QueryParseError(String),
    #[error("Unable to parse identifier")]
    IdentifierParseError(String),
    #[error("Unable to parse resource_id")]
    ResourceIdParseError(String),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
