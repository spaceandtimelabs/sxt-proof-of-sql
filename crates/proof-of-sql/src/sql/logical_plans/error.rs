use crate::base::math::decimal::DecimalError;
use snafu::Snafu;
use sqlparser::ast::{BinaryOperator, Value};

/// Errors encountered during the process of converting `sqlparser::ast::Statement` to `LogicalPlan`
#[derive(Debug, Snafu)]
pub enum LogicalPlanError {
    #[snafu(display("Unsupported binary operator: {:?}", op))]
    /// Used when a binary operator is not supported
    UnsupportedBinaryOperator {
        /// The unsupported binary operator
        op: BinaryOperator,
    },
    #[snafu(display("Unsupported Value: {:?}", value))]
    /// Used when a value is not supported
    UnsupportedValue {
        /// The unsupported value
        value: Value,
    },
    /// Used when a value can not be parsed as a decimal
    #[snafu(transparent)]
    DecimalParseError {
        /// The underlying error
        source: DecimalError,
    },
}
