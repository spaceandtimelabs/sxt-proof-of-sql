use super::{
    logical_plan_to_proof_plan, postprocessing::SelectPostprocessing, PlannerError, PlannerResult,
};
use datafusion::{
    common::DFSchema,
    logical_expr::{LogicalPlan, Projection},
    sql::TableReference,
};
use indexmap::IndexMap;
use proof_of_sql::sql::proof_plans::DynProofPlan;

/// A [`DynProofPlan`] with optional postprocessing
#[derive(Debug, Clone)]
pub struct ProofPlanWithPostprocessing {
    plan: DynProofPlan,
    postprocessing: Option<SelectPostprocessing>,
}

impl ProofPlanWithPostprocessing {
    /// Create a new `ProofPlanWithPostprocessing`
    #[must_use]
    pub fn new(plan: DynProofPlan, postprocessing: Option<SelectPostprocessing>) -> Self {
        Self {
            plan,
            postprocessing,
        }
    }

    /// Get the `DynProofPlan`
    #[must_use]
    pub fn plan(&self) -> &DynProofPlan {
        &self.plan
    }

    /// Get the postprocessing
    #[must_use]
    pub fn postprocessing(&self) -> Option<&SelectPostprocessing> {
        self.postprocessing.as_ref()
    }
}

/// Visit a [`datafusion::logical_plan::LogicalPlan`] and return a [`DynProofPlan`] with optional postprocessing
pub fn logical_plan_to_proof_plan_with_postprocessing(
    plan: &LogicalPlan,
    schemas: &IndexMap<TableReference, DFSchema>,
) -> PlannerResult<ProofPlanWithPostprocessing> {
    let result_proof_plan = logical_plan_to_proof_plan(plan, schemas);
    match result_proof_plan {
        Ok(proof_plan) => Ok(ProofPlanWithPostprocessing::new(proof_plan, None)),
        Err(_err) => {
            match plan {
                // For projections, we can apply a postprocessing step
                LogicalPlan::Projection(Projection { input, expr, .. }) => {
                    // If the inner `LogicalPlan` is not provable we error out
                    let input_proof_plan = logical_plan_to_proof_plan(input, schemas)?;
                    let postprocessing = SelectPostprocessing::new(expr.clone());
                    Ok(ProofPlanWithPostprocessing::new(
                        input_proof_plan,
                        Some(postprocessing),
                    ))
                }
                _ => Err(PlannerError::UnsupportedLogicalPlan { plan: plan.clone() }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{df_util::*, PoSqlTableSource};
    use alloc::sync::Arc;
    use arrow::datatypes::DataType;
    use core::ops::Mul;
    use datafusion::{
        common::{Column, ScalarValue},
        logical_expr::{
            expr::{AggregateFunction, AggregateFunctionDefinition},
            Aggregate, EmptyRelation, Expr, LogicalPlan, Prepare, TableScan, TableSource,
        },
        physical_plan,
    };
    use indexmap::indexmap;
    use proof_of_sql::{
        base::database::{ColumnField, ColumnRef, ColumnType, TableRef},
        sql::{
            proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
            proof_plans::DynProofPlan,
        },
    };

    const SUM: AggregateFunctionDefinition =
        AggregateFunctionDefinition::BuiltIn(physical_plan::aggregates::AggregateFunction::Sum);
    const COUNT: AggregateFunctionDefinition =
        AggregateFunctionDefinition::BuiltIn(physical_plan::aggregates::AggregateFunction::Count);

    #[expect(non_snake_case)]
    fn TABLE_LANGUAGES() -> TableRef {
        TableRef::from_names(None, "languages")
    }

    #[expect(non_snake_case)]
    fn SCHEMAS() -> IndexMap<TableReference, DFSchema> {
        indexmap! {
            TableReference::from("languages") => df_schema(
                "languages",
                vec![
                    ("name", DataType::Utf8),
                    ("language_family", DataType::Utf8),
                    ("uses_abjad", DataType::Boolean),
                    ("num_of_letters", DataType::Int64),
                    ("grace", DataType::Utf8),
                    ("love", DataType::Utf8),
                    ("joy", DataType::Utf8),
                    ("peace", DataType::Utf8),
                ],
            ),
        }
    }

    #[expect(non_snake_case)]
    fn TABLE_SOURCE() -> Arc<dyn TableSource> {
        Arc::new(PoSqlTableSource::new(vec![
            ColumnField::new("name".into(), ColumnType::VarChar),
            ColumnField::new("language_family".into(), ColumnType::VarChar),
            ColumnField::new("uses_abjad".into(), ColumnType::Boolean),
            ColumnField::new("num_of_letters".into(), ColumnType::BigInt),
            ColumnField::new("grace".into(), ColumnType::VarChar),
            ColumnField::new("love".into(), ColumnType::VarChar),
            ColumnField::new("joy".into(), ColumnType::VarChar),
            ColumnField::new("peace".into(), ColumnType::VarChar),
        ]))
    }

    #[expect(non_snake_case)]
    fn ALIASED_NAME() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_LANGUAGES(),
                "name".into(),
                ColumnType::VarChar,
            )),
            alias: "name".into(),
        }
    }

    #[expect(non_snake_case)]
    fn ALIASED_GRACE() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_LANGUAGES(),
                "grace".into(),
                ColumnType::VarChar,
            )),
            alias: "grace".into(),
        }
    }

    #[expect(non_snake_case)]
    fn ALIASED_LOVE() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_LANGUAGES(),
                "love".into(),
                ColumnType::VarChar,
            )),
            alias: "love".into(),
        }
    }

    #[expect(non_snake_case)]
    fn ALIASED_JOY() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_LANGUAGES(),
                "joy".into(),
                ColumnType::VarChar,
            )),
            alias: "joy".into(),
        }
    }

    #[expect(non_snake_case)]
    fn COUNT_1() -> Expr {
        Expr::AggregateFunction(AggregateFunction {
            func_def: COUNT,
            args: vec![Expr::Literal(ScalarValue::Int64(Some(1)))],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        })
    }

    #[expect(non_snake_case)]
    fn SUM_NUM_LETTERS() -> Expr {
        Expr::AggregateFunction(AggregateFunction {
            func_def: SUM,
            args: vec![df_column("languages", "num_of_letters")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        })
    }

    #[expect(non_snake_case)]
    fn ALIASED_PEACE() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_LANGUAGES(),
                "peace".into(),
                ColumnType::VarChar,
            )),
            alias: "peace".into(),
        }
    }

    #[test]
    fn we_can_convert_logical_plan_to_proof_plan_without_postprocessing() {
        let plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "languages",
                TABLE_SOURCE(),
                Some(vec![0, 4, 5, 6, 7]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan_with_postprocessing(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_projection(
            vec![
                ALIASED_NAME(),
                ALIASED_GRACE(),
                ALIASED_LOVE(),
                ALIASED_JOY(),
                ALIASED_PEACE(),
            ],
            DynProofPlan::new_table(
                TABLE_LANGUAGES(),
                vec![
                    ColumnField::new("name".into(), ColumnType::VarChar),
                    ColumnField::new("language_family".into(), ColumnType::VarChar),
                    ColumnField::new("uses_abjad".into(), ColumnType::Boolean),
                    ColumnField::new("num_of_letters".into(), ColumnType::BigInt),
                    ColumnField::new("grace".into(), ColumnType::VarChar),
                    ColumnField::new("love".into(), ColumnType::VarChar),
                    ColumnField::new("joy".into(), ColumnType::VarChar),
                    ColumnField::new("peace".into(), ColumnType::VarChar),
                ],
            ),
        );
        assert_eq!(result.plan(), &expected);
        assert!(result.postprocessing().is_none());
    }

    #[test]
    fn we_can_convert_logical_plan_to_proof_plan_with_postprocessing() {
        // Setup group expression
        let group_expr = vec![df_column("languages", "language_family")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            SUM_NUM_LETTERS(), // SUM
            COUNT_1(),         // COUNT
        ];

        // Create filters
        let filter_exprs = vec![
            df_column("languages", "uses_abjad"), // Boolean column as filter
        ];

        // Create the input plan with filters
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "languages",
                TABLE_SOURCE(),
                Some(vec![1, 2, 3]),
                filter_exprs,
                None,
            )
            .unwrap(),
        );

        let agg_plan = LogicalPlan::Aggregate(
            Aggregate::try_new(Arc::new(input_plan), group_expr.clone(), aggr_expr.clone())
                .unwrap(),
        );

        let proj_plan = LogicalPlan::Projection(
            Projection::try_new(
                vec![
                    df_column("languages", "language_family"),
                    Expr::Column(Column::new(
                        None::<TableReference>,
                        "COUNT(Int64(1))".to_string(),
                    ))
                    .mul(Expr::Literal(ScalarValue::Int64(Some(2))))
                    .alias("twice_num_languages_using_abjad"),
                    Expr::Column(Column::new(
                        None::<TableReference>,
                        "SUM(languages.num_of_letters)".to_string(),
                    ))
                    .alias("sum_num_of_letters"),
                ],
                Arc::new(agg_plan),
            )
            .unwrap(),
        );

        // Test the function
        let result =
            logical_plan_to_proof_plan_with_postprocessing(&proj_plan, &SCHEMAS()).unwrap();

        // Expected result
        let expected_plan = DynProofPlan::new_group_by(
            vec![ColumnExpr::new(ColumnRef::new(
                TABLE_LANGUAGES(),
                "language_family".into(),
                ColumnType::VarChar,
            ))],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    TABLE_LANGUAGES(),
                    "num_of_letters".into(),
                    ColumnType::BigInt,
                )),
                alias: "SUM(languages.num_of_letters)".into(),
            }],
            "COUNT(Int64(1))".into(),
            TableExpr {
                table_ref: TABLE_LANGUAGES(),
            },
            DynProofExpr::new_column(ColumnRef::new(
                TABLE_LANGUAGES(),
                "uses_abjad".into(),
                ColumnType::Boolean,
            )),
        );

        let expected_postprocessing = SelectPostprocessing::new(vec![
            df_column("languages", "language_family"),
            Expr::Column(Column::new(
                None::<TableReference>,
                "COUNT(Int64(1))".to_string(),
            ))
            .mul(Expr::Literal(ScalarValue::Int64(Some(2))))
            .alias("twice_num_languages_using_abjad"),
            Expr::Column(Column::new(
                None::<TableReference>,
                "SUM(languages.num_of_letters)".to_string(),
            ))
            .alias("sum_num_of_letters"),
        ]);

        assert_eq!(result.plan(), &expected_plan);
        assert_eq!(result.postprocessing().unwrap(), &expected_postprocessing);
    }

    // Unsupported
    #[test]
    fn we_cannot_convert_unsupported_logical_plan_to_proof_plan_with_postprocessing() {
        let plan = LogicalPlan::Prepare(Prepare {
            name: "not_a_real_plan".to_string(),
            data_types: vec![],
            input: Arc::new(LogicalPlan::EmptyRelation(EmptyRelation {
                produce_one_row: false,
                schema: Arc::new(DFSchema::empty()),
            })),
        });
        let schemas = SCHEMAS();
        assert!(matches!(
            logical_plan_to_proof_plan_with_postprocessing(&plan, &schemas),
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }
}
