use alloc::string::String;
use snafu::Snafu;

/// Errors encountered during the process of converting `sqlparser::ast::Statement` to `LogicalPlan`
#[derive(Debug, Snafu)]
pub enum LogicalPlanError {
    #[snafu(display("Unsupported feature: {:?}", message))]
    /// Used when a feature parseable in `sqlparser` is not supported
    Unsupported {
        /// The unsupported feature
        message: String,
    },
}
