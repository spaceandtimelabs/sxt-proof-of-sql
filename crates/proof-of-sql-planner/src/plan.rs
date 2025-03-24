use super::{
    aggregate_function_to_proof_expr, column_to_column_ref, df_schema_to_column_fields,
    expr_to_proof_expr, table_reference_to_table_ref, AggregateFunc, PlannerError, PlannerResult,
};
use alloc::vec::Vec;
use datafusion::{
    common::DFSchema,
    logical_expr::{
        expr::Alias, Aggregate, Expr, Limit, LogicalPlan, Projection, TableScan, Union,
    },
    sql::{sqlparser::ast::Ident, TableReference},
};
use indexmap::IndexMap;
use proof_of_sql::{
    base::database::{ColumnRef, ColumnType, LiteralValue, TableRef},
    sql::{
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
        proof_plans::DynProofPlan,
    },
};

/// Get `AliasedDynProofExpr` from a `TableRef`, column indices for projection as well as
/// input and output schemas
///
/// Note that at least in the current implementation of `DataFusion`
/// the output schema should be a subset of the input schema
/// and that no aliasing should take place.
/// However that shouldn't be taken for granted.
fn get_aliased_dyn_proof_exprs(
    table_ref: &TableRef,
    projection: &[usize],
    input_schema: &DFSchema,
    output_schema: &DFSchema,
) -> PlannerResult<Vec<AliasedDynProofExpr>> {
    projection
        .iter()
        .enumerate()
        .map(
            |(output_index, input_index)| -> PlannerResult<AliasedDynProofExpr> {
                // Get output column name / alias
                let alias: Ident = output_schema.field(output_index).name().as_str().into();
                let input_column_name: Ident =
                    input_schema.field(*input_index).name().as_str().into();
                let data_type = input_schema.field(*input_index).data_type();
                let expr = DynProofExpr::new_column(ColumnRef::new(
                    table_ref.clone(),
                    input_column_name,
                    ColumnType::try_from(data_type.clone()).map_err(|_e| {
                        PlannerError::UnsupportedDataType {
                            data_type: data_type.clone(),
                        }
                    })?,
                ));
                Ok(AliasedDynProofExpr { expr, alias })
            },
        )
        .collect::<PlannerResult<Vec<_>>>()
}

/// Convert a `TableScan` without filters or fetch limit to a `DynProofPlan`
fn table_scan_to_projection(
    table_name: &TableReference,
    schemas: &IndexMap<TableReference, DFSchema>,
    projection: &[usize],
    projected_schema: &DFSchema,
) -> PlannerResult<DynProofPlan> {
    // Check if the table exists
    let table_ref = table_reference_to_table_ref(table_name)?;
    let input_schema = schemas
        .get(table_name)
        .ok_or_else(|| PlannerError::TableNotFound {
            table_name: table_name.to_string(),
        })?;
    // Get aliased expressions
    let aliased_dyn_proof_exprs =
        get_aliased_dyn_proof_exprs(&table_ref, projection, input_schema, projected_schema)?;
    let input_column_fields = df_schema_to_column_fields(input_schema)?;
    let table_exec = DynProofPlan::new_table(table_ref, input_column_fields);
    Ok(DynProofPlan::new_projection(
        aliased_dyn_proof_exprs,
        table_exec,
    ))
}

/// Convert a `TableScan` with filters but without fetch limit to a `DynProofPlan`
///
/// # Panics
/// Panics if there are no filters which should not happen if called from `logical_plan_to_proof_plan`
fn table_scan_to_filter(
    table_name: &TableReference,
    schemas: &IndexMap<TableReference, DFSchema>,
    projection: &[usize],
    projected_schema: &DFSchema,
    filters: &[Expr],
) -> PlannerResult<DynProofPlan> {
    // Check if the table exists
    let table_ref = table_reference_to_table_ref(table_name)?;
    let input_schema = schemas
        .get(table_name)
        .ok_or_else(|| PlannerError::TableNotFound {
            table_name: table_name.to_string(),
        })?;
    // Get aliased expressions
    let aliased_dyn_proof_exprs =
        get_aliased_dyn_proof_exprs(&table_ref, projection, input_schema, projected_schema)?;
    let table_expr = TableExpr { table_ref };
    // Filter
    let consolidated_filter_proof_expr = filters
        .iter()
        .map(|f| expr_to_proof_expr(f, input_schema))
        .reduce(|a, b| Ok(DynProofExpr::try_new_and(a?, b?)?))
        .expect("At least one filter expression is required")?;
    Ok(DynProofPlan::new_filter(
        aliased_dyn_proof_exprs,
        table_expr,
        consolidated_filter_proof_expr,
    ))
}

/// Converts a [`datafusion::logical_expr::Projection`] to a [`DynProofPlan`]
fn projection_to_proof_plan(
    expr: &[Expr],
    input: &LogicalPlan,
    output_schema: &DFSchema,
    schemas: &IndexMap<TableReference, DFSchema>,
) -> PlannerResult<DynProofPlan> {
    let input_plan = logical_plan_to_proof_plan(input, schemas)?;
    let input_schema = input.schema();
    let aliased_exprs = expr
        .iter()
        .zip(output_schema.fields().into_iter())
        .map(|(e, field)| -> PlannerResult<AliasedDynProofExpr> {
            let proof_expr = expr_to_proof_expr(e, input_schema)?;
            let alias = field.name().as_str().into();
            Ok(AliasedDynProofExpr {
                expr: proof_expr,
                alias,
            })
        })
        .collect::<PlannerResult<Vec<_>>>()?;
    Ok(DynProofPlan::new_projection(aliased_exprs, input_plan))
}

/// Convert a [`datafusion::logical_plan::LogicalPlan`] to a [`DynProofPlan`] for GROUP BYs
///
/// TODO: Improve how we handle GROUP BYs so that all the tech debt is resolved
///
/// # Panics
/// The code should never panic
fn aggregate_to_proof_plan(
    input: &LogicalPlan,
    group_expr: &[Expr],
    aggr_expr: &[Expr],
    schemas: &IndexMap<TableReference, DFSchema>,
    alias_map: &IndexMap<&str, &str>,
) -> PlannerResult<DynProofPlan> {
    // Check that all of `group_expr` are columns and get their names
    let group_columns = group_expr
        .iter()
        .map(|e| match e {
            Expr::Column(c) => Ok(c),
            _ => Err(PlannerError::UnsupportedLogicalPlan {
                plan: input.clone(),
            }),
        })
        .collect::<PlannerResult<Vec<_>>>()?;
    match input {
        // Only TableScan without fetch is supported
        LogicalPlan::TableScan(TableScan {
            table_name,
            filters,
            fetch: None,
            ..
        }) => {
            let table_ref = table_reference_to_table_ref(table_name)?;
            let input_schema =
                schemas
                    .get(table_name)
                    .ok_or_else(|| PlannerError::TableNotFound {
                        table_name: table_name.to_string(),
                    })?;
            let table_expr = TableExpr { table_ref };
            // Filter
            let consolidated_filter_proof_expr = filters
                .iter()
                .map(|f| expr_to_proof_expr(f, input_schema))
                .reduce(|a, b| Ok(DynProofExpr::try_new_and(a?, b?)?))
                .unwrap_or_else(|| Ok(DynProofExpr::new_literal(LiteralValue::Boolean(true))))?;
            // Aggregate
            // Prove that the ordering of `aggr_expr` is
            // 1. All group columns according to `group_columns`
            // 2. (Optional) All the SUMs
            // 3. COUNT
            if aggr_expr.is_empty() {
                return Err(PlannerError::UnsupportedLogicalPlan {
                    plan: input.clone(),
                });
            }
            let agg_aliased_proof_exprs: Vec<((AggregateFunc, DynProofExpr), Ident)> = aggr_expr
                .iter()
                .map(|e| match e.clone().unalias() {
                    Expr::AggregateFunction(agg) => {
                        let name_string = e.display_name()?;
                        let name = name_string.as_str();
                        let alias = alias_map.get(&name).ok_or_else(|| {
                            PlannerError::UnsupportedLogicalPlan {
                                plan: input.clone(),
                            }
                        })?;
                        Ok((
                            aggregate_function_to_proof_expr(&agg, input_schema)?,
                            (*alias).into(),
                        ))
                    }
                    _ => Err(PlannerError::UnsupportedLogicalPlan {
                        plan: input.clone(),
                    }),
                })
                .collect::<PlannerResult<Vec<_>>>()?;
            // Check that the last expression is COUNT and the rest are SUMs
            let (sum_tuples, count_tuple) =
                agg_aliased_proof_exprs.split_at(agg_aliased_proof_exprs.len() - 1);
            let sum_is_compliant = sum_tuples
                .iter()
                .all(|((op, _), _)| matches!(op, AggregateFunc::Sum));
            let count_is_compliant = count_tuple
                .iter()
                .all(|((op, _), _)| matches!(op, AggregateFunc::Count));
            if !sum_is_compliant || !count_is_compliant {
                return Err(PlannerError::UnsupportedLogicalPlan {
                    plan: input.clone(),
                });
            }
            let count_alias = agg_aliased_proof_exprs
                .last()
                .expect("We have already checked that this exists")
                .1
                .clone();
            // `group_by_exprs`
            let group_by_exprs = group_columns
                .iter()
                .map(|column| Ok(ColumnExpr::new(column_to_column_ref(column, input_schema)?)))
                .collect::<PlannerResult<Vec<_>>>()?;
            // `sum_expr`
            let sum_expr = sum_tuples
                .iter()
                .map(|((_, expr), alias)| AliasedDynProofExpr {
                    expr: expr.clone(),
                    alias: alias.clone(),
                })
                .collect::<Vec<_>>();
            Ok(DynProofPlan::new_group_by(
                group_by_exprs,
                sum_expr,
                count_alias,
                table_expr,
                consolidated_filter_proof_expr,
            ))
        }
        _ => Err(PlannerError::UnsupportedLogicalPlan {
            plan: input.clone(),
        }),
    }
}

/// Visit a [`datafusion::logical_plan::LogicalPlan`] and return a [`DynProofPlan`]
pub fn logical_plan_to_proof_plan(
    plan: &LogicalPlan,
    schemas: &IndexMap<TableReference, DFSchema>,
) -> PlannerResult<DynProofPlan> {
    match plan {
        LogicalPlan::EmptyRelation { .. } => Ok(DynProofPlan::new_empty()),
        // `projection` shouldn't be None in analyzed and optimized plans
        LogicalPlan::TableScan(TableScan {
            table_name,
            projection: Some(projection),
            projected_schema,
            filters,
            fetch,
            ..
        }) => {
            let base_plan = if filters.is_empty() {
                table_scan_to_projection(table_name, schemas, projection, projected_schema)
            } else {
                table_scan_to_filter(table_name, schemas, projection, projected_schema, filters)
            }?;
            if let Some(fetch) = fetch {
                Ok(DynProofPlan::new_slice(base_plan, 0, Some(*fetch)))
            } else {
                Ok(base_plan)
            }
        }
        // Aggregation
        LogicalPlan::Aggregate(Aggregate {
            input,
            group_expr,
            aggr_expr,
            schema,
            ..
        }) => {
            let name_strings = group_expr
                .iter()
                .chain(aggr_expr.iter())
                .map(Expr::display_name)
                .collect::<Result<Vec<_>, _>>()?;
            let alias_map = name_strings
                .iter()
                .zip(schema.fields().iter())
                .map(|(name_string, field)| {
                    let name = name_string.as_str();
                    let alias = field.name().as_str();
                    Ok((name, alias))
                })
                .collect::<PlannerResult<IndexMap<_, _>>>()?;
            aggregate_to_proof_plan(input, group_expr, aggr_expr, schemas, &alias_map)
        }
        // Projection
        LogicalPlan::Projection(Projection {
            input,
            expr,
            schema,
            ..
        }) => {
            match &**input {
                LogicalPlan::Aggregate(Aggregate {
                    input: agg_input,
                    group_expr,
                    aggr_expr,
                    ..
                }) => {
                    // Check whether the last layer is identity
                    let alias_map = expr
                        .iter()
                        .map(|e| match e {
                            Expr::Column(c) => Ok((c.name.as_str(), c.name.as_str())),
                            Expr::Alias(Alias { expr, name, .. }) => {
                                if let Expr::Column(c) = expr.as_ref() {
                                    Ok((c.name.as_str(), name.as_str()))
                                } else {
                                    Err(PlannerError::UnsupportedLogicalPlan { plan: plan.clone() })
                                }
                            }
                            _ => Err(PlannerError::UnsupportedLogicalPlan { plan: plan.clone() }),
                        })
                        .collect::<PlannerResult<IndexMap<_, _>>>()?;
                    aggregate_to_proof_plan(agg_input, group_expr, aggr_expr, schemas, &alias_map)
                }
                _ => projection_to_proof_plan(expr, input, schema, schemas),
            }
        }
        // Limit
        LogicalPlan::Limit(Limit { input, fetch, skip }) => {
            let input_plan = logical_plan_to_proof_plan(input, schemas)?;
            Ok(DynProofPlan::new_slice(input_plan, *skip, *fetch))
        }
        // Union
        LogicalPlan::Union(Union { inputs, schema }) => {
            let input_plans = inputs
                .iter()
                .map(|input| logical_plan_to_proof_plan(input, schemas))
                .collect::<PlannerResult<Vec<_>>>()?;
            let column_fields = df_schema_to_column_fields(schema)?;
            Ok(DynProofPlan::new_union(input_plans, column_fields))
        }
        _ => Err(PlannerError::UnsupportedLogicalPlan { plan: plan.clone() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{df_util::*, PoSqlTableSource};
    use alloc::{sync::Arc, vec};
    use arrow::datatypes::DataType;
    use core::ops::Add;
    use datafusion::{
        common::{Column, ScalarValue},
        logical_expr::{
            expr::{AggregateFunction, AggregateFunctionDefinition},
            not, BinaryExpr, EmptyRelation, Operator, Prepare, TableScan, TableSource,
        },
        physical_plan,
    };
    use indexmap::indexmap;
    use proof_of_sql::base::database::ColumnField;

    const SUM: AggregateFunctionDefinition =
        AggregateFunctionDefinition::BuiltIn(physical_plan::aggregates::AggregateFunction::Sum);
    const COUNT: AggregateFunctionDefinition =
        AggregateFunctionDefinition::BuiltIn(physical_plan::aggregates::AggregateFunction::Count);
    const AVG: AggregateFunctionDefinition =
        AggregateFunctionDefinition::BuiltIn(physical_plan::aggregates::AggregateFunction::Avg);

    #[expect(non_snake_case)]
    fn TABLE_REF_TABLE() -> TableRef {
        TableRef::from_names(None, "table")
    }

    #[expect(non_snake_case)]
    fn SCHEMAS() -> IndexMap<TableReference, DFSchema> {
        indexmap! {
            TableReference::from("table") => df_schema(
                "table",
                vec![
                    ("a", DataType::Int64),
                    ("b", DataType::Int32),
                    ("c", DataType::Utf8),
                    ("d", DataType::Boolean),
                ],
            ),
        }
    }

    #[expect(non_snake_case)]
    fn UNION_SCHEMAS() -> IndexMap<TableReference, DFSchema> {
        indexmap! {
            TableReference::from("table1") => df_schema(
                "table1",
                vec![
                    ("a1", DataType::Int64),
                    ("b1", DataType::Int32),
                ],
            ),
            TableReference::from("table2") => df_schema(
                "table2",
                vec![
                    ("a2", DataType::Int64),
                    ("b2", DataType::Int32),
                ],
            ),
            TableReference::from("schema.table3") => df_schema(
                "table3",
                vec![
                    ("a3", DataType::Int64),
                    ("b3", DataType::Int32),
                ],
            ),
        }
    }

    #[expect(non_snake_case)]
    fn EMPTY_SCHEMAS() -> IndexMap<TableReference, DFSchema> {
        indexmap! {}
    }

    #[expect(non_snake_case)]
    fn TABLE_SOURCE() -> Arc<dyn TableSource> {
        Arc::new(PoSqlTableSource::new(vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::Int),
            ColumnField::new("c".into(), ColumnType::VarChar),
            ColumnField::new("d".into(), ColumnType::Boolean),
        ]))
    }

    #[expect(non_snake_case)]
    fn ALIASED_A() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "a".into(),
                ColumnType::BigInt,
            )),
            alias: "a".into(),
        }
    }

    #[expect(non_snake_case)]
    fn ALIASED_B() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "b".into(),
                ColumnType::Int,
            )),
            alias: "b".into(),
        }
    }

    #[expect(non_snake_case)]
    fn ALIASED_C() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "c".into(),
                ColumnType::VarChar,
            )),
            alias: "c".into(),
        }
    }

    #[expect(non_snake_case)]
    fn ALIASED_D() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "d".into(),
                ColumnType::Boolean,
            )),
            alias: "d".into(),
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
    fn SUM_B() -> Expr {
        Expr::AggregateFunction(AggregateFunction {
            func_def: SUM,
            args: vec![df_column("table", "b")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        })
    }

    #[expect(non_snake_case)]
    fn SUM_D() -> Expr {
        Expr::AggregateFunction(AggregateFunction {
            func_def: SUM,
            args: vec![df_column("table", "d")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        })
    }

    // get_aliased_dyn_proof_exprs
    #[test]
    fn we_can_get_aliased_proof_expr_with_specified_projection_columns() {
        // Unused columns can be of unsupported types
        let table_ref = TABLE_REF_TABLE();
        let input_schema = df_schema(
            "table",
            vec![
                ("a", DataType::Int64),
                ("b", DataType::Int32),
                ("c", DataType::Utf8),
                ("d", DataType::Float32), // Unused column
            ],
        );
        let output_schema = df_schema("table", vec![("b", DataType::Int32), ("c", DataType::Utf8)]);
        let result =
            get_aliased_dyn_proof_exprs(&table_ref, &[1, 2], &input_schema, &output_schema)
                .unwrap();
        let expected = vec![ALIASED_B(), ALIASED_C()];
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_aliased_proof_expr_without_specified_projection_columns() {
        let table_ref = TABLE_REF_TABLE();
        let input_schema = df_schema(
            "table",
            vec![
                ("a", DataType::Int64),
                ("b", DataType::Int32),
                ("c", DataType::Utf8),
                ("d", DataType::Boolean),
            ],
        );
        let output_schema = df_schema(
            "table",
            vec![
                ("a", DataType::Int64),
                ("b", DataType::Int32),
                ("c", DataType::Utf8),
                ("d", DataType::Boolean),
            ],
        );
        let result =
            get_aliased_dyn_proof_exprs(&table_ref, &[0, 1, 2, 3], &input_schema, &output_schema)
                .unwrap();
        let expected = vec![ALIASED_A(), ALIASED_B(), ALIASED_C(), ALIASED_D()];
        assert_eq!(result, expected);
    }

    #[test]
    fn we_cannot_get_aliased_proof_expr_if_unsupported_data_types_are_included() {
        let table_ref = TABLE_REF_TABLE();
        let input_schema = df_schema(
            "table",
            vec![
                ("a", DataType::Int64),
                ("b", DataType::Float64),
                ("c", DataType::Utf8),
                ("d", DataType::Boolean),
            ],
        );
        let output_schema = df_schema(
            "table",
            vec![
                ("b", DataType::Float64),
                ("c", DataType::Utf8),
                ("d", DataType::Date32),
            ],
        );
        assert!(matches!(
            get_aliased_dyn_proof_exprs(&table_ref, &[1, 2, 3], &input_schema, &output_schema,),
            Err(PlannerError::UnsupportedDataType { .. })
        ));
    }

    // aggregate_to_proof_plan
    #[test]
    fn we_can_aggregate_with_group_by_and_sum_count() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions (must follow the pattern: group columns, then SUMs, then COUNT)
        let aggr_expr = vec![
            SUM_B(),   // SUM
            COUNT_1(), // COUNT
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "SUM(table.b)" => "sum_b",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        // Expected result
        let expected = DynProofPlan::new_group_by(
            vec![ColumnExpr::new(ColumnRef::new(
                TABLE_REF_TABLE(),
                "a".into(),
                ColumnType::BigInt,
            ))],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "b".into(),
                    ColumnType::Int,
                )),
                alias: "sum_b".into(),
            }],
            "count_1".into(),
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_aggregate_with_filters() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            SUM_B(),   // SUM
            COUNT_1(), // COUNT
        ];

        // Create filters
        let filter_exprs = vec![
            df_column("table", "d"), // Boolean column as filter
        ];

        // Create the input plan with filters
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                filter_exprs,
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "SUM(table.b)" => "sum_b",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        // Expected result
        let expected = DynProofPlan::new_group_by(
            vec![ColumnExpr::new(ColumnRef::new(
                TABLE_REF_TABLE(),
                "a".into(),
                ColumnType::BigInt,
            ))],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "b".into(),
                    ColumnType::Int,
                )),
                alias: "sum_b".into(),
            }],
            "count_1".into(),
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "d".into(),
                ColumnType::Boolean,
            )),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_aggregate_with_multiple_group_columns() {
        // Setup group expressions
        let group_expr = vec![df_column("table", "a"), df_column("table", "c")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            SUM_B(),   // SUM
            COUNT_1(), // COUNT
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "c" => "c",
            "SUM(table.b)" => "sum_b",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        // Expected result
        let expected = DynProofPlan::new_group_by(
            vec![
                ColumnExpr::new(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "a".into(),
                    ColumnType::BigInt,
                )),
                ColumnExpr::new(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "c".into(),
                    ColumnType::VarChar,
                )),
            ],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "b".into(),
                    ColumnType::Int,
                )),
                alias: "sum_b".into(),
            }],
            "count_1".into(),
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_aggregate_with_multiple_sum_expressions() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            SUM_B(),   // First SUM
            SUM_D(),   // Second SUM
            COUNT_1(), // COUNT
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "SUM(table.b)" => "sum_b",
            "SUM(table.d)" => "sum_d",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        // Expected result
        let expected = DynProofPlan::new_group_by(
            vec![ColumnExpr::new(ColumnRef::new(
                TABLE_REF_TABLE(),
                "a".into(),
                ColumnType::BigInt,
            ))],
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "b".into(),
                        ColumnType::Int,
                    )),
                    alias: "sum_b".into(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "d".into(),
                        ColumnType::Boolean,
                    )),
                    alias: "sum_d".into(),
                },
            ],
            "count_1".into(),
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_aggregate_without_sum_expressions() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            COUNT_1(), // COUNT
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        // Expected result
        let expected = DynProofPlan::new_group_by(
            vec![ColumnExpr::new(ColumnRef::new(
                TABLE_REF_TABLE(),
                "a".into(),
                ColumnType::BigInt,
            ))],
            vec![], // No SUMs
            "count_1".into(),
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        );

        assert_eq!(result, expected);
    }

    // Error case tests
    #[test]
    fn we_cannot_aggregate_with_non_column_group_expr() {
        // Setup group expression with a non-column expression
        let group_expr = vec![Expr::BinaryExpr(BinaryExpr::new(
            Box::new(df_column("table", "a")),
            Operator::Plus,
            Box::new(df_column("table", "b")),
        ))];

        // Create the aggregate expressions
        let aggr_expr = vec![
            Expr::BinaryExpr(BinaryExpr::new(
                Box::new(df_column("table", "a")),
                Operator::Plus,
                Box::new(df_column("table", "b")),
            )),
            COUNT_1(),
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a+b" => "res",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(matches!(
            result,
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }

    #[test]
    fn we_cannot_aggregate_with_non_aggregate_expression() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Setup a non-aggregate expression
        let non_agg_expr = Expr::BinaryExpr(BinaryExpr::new(
            Box::new(df_column("table", "b")),
            Operator::Plus,
            Box::new(df_column("table", "c")),
        ));

        // Setup aliased expression
        let aliased_non_agg = Expr::Alias(Alias {
            expr: Box::new(non_agg_expr),
            relation: None,
            name: "b_plus_c".to_string(),
        });

        // Create the aggregate expressions
        let aggr_expr = vec![
            aliased_non_agg, // Non-aggregate expression
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "b+c" => "b_plus_c",
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(matches!(
            result,
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }

    #[test]
    fn we_cannot_aggregate_with_non_sum_aggregate_function() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Setup a non-SUM aggregate function (e.g., Avg)
        let avg_expr = Expr::AggregateFunction(AggregateFunction {
            func_def: AVG,
            args: vec![df_column("table", "b")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        });

        // Setup aliased expressions
        let aliased_avg = Expr::Alias(Alias {
            expr: Box::new(avg_expr),
            relation: None,
            name: "avg_b".to_string(),
        });

        // Create the aggregate expressions
        let aggr_expr = vec![
            aliased_avg, // AVG aggregate (not SUM)
            COUNT_1(),   // COUNT
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "AVG(table.b)" => "avg_b",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(matches!(
            result,
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }

    #[test]
    fn we_cannot_aggregate_with_non_count_last_aggregate() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Setup SUM aggregates
        let sum_expr1 = Expr::AggregateFunction(AggregateFunction {
            func_def: SUM,
            args: vec![df_column("table", "b")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        });

        let sum_expr2 = Expr::AggregateFunction(AggregateFunction {
            func_def: SUM,
            args: vec![df_column("table", "c")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        });

        // Setup aliased expressions
        let aliased_sum1 = Expr::Alias(Alias {
            expr: Box::new(sum_expr1),
            relation: None,
            name: "sum_b".to_string(),
        });

        let aliased_sum2 = Expr::Alias(Alias {
            expr: Box::new(sum_expr2),
            relation: None,
            name: "sum_c".to_string(),
        });

        // Create the aggregate expressions with no COUNT at the end
        let aggr_expr = vec![
            aliased_sum1, // SUM
            aliased_sum2, // Another SUM (should be COUNT)
        ];

        // Create the input plan
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "SUM(table.b)" => "sum_b",
            "SUM(c)" => "sum_c",
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(matches!(
            result,
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }

    #[test]
    fn we_cannot_aggregate_with_fetch_limit() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            COUNT_1(), // COUNT
        ];

        // Create the input plan with fetch limit
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                Some(10),
            )
            .unwrap(),
        );
        let alias_map = indexmap! {
            "a" => "a",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function - should return an error because fetch limit is not supported
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(matches!(
            result,
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }

    #[test]
    fn we_cannot_aggregate_with_non_table_scan_input() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            COUNT_1(), // COUNT
        ];

        // Create a non-TableScan input plan
        let input_plan = LogicalPlan::EmptyRelation(EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(DFSchema::empty()),
        });
        let alias_map = indexmap! {
            "a" => "a",
            "COUNT(Int64(1))" => "count_1",
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(matches!(
            result,
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }

    // EmptyRelation
    #[test]
    fn we_can_convert_empty_plan_to_proof_plan() {
        let empty_plan = LogicalPlan::EmptyRelation(EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(DFSchema::empty()),
        });
        let schemas = indexmap! {};
        let result = logical_plan_to_proof_plan(&empty_plan, &schemas).unwrap();
        assert_eq!(result, DynProofPlan::new_empty());
    }

    // TableScan
    #[test]
    fn we_can_convert_table_scan_plan_to_proof_plan_without_filter_or_fetch_limit() {
        let plan = LogicalPlan::TableScan(
            TableScan::try_new("table", TABLE_SOURCE(), Some(vec![0, 1, 2]), vec![], None).unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_projection(
            vec![ALIASED_A(), ALIASED_B(), ALIASED_C()],
            DynProofPlan::new_table(
                TABLE_REF_TABLE(),
                vec![
                    ColumnField::new("a".into(), ColumnType::BigInt),
                    ColumnField::new("b".into(), ColumnType::Int),
                    ColumnField::new("c".into(), ColumnType::VarChar),
                    ColumnField::new("d".into(), ColumnType::Boolean),
                ],
            ),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn we_cannot_convert_table_scan_plan_to_proof_plan_without_filter_or_fetch_limit_if_bad_schemas(
    ) {
        let plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                None,
            )
            .unwrap(),
        );
        let schemas = EMPTY_SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas);
        assert!(matches!(result, Err(PlannerError::TableNotFound { .. })));
    }

    #[test]
    fn we_can_convert_table_scan_plan_to_proof_plan_with_filter_but_without_fetch_limit() {
        let filter_exprs = vec![
            df_column("table", "a").eq(df_column("table", "b")),
            df_column("table", "d"),
        ];
        let plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 2]),
                filter_exprs,
                None,
            )
            .unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_filter(
            vec![ALIASED_A(), ALIASED_C()],
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::try_new_and(
                DynProofExpr::try_new_equals(
                    DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "a".into(),
                        ColumnType::BigInt,
                    )),
                    DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "b".into(),
                        ColumnType::Int,
                    )),
                )
                .unwrap(),
                DynProofExpr::new_column(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "d".into(),
                    ColumnType::Boolean,
                )),
            )
            .unwrap(),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn we_cannot_convert_table_scan_plan_to_proof_plan_with_filter_but_without_fetch_limit_if_bad_schemas(
    ) {
        let filter_exprs = vec![
            df_column("table", "a").eq(df_column("table", "b")),
            df_column("table", "d"),
        ];
        let plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 2]),
                filter_exprs,
                None,
            )
            .unwrap(),
        );
        let schemas = EMPTY_SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas);
        assert!(matches!(result, Err(PlannerError::TableNotFound { .. })));
    }

    #[test]
    fn we_can_convert_table_scan_plan_to_proof_plan_without_filter_but_with_fetch_limit() {
        let plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                vec![],
                Some(2),
            )
            .unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_slice(
            DynProofPlan::new_projection(
                vec![ALIASED_A(), ALIASED_B(), ALIASED_C(), ALIASED_D()],
                DynProofPlan::new_table(
                    TABLE_REF_TABLE(),
                    vec![
                        ColumnField::new("a".into(), ColumnType::BigInt),
                        ColumnField::new("b".into(), ColumnType::Int),
                        ColumnField::new("c".into(), ColumnType::VarChar),
                        ColumnField::new("d".into(), ColumnType::Boolean),
                    ],
                ),
            ),
            0,
            Some(2),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_convert_table_scan_plan_to_proof_plan_with_filter_and_fetch_limit() {
        let filter_exprs = vec![
            df_column("table", "a").gt(df_column("table", "b")),
            df_column("table", "d"),
        ];
        let plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 3]),
                filter_exprs,
                Some(5),
            )
            .unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_slice(
            DynProofPlan::new_filter(
                vec![ALIASED_A(), ALIASED_D()],
                TableExpr {
                    table_ref: TABLE_REF_TABLE(),
                },
                DynProofExpr::try_new_and(
                    DynProofExpr::try_new_inequality(
                        DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "a".into(),
                            ColumnType::BigInt,
                        )),
                        DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "b".into(),
                            ColumnType::Int,
                        )),
                        false,
                    )
                    .unwrap(),
                    DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "d".into(),
                        ColumnType::Boolean,
                    )),
                )
                .unwrap(),
            ),
            0,
            Some(5),
        );
        assert_eq!(result, expected);
    }

    // Projection
    #[test]
    fn we_can_convert_projection_plan_to_proof_plan() {
        let plan = LogicalPlan::Projection(
            Projection::try_new(
                vec![
                    Expr::BinaryExpr(BinaryExpr::new(
                        Box::new(df_column("table", "a")),
                        Operator::Plus,
                        Box::new(df_column("table", "b")),
                    )),
                    not(df_column("table", "d")),
                ],
                Arc::new(LogicalPlan::TableScan(
                    TableScan::try_new("table", TABLE_SOURCE(), Some(vec![0, 1, 3]), vec![], None)
                        .unwrap(),
                )),
            )
            .unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_projection(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::try_new_add(
                        DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "a".into(),
                            ColumnType::BigInt,
                        )),
                        DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "b".into(),
                            ColumnType::Int,
                        )),
                    )
                    .unwrap(),
                    alias: "table.a + table.b".into(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::try_new_not(DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "d".into(),
                        ColumnType::Boolean,
                    )))
                    .unwrap(),
                    alias: "NOT table.d".into(),
                },
            ],
            DynProofPlan::new_projection(
                vec![ALIASED_A(), ALIASED_B(), ALIASED_D()],
                DynProofPlan::new_table(
                    TABLE_REF_TABLE(),
                    vec![
                        ColumnField::new("a".into(), ColumnType::BigInt),
                        ColumnField::new("b".into(), ColumnType::Int),
                        ColumnField::new("c".into(), ColumnType::VarChar),
                        ColumnField::new("d".into(), ColumnType::Boolean),
                    ],
                ),
            ),
        );
        assert_eq!(result, expected);
    }

    // Limit
    // Note that either fetch or skip will exist or optimizer will remove the Limit node
    #[test]
    fn we_can_convert_limit_plan_with_fetch_and_skip_to_proof_plan() {
        let plan = LogicalPlan::Limit(Limit {
            input: Arc::new(LogicalPlan::TableScan(
                TableScan::try_new(
                    "table",
                    TABLE_SOURCE(),
                    Some(vec![0, 1]),
                    vec![],
                    // Optimizer will put a fetch on TableScan if there is a non-empty fetch in an outer Limit
                    Some(5),
                )
                .unwrap(),
            )),
            fetch: Some(3),
            skip: 2,
        });
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_slice(
            DynProofPlan::new_slice(
                DynProofPlan::new_projection(
                    vec![ALIASED_A(), ALIASED_B()],
                    DynProofPlan::new_table(
                        TABLE_REF_TABLE(),
                        vec![
                            ColumnField::new("a".into(), ColumnType::BigInt),
                            ColumnField::new("b".into(), ColumnType::Int),
                            ColumnField::new("c".into(), ColumnType::VarChar),
                            ColumnField::new("d".into(), ColumnType::Boolean),
                        ],
                    ),
                ),
                0,
                Some(5),
            ),
            2,
            Some(3),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_convert_limit_plan_with_fetch_no_skip_to_proof_plan() {
        //TODO: Optimize proof plan to remove redundant slices
        let plan = LogicalPlan::Limit(Limit {
            input: Arc::new(LogicalPlan::TableScan(
                TableScan::try_new("table", TABLE_SOURCE(), Some(vec![0, 1]), vec![], Some(3))
                    .unwrap(),
            )),
            fetch: Some(3),
            skip: 0,
        });

        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();

        let expected = DynProofPlan::new_slice(
            DynProofPlan::new_slice(
                DynProofPlan::new_projection(
                    vec![ALIASED_A(), ALIASED_B()],
                    DynProofPlan::new_table(
                        TABLE_REF_TABLE(),
                        vec![
                            ColumnField::new("a".into(), ColumnType::BigInt),
                            ColumnField::new("b".into(), ColumnType::Int),
                            ColumnField::new("c".into(), ColumnType::VarChar),
                            ColumnField::new("d".into(), ColumnType::Boolean),
                        ],
                    ),
                ),
                0,
                Some(3),
            ),
            0,
            Some(3),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_convert_limit_plan_with_skip_no_fetch_to_proof_plan() {
        let plan = LogicalPlan::Limit(Limit {
            input: Arc::new(LogicalPlan::TableScan(
                TableScan::try_new("table", TABLE_SOURCE(), Some(vec![0, 1]), vec![], None)
                    .unwrap(),
            )),
            fetch: None,
            skip: 2,
        });

        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();

        let expected = DynProofPlan::new_slice(
            DynProofPlan::new_projection(
                vec![ALIASED_A(), ALIASED_B()],
                DynProofPlan::new_table(
                    TABLE_REF_TABLE(),
                    vec![
                        ColumnField::new("a".into(), ColumnType::BigInt),
                        ColumnField::new("b".into(), ColumnType::Int),
                        ColumnField::new("c".into(), ColumnType::VarChar),
                        ColumnField::new("d".into(), ColumnType::Boolean),
                    ],
                ),
            ),
            2,
            None,
        );
        assert_eq!(result, expected);
    }

    // Union
    #[expect(clippy::too_many_lines)]
    #[test]
    fn we_can_convert_union_plan_to_proof_plan() {
        let plan = LogicalPlan::Union(Union {
            schema: Arc::new(df_schema(
                "table",
                vec![("a", DataType::Int64), ("b", DataType::Int32)],
            )),
            inputs: vec![
                Arc::new(LogicalPlan::TableScan(
                    TableScan::try_new("table1", TABLE_SOURCE(), Some(vec![0, 1]), vec![], None)
                        .unwrap(),
                )),
                Arc::new(LogicalPlan::TableScan(
                    TableScan::try_new("table2", TABLE_SOURCE(), Some(vec![0, 1]), vec![], None)
                        .unwrap(),
                )),
                Arc::new(LogicalPlan::TableScan(
                    TableScan::try_new(
                        "schema.table3",
                        TABLE_SOURCE(),
                        Some(vec![0, 1]),
                        vec![],
                        None,
                    )
                    .unwrap(),
                )),
            ],
        });
        let schemas = UNION_SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_union(
            vec![
                DynProofPlan::new_projection(
                    vec![
                        AliasedDynProofExpr {
                            expr: DynProofExpr::new_column(ColumnRef::new(
                                TableRef::from_names(None, "table1"),
                                "a1".into(),
                                ColumnType::BigInt,
                            )),
                            alias: "a".into(),
                        },
                        AliasedDynProofExpr {
                            expr: DynProofExpr::new_column(ColumnRef::new(
                                TableRef::from_names(None, "table1"),
                                "b1".into(),
                                ColumnType::Int,
                            )),
                            alias: "b".into(),
                        },
                    ],
                    DynProofPlan::new_table(
                        TableRef::from_names(None, "table1"),
                        vec![
                            ColumnField::new("a1".into(), ColumnType::BigInt),
                            ColumnField::new("b1".into(), ColumnType::Int),
                        ],
                    ),
                ),
                DynProofPlan::new_projection(
                    vec![
                        AliasedDynProofExpr {
                            expr: DynProofExpr::new_column(ColumnRef::new(
                                TableRef::from_names(None, "table2"),
                                "a2".into(),
                                ColumnType::BigInt,
                            )),
                            alias: "a".into(),
                        },
                        AliasedDynProofExpr {
                            expr: DynProofExpr::new_column(ColumnRef::new(
                                TableRef::from_names(None, "table2"),
                                "b2".into(),
                                ColumnType::Int,
                            )),
                            alias: "b".into(),
                        },
                    ],
                    DynProofPlan::new_table(
                        TableRef::from_names(None, "table2"),
                        vec![
                            ColumnField::new("a2".into(), ColumnType::BigInt),
                            ColumnField::new("b2".into(), ColumnType::Int),
                        ],
                    ),
                ),
                DynProofPlan::new_projection(
                    vec![
                        AliasedDynProofExpr {
                            expr: DynProofExpr::new_column(ColumnRef::new(
                                TableRef::from_names(Some("schema"), "table3"),
                                "a3".into(),
                                ColumnType::BigInt,
                            )),
                            alias: "a".into(),
                        },
                        AliasedDynProofExpr {
                            expr: DynProofExpr::new_column(ColumnRef::new(
                                TableRef::from_names(Some("schema"), "table3"),
                                "b3".into(),
                                ColumnType::Int,
                            )),
                            alias: "b".into(),
                        },
                    ],
                    DynProofPlan::new_table(
                        TableRef::from_names(Some("schema"), "table3"),
                        vec![
                            ColumnField::new("a3".into(), ColumnType::BigInt),
                            ColumnField::new("b3".into(), ColumnType::Int),
                        ],
                    ),
                ),
            ],
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::Int),
            ],
        );
        assert_eq!(result, expected);
    }

    // Aggregate
    #[test]
    fn we_can_convert_supported_simple_agg_plan_to_proof_plan() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            SUM_B(),   // SUM
            COUNT_1(), // COUNT
        ];

        // Create filters
        let filter_exprs = vec![
            df_column("table", "d"), // Boolean column as filter
        ];

        // Create the input plan with filters
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
                filter_exprs,
                None,
            )
            .unwrap(),
        );

        let agg_plan = LogicalPlan::Aggregate(
            Aggregate::try_new(Arc::new(input_plan), group_expr.clone(), aggr_expr.clone())
                .unwrap(),
        );

        // Test the function
        let result = logical_plan_to_proof_plan(&agg_plan, &SCHEMAS()).unwrap();

        // Expected result
        let expected = DynProofPlan::new_group_by(
            vec![ColumnExpr::new(ColumnRef::new(
                TABLE_REF_TABLE(),
                "a".into(),
                ColumnType::BigInt,
            ))],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "b".into(),
                    ColumnType::Int,
                )),
                alias: "SUM(table.b)".into(),
            }],
            "COUNT(Int64(1))".into(),
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "d".into(),
                ColumnType::Boolean,
            )),
        );

        assert_eq!(result, expected);
    }

    // Aggregate + Projection
    #[test]
    fn we_can_convert_supported_agg_plan_to_proof_plan() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            SUM_B(),   // SUM
            COUNT_1(), // COUNT
        ];

        // Create filters
        let filter_exprs = vec![
            df_column("table", "d"), // Boolean column as filter
        ];

        // Create the input plan with filters
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
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
                    df_column("table", "a"),
                    Expr::Column(Column::new(
                        None::<TableReference>,
                        "SUM(table.b)".to_string(),
                    ))
                    .alias("sum_b"),
                    Expr::Column(Column::new(
                        None::<TableReference>,
                        "COUNT(Int64(1))".to_string(),
                    ))
                    .alias("count_1"),
                ],
                Arc::new(agg_plan),
            )
            .unwrap(),
        );

        // Test the function
        let result = logical_plan_to_proof_plan(&proj_plan, &SCHEMAS()).unwrap();

        // Expected result
        let expected = DynProofPlan::new_group_by(
            vec![ColumnExpr::new(ColumnRef::new(
                TABLE_REF_TABLE(),
                "a".into(),
                ColumnType::BigInt,
            ))],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "b".into(),
                    ColumnType::Int,
                )),
                alias: "sum_b".into(),
            }],
            "count_1".into(),
            TableExpr {
                table_ref: TABLE_REF_TABLE(),
            },
            DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "d".into(),
                ColumnType::Boolean,
            )),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn we_cannot_convert_unsupported_agg_plan_to_proof_plan() {
        // Setup group expression
        let group_expr = vec![df_column("table", "a")];

        // Create the aggregate expressions
        let aggr_expr = vec![
            SUM_B(),   // SUM
            COUNT_1(), // COUNT
        ];

        // Create filters
        let filter_exprs = vec![
            df_column("table", "d"), // Boolean column as filter
        ];

        // Create the input plan with filters
        let input_plan = LogicalPlan::TableScan(
            TableScan::try_new(
                "table",
                TABLE_SOURCE(),
                Some(vec![0, 1, 2, 3]),
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
                vec![df_column("table", "a").add(df_column("table", "a"))],
                Arc::new(agg_plan),
            )
            .unwrap(),
        );

        // Test the function
        assert!(matches!(
            logical_plan_to_proof_plan(&proj_plan, &SCHEMAS()),
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }

    // Unsupported
    #[test]
    fn we_cannot_convert_unsupported_logical_plan_to_proof_plan() {
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
            logical_plan_to_proof_plan(&plan, &schemas),
            Err(PlannerError::UnsupportedLogicalPlan { .. })
        ));
    }
}
