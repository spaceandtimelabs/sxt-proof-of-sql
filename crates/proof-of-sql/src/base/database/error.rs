use alloc::string::String;
use snafu::Snafu;

/// Errors encountered during the parsing process
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Snafu, Eq, PartialEq)]
pub enum ParseError {
    #[snafu(display("Invalid table reference: {}", table_reference))]
    /// Cannot parse the `TableRef`
    InvalidTableReference {
        /// The underlying error
        table_reference: String,
    },
}
