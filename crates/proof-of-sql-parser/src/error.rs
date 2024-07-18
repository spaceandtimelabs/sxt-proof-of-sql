use serde::{Deserialize, Serialize};
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

/// General parsing error that may occur, for example if the provided schema/object_name strings
/// aren't valid postgres-style identifiers (excluding dollar signs).
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// Errors related to time operations, including timezone and timestamp conversions.s
#[derive(Error, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoSQLTimestampError {
    /// Error when the timezone string provided cannot be parsed into a valid timezone.
    #[error("invalid timezone string: {0}")]
    InvalidTimezone(String),

    /// Error indicating an invalid timezone offset was provided.
    #[error("invalid timezone offset")]
    InvalidTimezoneOffset,

    /// Indicates a failure to convert between different representations of time units.
    #[error("Invalid time unit")]
    InvalidTimeUnit(String),

    /// The local time does not exist because there is a gap in the local time.
    /// This variant may also be returned if there was an error while resolving the local time,
    /// caused by for example missing time zone data files, an error in an OS API, or overflow.
    #[error("Local time does not exist because there is a gap in the local time")]
    LocalTimeDoesNotExist,

    /// The local time is ambiguous because there is a fold in the local time.
    /// This variant contains the two possible results, in the order (earliest, latest).
    #[error("Unix timestamp is ambiguous because there is a fold in the local time.")]
    Ambiguous(String),

    /// Represents a catch-all for parsing errors not specifically covered by other variants.
    #[error("Timestamp parsing error: {0}")]
    ParsingError(String),

    /// Represents a failure to parse a provided time unit precision value, PoSQL supports
    /// Seconds, Milliseconds, Microseconds, and Nanoseconds
    #[error("Timestamp parsing error: {0}")]
    UnsupportedPrecision(String),
}

// This exists because TryFrom<DataType> for ColumnType error is String
impl From<PoSQLTimestampError> for String {
    fn from(error: PoSQLTimestampError) -> Self {
        error.to_string()
    }
}
