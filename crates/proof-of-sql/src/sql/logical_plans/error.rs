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
    #[snafu(display("Unsupported unary operator: {:?}", op))]
    /// Used when a unary operator is not supported
    UnsupportedUnaryOperator {
        /// The unsupported unary operator
        op: sqlparser::ast::UnaryOperator,
    },
}
