use thiserror::Error;

/// Errors encountered during the parsing process
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseError {
    #[error("Unable to parse query")]
    /// Cannot parse the query
    QueryParseError(String),
    #[error("Unable to parse identifier")]
    /// Cannot parse the identifier
    IdentifierParseError(String),
    #[error("Unable to parse resource_id")]
    /// Can not parse the resource_id
    ResourceIdParseError(String),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
