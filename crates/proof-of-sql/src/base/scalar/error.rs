use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
/// These errors occur when a scalar conversion fails.
pub enum ScalarConversionError {
    #[error("Overflow error: {error}")]
    /// This error occurs when a scalar is too large to be converted.
    Overflow {
        /// The underlying error
        error: String,
    },
}
