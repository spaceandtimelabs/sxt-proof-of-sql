use proof_of_sql::sql::proof_plans::AggregateExecError;
use proof_of_sql_planner::{AggregatePlanError, JoinPlanError, PlannerError};
use serde::{Deserialize, Serialize};

/// Structured validation failure exposed across the `WebAssembly` boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerDiagnostic {
    /// Human-readable planner error.
    pub message: String,
    /// Machine-readable error kind and its available context.
    #[serde(flatten)]
    pub kind: PlannerDiagnosticKind,
}

impl PlannerDiagnostic {
    pub(crate) fn new(message: impl Into<String>, kind: PlannerDiagnosticKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }

    pub(crate) fn from_planner_error(error: &PlannerError) -> Self {
        Self::new(error.to_string(), PlannerDiagnosticKind::from(error))
    }
}

/// Machine-readable validation failure and the context retained by the planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "code",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum PlannerDiagnosticKind {
    /// Input JSON did not match the planner request contract.
    InvalidInput,
    /// The input SQL is empty.
    EmptyQuery,
    /// SQL parsing failed.
    SqlParseError,
    /// Validation accepts exactly one statement.
    NotOneStatement {
        /// Number of statements supplied.
        count: usize,
    },
    /// A table identifier was not in `NAMESPACE.TABLE` form.
    InvalidTableReference,
    /// No schema was supplied for a referenced table.
    MissingSchema {
        /// Fully qualified table name.
        table: String,
    },
    /// A supplied schema contained an unsupported SQL datatype.
    UnsupportedSchemaType {
        /// Fully qualified table name.
        table: String,
        /// Column with the unsupported datatype.
        column: String,
        /// Unsupported SXT schema datatype.
        data_type: String,
    },
    /// Proof of SQL analysis rejected the query.
    AnalysisError,
    /// Decimal validation failed.
    DecimalError,
    /// The planner's SQL parser rejected the query.
    PlannerSqlParseError,
    /// `DataFusion` could not plan or optimize the query.
    DataFusionError,
    /// A referenced column was not found.
    ColumnNotFound,
    /// A referenced table was not found.
    TableNotFound {
        /// Fully qualified table name.
        table: String,
    },
    /// A placeholder id was invalid.
    InvalidPlaceholderId {
        /// Invalid placeholder identifier.
        id: String,
    },
    /// A placeholder did not have a resolvable type.
    UntypedPlaceholder {
        /// Untyped placeholder expression.
        placeholder: String,
    },
    /// A datatype in the query is unsupported.
    UnsupportedDataType {
        /// Unsupported datatype.
        data_type: String,
    },
    /// A binary operator in the query is unsupported.
    UnsupportedBinaryOperator {
        /// Unsupported operator.
        operator: String,
    },
    /// An aggregate operation in the physical plan is unsupported.
    UnsupportedAggregateOperation {
        /// Unsupported physical aggregate operation.
        operation: String,
    },
    /// An aggregate function in the logical plan is unsupported.
    UnsupportedAggregateFunction {
        /// Unsupported logical aggregate function.
        function: String,
    },
    /// A logical expression in the query is unsupported.
    UnsupportedLogicalExpression {
        /// Unsupported logical expression.
        expression: String,
    },
    /// A grouping expression was absent from the output alias map.
    MissingGroupExpressionAlias {
        /// Grouping expression without an alias.
        expression: String,
    },
    /// An aggregate expression was absent from the output alias map.
    MissingAggregateExpressionAlias {
        /// Aggregate expression without an alias.
        expression: String,
    },
    /// An aggregate plan contained a non-aggregate expression.
    UnexpectedAggregateExpression {
        /// Unexpected aggregate-plan expression.
        expression: String,
    },
    /// Grouping or deduplication contained too many expressions.
    UnsupportedGroupByExpressionCount {
        /// Number of grouping or deduplication expressions.
        count: usize,
    },
    /// A grouping or deduplication expression had an unsupported datatype.
    UnsupportedGroupByExpressionType {
        /// Unsupported grouping datatype.
        data_type: String,
    },
    /// A join type is unsupported.
    UnsupportedJoinType {
        /// Unsupported join type.
        join_type: String,
    },
    /// A join constraint is unsupported.
    UnsupportedJoinConstraint {
        /// Unsupported join constraint.
        constraint: String,
    },
    /// A join predicate has an unsupported shape.
    UnsupportedJoinPredicate {
        /// Left-hand join expression.
        left: String,
        /// Right-hand join expression.
        right: String,
    },
    /// A `DataFusion` logical-plan node has no Proof of SQL conversion.
    UnsupportedLogicalPlan {
        /// Unsupported logical-plan node kind.
        node: String,
    },
    /// The planner produced an unresolved logical plan.
    UnresolvedLogicalPlan,
    /// Three-part catalog references are unsupported.
    CatalogNotSupported,
    /// The adapter could not classify or encode a validation failure.
    UnknownPlannerError,
}

impl From<&PlannerError> for PlannerDiagnosticKind {
    fn from(error: &PlannerError) -> Self {
        match error {
            PlannerError::AnalyzeError { .. } => Self::AnalysisError,
            PlannerError::DecimalError { .. } => Self::DecimalError,
            PlannerError::SqlParserError { .. } => Self::PlannerSqlParseError,
            PlannerError::DataFusionError { .. } => Self::DataFusionError,
            PlannerError::ColumnNotFound => Self::ColumnNotFound,
            PlannerError::TableNotFound { table_name } => Self::TableNotFound {
                table: table_name.clone(),
            },
            PlannerError::InvalidPlaceholderId { id } => {
                Self::InvalidPlaceholderId { id: id.clone() }
            }
            PlannerError::UntypedPlaceholder { placeholder } => Self::UntypedPlaceholder {
                placeholder: format!("{placeholder:?}"),
            },
            PlannerError::UnsupportedDataType { data_type } => Self::UnsupportedDataType {
                data_type: data_type.to_string(),
            },
            PlannerError::UnsupportedBinaryOperator { op } => Self::UnsupportedBinaryOperator {
                operator: op.to_string(),
            },
            PlannerError::UnsupportedAggregateOperation { op } => {
                Self::UnsupportedAggregateOperation {
                    operation: format!("{op:?}"),
                }
            }
            PlannerError::UnsupportedAggregateFunction { function } => {
                Self::UnsupportedAggregateFunction {
                    function: format!("{function:?}"),
                }
            }
            PlannerError::UnsupportedLogicalExpression { expr } => {
                Self::UnsupportedLogicalExpression {
                    expression: format!("{expr:?}"),
                }
            }
            PlannerError::UnsupportedAggregatePlan { source } => Self::from(source),
            PlannerError::UnsupportedJoinPlan { source } => Self::from(source),
            PlannerError::UnsupportedLogicalPlan { node } => Self::UnsupportedLogicalPlan {
                node: node.to_string(),
            },
            PlannerError::UnresolvedLogicalPlan => Self::UnresolvedLogicalPlan,
            PlannerError::CatalogNotSupported => Self::CatalogNotSupported,
            _ => Self::UnknownPlannerError,
        }
    }
}

impl From<&AggregatePlanError> for PlannerDiagnosticKind {
    fn from(error: &AggregatePlanError) -> Self {
        match error {
            AggregatePlanError::MissingGroupExpressionAlias { expression } => {
                Self::MissingGroupExpressionAlias {
                    expression: expression.clone(),
                }
            }
            AggregatePlanError::MissingAggregateExpressionAlias { expression } => {
                Self::MissingAggregateExpressionAlias {
                    expression: expression.clone(),
                }
            }
            AggregatePlanError::UnexpectedAggregateExpression { expression } => {
                Self::UnexpectedAggregateExpression {
                    expression: expression.clone(),
                }
            }
            AggregatePlanError::AggregateExec { source } => Self::from(source),
            _ => Self::UnknownPlannerError,
        }
    }
}

impl From<&AggregateExecError> for PlannerDiagnosticKind {
    fn from(error: &AggregateExecError) -> Self {
        match error {
            AggregateExecError::UnsupportedGroupByExpressionCount { count } => {
                Self::UnsupportedGroupByExpressionCount { count: *count }
            }
            AggregateExecError::UnsupportedGroupByExpressionType { data_type } => {
                Self::UnsupportedGroupByExpressionType {
                    data_type: data_type.to_string(),
                }
            }
            _ => Self::UnknownPlannerError,
        }
    }
}

impl From<&JoinPlanError> for PlannerDiagnosticKind {
    fn from(error: &JoinPlanError) -> Self {
        match error {
            JoinPlanError::UnsupportedJoinType { join_type } => Self::UnsupportedJoinType {
                join_type: join_type.to_string(),
            },
            JoinPlanError::UnsupportedJoinConstraint { constraint } => {
                Self::UnsupportedJoinConstraint {
                    constraint: format!("{constraint:?}"),
                }
            }
            JoinPlanError::UnsupportedPredicate { left, right } => Self::UnsupportedJoinPredicate {
                left: left.clone(),
                right: right.clone(),
            },
            _ => Self::UnknownPlannerError,
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::missing_panics_doc, clippy::too_many_lines)]

    use super::*;
    use datafusion::{
        arrow::datatypes::DataType,
        common::{DataFusionError, ScalarValue},
        logical_expr::{
            expr::{AggregateFunction, Placeholder},
            Expr, Operator,
        },
        physical_plan,
    };
    use proof_of_sql::{
        base::{database::ColumnType, math::decimal::DecimalError},
        sql::AnalyzeError,
    };
    use proof_of_sql_planner::LogicalPlanNodeKind;
    use sqlparser::parser::ParserError;

    fn assert_mapping(error: &PlannerError, expected: &PlannerDiagnosticKind) {
        let diagnostic = PlannerDiagnostic::from_planner_error(error);
        assert_eq!(diagnostic.message, error.to_string());
        assert_eq!(&diagnostic.kind, expected);
    }

    #[test]
    fn aggregate_plan_errors_map_to_diagnostics() {
        let cases = [
            (
                PlannerError::UnsupportedAggregatePlan {
                    source: AggregatePlanError::MissingGroupExpressionAlias {
                        expression: "APP.T.ID".to_string(),
                    },
                },
                PlannerDiagnosticKind::MissingGroupExpressionAlias {
                    expression: "APP.T.ID".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedAggregatePlan {
                    source: AggregatePlanError::MissingAggregateExpressionAlias {
                        expression: "SUM(APP.T.ID)".to_string(),
                    },
                },
                PlannerDiagnosticKind::MissingAggregateExpressionAlias {
                    expression: "SUM(APP.T.ID)".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedAggregatePlan {
                    source: AggregatePlanError::UnexpectedAggregateExpression {
                        expression: "APP.T.ID".to_string(),
                    },
                },
                PlannerDiagnosticKind::UnexpectedAggregateExpression {
                    expression: "APP.T.ID".to_string(),
                },
            ),
        ];

        for (error, expected) in cases {
            let diagnostic = PlannerDiagnostic::from_planner_error(&error);
            assert_eq!(diagnostic.message, error.to_string());
            assert_eq!(diagnostic.kind, expected);
        }
    }

    #[test]
    fn planner_errors_map_to_diagnostics() {
        let cases = [
            (
                PlannerError::AnalyzeError {
                    source: AnalyzeError::InvalidDataType {
                        expr_type: ColumnType::VarChar,
                    },
                },
                PlannerDiagnosticKind::AnalysisError,
            ),
            (
                PlannerError::DecimalError {
                    source: DecimalError::InvalidPrecision {
                        error: "76".to_string(),
                    },
                },
                PlannerDiagnosticKind::DecimalError,
            ),
            (
                PlannerError::SqlParserError {
                    source: ParserError::ParserError("invalid SQL".to_string()),
                },
                PlannerDiagnosticKind::PlannerSqlParseError,
            ),
            (
                PlannerError::DataFusionError {
                    source: DataFusionError::Plan("invalid plan".to_string()),
                },
                PlannerDiagnosticKind::DataFusionError,
            ),
            (
                PlannerError::ColumnNotFound,
                PlannerDiagnosticKind::ColumnNotFound,
            ),
            (
                PlannerError::TableNotFound {
                    table_name: "APP.T".to_string(),
                },
                PlannerDiagnosticKind::TableNotFound {
                    table: "APP.T".to_string(),
                },
            ),
            (
                PlannerError::InvalidPlaceholderId {
                    id: "$0".to_string(),
                },
                PlannerDiagnosticKind::InvalidPlaceholderId {
                    id: "$0".to_string(),
                },
            ),
            (
                PlannerError::UntypedPlaceholder {
                    placeholder: Placeholder::new("$1".to_string(), None),
                },
                PlannerDiagnosticKind::UntypedPlaceholder {
                    placeholder: "Placeholder { id: \"$1\", data_type: None }".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedDataType {
                    data_type: DataType::Float64,
                },
                PlannerDiagnosticKind::UnsupportedDataType {
                    data_type: "Float64".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedBinaryOperator {
                    op: Operator::Divide,
                },
                PlannerDiagnosticKind::UnsupportedBinaryOperator {
                    operator: "/".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedAggregateOperation {
                    op: physical_plan::aggregates::AggregateFunction::Avg,
                },
                PlannerDiagnosticKind::UnsupportedAggregateOperation {
                    operation: "Avg".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedLogicalExpression {
                    expr: Box::new(Expr::Literal(ScalarValue::Null)),
                },
                PlannerDiagnosticKind::UnsupportedLogicalExpression {
                    expression: "Literal(NULL)".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedLogicalPlan {
                    node: LogicalPlanNodeKind::Sort,
                },
                PlannerDiagnosticKind::UnsupportedLogicalPlan {
                    node: "sort".to_string(),
                },
            ),
            (
                PlannerError::UnresolvedLogicalPlan,
                PlannerDiagnosticKind::UnresolvedLogicalPlan,
            ),
            (
                PlannerError::CatalogNotSupported,
                PlannerDiagnosticKind::CatalogNotSupported,
            ),
        ];

        for (error, expected) in cases {
            assert_mapping(&error, &expected);
        }

        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Count,
            Vec::new(),
            true,
            None,
            None,
            None,
        );
        let expected_function = format!("{function:?}");
        assert_mapping(
            &PlannerError::UnsupportedAggregateFunction { function },
            &PlannerDiagnosticKind::UnsupportedAggregateFunction {
                function: expected_function,
            },
        );
    }

    #[test]
    fn aggregate_exec_and_join_errors_map_to_diagnostics() {
        let cases = [
            (
                PlannerError::UnsupportedAggregatePlan {
                    source: AggregatePlanError::AggregateExec {
                        source: AggregateExecError::UnsupportedGroupByExpressionCount { count: 2 },
                    },
                },
                PlannerDiagnosticKind::UnsupportedGroupByExpressionCount { count: 2 },
            ),
            (
                PlannerError::UnsupportedAggregatePlan {
                    source: AggregatePlanError::AggregateExec {
                        source: AggregateExecError::UnsupportedGroupByExpressionType {
                            data_type: ColumnType::VarBinary,
                        },
                    },
                },
                PlannerDiagnosticKind::UnsupportedGroupByExpressionType {
                    data_type: "BINARY".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedJoinPlan {
                    source: JoinPlanError::UnsupportedJoinType {
                        join_type: datafusion::logical_expr::JoinType::Left,
                    },
                },
                PlannerDiagnosticKind::UnsupportedJoinType {
                    join_type: "Left".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedJoinPlan {
                    source: JoinPlanError::UnsupportedJoinConstraint {
                        constraint: datafusion::logical_expr::JoinConstraint::Using,
                    },
                },
                PlannerDiagnosticKind::UnsupportedJoinConstraint {
                    constraint: "Using".to_string(),
                },
            ),
            (
                PlannerError::UnsupportedJoinPlan {
                    source: JoinPlanError::UnsupportedPredicate {
                        left: "A.ID".to_string(),
                        right: "B.OTHER_ID".to_string(),
                    },
                },
                PlannerDiagnosticKind::UnsupportedJoinPredicate {
                    left: "A.ID".to_string(),
                    right: "B.OTHER_ID".to_string(),
                },
            ),
        ];

        for (error, expected) in cases {
            assert_mapping(&error, &expected);
        }
    }
}
