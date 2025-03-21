use datafusion::common::DataFusionError;
use proof_of_sql::base::database::OwnedTableError;
use snafu::Snafu;

/// Errors in postprocessing
#[derive(Snafu, Debug)]
pub enum PostprocessingError {
    /// Errors in evaluation of `Expression`s
    #[snafu(transparent)]
    ExpressionEvaluationError {
        /// The underlying source error
        source: crate::postprocessing::ExpressionEvaluationError,
    },
    /// Returned when a datafusion error occurs
    #[snafu(transparent)]
    DataFusionError {
        /// Underlying datafusion error
        source: DataFusionError,
    },
    /// Returned when an `OwnedTableError` occurs
    #[snafu(transparent)]
    OwnedTableError {
        /// Underlying `OwnedTableError`
        source: OwnedTableError,
    },
}

/// Result type for postprocessing
pub type PostprocessingResult<T> = core::result::Result<T, PostprocessingError>;
