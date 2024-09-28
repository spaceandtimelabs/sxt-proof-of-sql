use alloc::string::String;
use snafu::Snafu;

/// Errors encountered during the parsing process
#[derive(Debug, Snafu, Eq, PartialEq)]
pub enum ParseError {
    #[snafu(display("Unable to parse query"))]
    /// Cannot parse the query
    QueryParseError {
        /// The underlying error
        error: String,
    },
    #[snafu(display("Unable to parse identifier"))]
    /// Cannot parse the identifier
    IdentifierParseError {
        /// The underlying error
        error: String,
    },
    #[snafu(display("Unable to parse resource_id"))]
    /// Can not parse the resource_id
    ResourceIdParseError {
        /// The underlying error
        error: String,
    },
}

/// General parsing error that may occur, for example if the provided schema/object_name strings
/// aren't valid postgres-style identifiers (excluding dollar signs).
pub type ParseResult<T> = Result<T, ParseError>;
