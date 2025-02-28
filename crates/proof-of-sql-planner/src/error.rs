use datafusion::common::DataFusionError;
use proof_of_sql::sql::parse::ConversionError;
use snafu::Snafu;

/// Planner error
#[derive(Debug, Snafu)]
pub enum PlannerError {
    /// Returned when a conversion fails
    #[snafu(transparent)]
    ConversionError { source: ConversionError },
    /// Returned when datafusion fails to plan a query
    #[snafu(transparent)]
    DataFusionError { source: DataFusionError },
    /// Internal error. Should never happen in normal operation of the program
    #[snafu(display("Internal error: {}", message))]
    InternalError {
        /// Error message
        message: &'static str,
    },
}

/// Planner result
pub type PlannerResult<T> = Result<T, PlannerError>;
