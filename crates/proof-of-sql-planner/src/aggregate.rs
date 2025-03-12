use super::{PlannerError, PlannerResult};
use crate::expr_to_proof_expr;
use datafusion::{
    common::DFSchema,
    logical_expr::expr::{AggregateFunction, AggregateFunctionDefinition},
    physical_plan,
};
use proof_of_sql::sql::proof_exprs::DynProofExpr;
use proof_of_sql_parser::intermediate_ast::AggregationOperator;

/// Convert an [`AggregateFunction`] to a [`DynProofExpr`]
///
/// TODO: Some moderate changes are necessary once we upgrade `DataFusion` to 46.0.0
pub(crate) fn aggregate_function_to_proof_expr(
    function: &AggregateFunction,
    schema: &DFSchema,
) -> PlannerResult<DynProofExpr> {
    if !matches!(
        (
            function.distinct,
            &function.filter,
            &function.order_by,
            function.args.len()
        ),
        (false, &None, &None, 1)
    ) {
        return Err(PlannerError::UnsupportedAggregateFunction {
            function: function.clone(),
        });
    }
    let aggregation_operator = match function.func_def {
        AggregateFunctionDefinition::BuiltIn(
            physical_plan::aggregates::AggregateFunction::Count,
        ) => AggregationOperator::Count,
        AggregateFunctionDefinition::BuiltIn(physical_plan::aggregates::AggregateFunction::Sum) => {
            AggregationOperator::Sum
        }
        _ => {
            return Err(PlannerError::UnsupportedAggregateFunction {
                function: function.clone(),
            });
        }
    };
    Ok(DynProofExpr::new_aggregate(
        aggregation_operator,
        expr_to_proof_expr(&function.args[0], schema)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::df_util::*;
    use arrow::datatypes::DataType;
    use proof_of_sql::base::database::{ColumnRef, ColumnType, TableRef};

    // AggregateFunction to DynProofExpr
    #[test]
    fn we_can_convert_an_aggregate_function_to_proof_expr() {
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        for (function, operator) in &[
            (
                physical_plan::aggregates::AggregateFunction::Sum,
                AggregationOperator::Sum,
            ),
            (
                physical_plan::aggregates::AggregateFunction::Count,
                AggregationOperator::Count,
            ),
        ] {
            let function = AggregateFunction::new(
                function.clone(),
                vec![expr.clone()],
                false,
                None,
                None,
                None,
            );
            assert_eq!(
                aggregate_function_to_proof_expr(&function, &schema).unwrap(),
                DynProofExpr::new_aggregate(
                    *operator,
                    DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(None, "table"),
                        "a".into(),
                        ColumnType::BigInt
                    ))
                )
            );
        }
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_unsupported() {
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::RegrIntercept,
            vec![expr.clone()],
            false,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_too_many_or_no_exprs() {
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        // Too many exprs
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Sum,
            vec![expr.clone(); 2],
            false,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // No exprs
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Sum,
            Vec::<_>::new(),
            false,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_unsupported_options() {
        // No distinct, filter, or order_by

        // Distinct
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Count,
            vec![expr.clone()],
            true,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // Filter
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Count,
            vec![expr.clone()],
            false,
            Some(Box::new(expr.clone())),
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // OrderBy
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Count,
            vec![expr.clone()],
            false,
            None,
            Some(vec![expr.clone()]),
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));
    }
}
