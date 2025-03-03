use snafu::Snafu;

/// Errors encountered during the process of converting `sqlparser::ast::Statement` to `LogicalPlan`
#[derive(Debug, Snafu)]
pub enum LogicalPlanError {
    #[snafu(display("Unsupported binary operator: {:?}", op))]
    /// Used when a binary operator is not supported
    UnsupportedBinaryOperator {
        /// The unsupported binary operator
        op: sqlparser::ast::BinaryOperator,
    },
}
