use arrow::datatypes::DataType;
use datafusion::{
    common::DataFusionError,
    logical_expr::{expr::AggregateFunction, Expr, LogicalPlan, Operator},
    physical_plan,
};
use proof_of_sql::{base::math::decimal::DecimalError, sql::AnalyzeError};
use snafu::Snafu;
use sqlparser::parser::ParserError;

/// Proof of SQL Planner error
#[derive(Debug, Snafu)]
pub enum PlannerError {
    /// Returned when the internal analyze process fails
    #[snafu(transparent)]
    AnalyzeError {
        /// Underlying analyze error
        source: AnalyzeError,
    },
    /// Returned when a decimal error occurs
    #[snafu(transparent)]
    DecimalError {
        /// Underlying decimal error
        source: DecimalError,
    },
    /// Returned when sqlparser fails to parse a query
    #[snafu(transparent)]
    SqlParserError {
        /// Underlying sqlparser error
        source: ParserError,
    },
    /// Returned when datafusion fails to plan a query
    #[snafu(transparent)]
    DataFusionError {
        /// Underlying datafusion error
        source: DataFusionError,
    },
    /// Returned if a table is not found
    #[snafu(display("Table not found: {}", table_name))]
    TableNotFound {
        /// Table name
        table_name: String,
    },
    /// Returned when a datatype is not supported
    #[snafu(display("Unsupported datatype: {}", data_type))]
    UnsupportedDataType {
        /// Unsupported datatype
        data_type: DataType,
    },
    /// Returned when a binary operator is not supported
    #[snafu(display("Binary operator {} is not supported", op))]
    UnsupportedBinaryOperator {
        /// Unsupported binary operation
        op: Operator,
    },
    /// Returned when the aggregate opetation is not supported
    #[snafu(display("Aggregate operation {op:?} is not supported"))]
    UnsupportedAggregateOperation {
        /// Unsupported aggregate operation
        op: physical_plan::aggregates::AggregateFunction,
    },
    /// Returned when the `AggregateFunction` is not supported
    #[snafu(display("AggregateFunction {function:?} is not supported"))]
    UnsupportedAggregateFunction {
        /// Unsupported `AggregateFunction`
        function: AggregateFunction,
    },
    /// Returned when a logical expression is not resolved
    #[snafu(display("Logical expression {:?} is not supported", expr))]
    UnsupportedLogicalExpression {
        /// Unsupported logical expression
        expr: Expr,
    },
    /// Returned when a `LogicalPlan` is not supported
    #[snafu(display("LogicalPlan is not supported"))]
    UnsupportedLogicalPlan {
        /// Unsupported `LogicalPlan`
        plan: LogicalPlan,
    },
    /// Returned when the `LogicalPlan` is not resolved
    #[snafu(display("LogicalPlan is not resolved"))]
    UnresolvedLogicalPlan,
    /// Returned when catalog is provided since it is not supported
    #[snafu(display("Catalog is not supported"))]
    CatalogNotSupported,
}

/// Proof of SQL Planner result
pub type PlannerResult<T> = Result<T, PlannerError>;
