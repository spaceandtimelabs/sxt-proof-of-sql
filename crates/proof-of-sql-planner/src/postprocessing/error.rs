use snafu::Snafu;

/// Errors in postprocessing
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum PostprocessingError {
    /// Errors in evaluation of `Expression`s
    #[snafu(transparent)]
    ExpressionEvaluationError {
        /// The underlying source error
        source: crate::postprocessing::ExpressionEvaluationError,
    },
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
