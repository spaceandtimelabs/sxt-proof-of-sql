use thiserror::Error;

/// Errors encountered during the parsing process
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ParseError {
    /// Unable to parse resource id
    #[error("Unable to parse resource_id")]
    ResourceIdParseError(String),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
