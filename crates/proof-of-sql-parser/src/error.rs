use alloc::string::String;
use snafu::Snafu;

/// Errors encountered during the parsing process
#[allow(clippy::module_name_repetitions)]
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
    /// Cannot parse the `resource_id`
    ResourceIdParseError {
        /// The underlying error
        error: String,
    },
}

/// General parsing error that may occur, for example if the provided `schema`/`object_name` strings
/// aren't valid postgres-style identifiers (excluding dollar signs).
pub type ParseResult<T> = Result<T, ParseError>;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_query_parse_error() {
        let error = ParseError::QueryParseError {
            error: "test error".into(),
        };
        assert_eq!(error.to_string(), "Unable to parse query");
    }

    #[test]
    fn test_identifier_parse_error() {
        let error = ParseError::IdentifierParseError {
            error: "test error".into(),
        };
        assert_eq!(error.to_string(), "Unable to parse identifier");
    }

    #[test]
    fn test_resource_id_parse_error() {
        let error = ParseError::ResourceIdParseError {
            error: "test error".into(),
        };
        assert_eq!(error.to_string(), "Unable to parse resource_id");
    }

    #[test]
    fn test_error_equality() {
        let error1 = ParseError::QueryParseError {
            error: "test error".into(),
        };
        let error2 = ParseError::QueryParseError {
            error: "test error".into(),
        };
        let error3 = ParseError::QueryParseError {
            error: "different error".into(),
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_parse_result_type() {
        let result: ParseResult<()> = Err(ParseError::QueryParseError {
            error: "test error".into(),
        });
        assert!(result.is_err());

        let ok_result: ParseResult<i32> = Ok(42);
        assert!(ok_result.is_ok());
    }
}
