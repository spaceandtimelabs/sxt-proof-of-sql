use super::{
    aggregate_function_to_proof_expr, expr_to_proof_expr, get_column_idents_from_expr,
    table_reference_to_table_ref, AggregateFunc, AggregatePlanError, JoinPlanError,
    LogicalPlanNodeKind, PlannerError, PlannerResult,
};
use alloc::vec::Vec;
use datafusion::{
    common::{DFSchema, JoinConstraint, JoinType},
    logical_expr::{
        Aggregate, Expr, Filter, Join, Limit, LogicalPlan, Projection, SubqueryAlias, TableScan,
        Union,
    },
    sql::{sqlparser::ast::Ident, TableReference},
};
use indexmap::{IndexMap, IndexSet};
use proof_of_sql::{
    base::database::{ColumnField, ColumnRef, ColumnType, LiteralValue, SchemaAccessor, TableRef},
    sql::{
        proof::ProofPlan,
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, ProofExpr},
        proof_plans::{DynProofPlan, SortMergeJoinExec},
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
    input_schema: &[(Ident, ColumnType)],
    output_schema: &DFSchema,
) -> PlannerResult<Vec<AliasedDynProofExpr>> {
    projection
        .iter()
        .enumerate()
        .map(
            |(output_index, input_index)| -> PlannerResult<AliasedDynProofExpr> {
                // Get output column name / alias
                let alias: Ident = output_schema.field(output_index).name().as_str().into();
                let (input_column_name, data_type) = input_schema
                    .get(*input_index)
                    .ok_or(PlannerError::ColumnNotFound)?;
                let expr = DynProofExpr::new_column(ColumnRef::new(
                    table_ref.clone(),
                    input_column_name.clone(),
                    *data_type,
                ));
                Ok(AliasedDynProofExpr { expr, alias })
            },
        )
        .collect::<PlannerResult<Vec<_>>>()
}

/// Get column identifiers needed in a `TableScan` given its projection and filters
fn table_scan_get_required_columns(
    projection: &[usize],
    filters: &[Expr],
    input_schema: &[(Ident, ColumnType)],
) -> IndexSet<Ident> {
    projection
        .iter()
        .filter_map(|&i| input_schema.get(i).map(|(ident, _)| ident.clone()))
        .chain(filters.iter().flat_map(get_column_idents_from_expr))
        .collect()
}

/// Convert a `TableScan` without filters or fetch limit to a `DynProofPlan`
///
/// # Panics
/// Panics if `schemas` is incorrect. Otherwise it should never happen.
fn table_scan_to_proof_plan(
    table_name: &TableReference,
    schemas: &impl SchemaAccessor,
    projection: &[usize],
) -> PlannerResult<DynProofPlan> {
    // Check if the table exists
    let table_ref = table_reference_to_table_ref(table_name)?;
    let input_schema = schemas.lookup_schema(&table_ref);
    // Filter for only the fields we use
    let input_column_fields = projection
        .iter()
        .map(|i| {
            let (ident, column_type) = input_schema
                .get(*i)
                .expect("Projection index out of bounds");
            ColumnField::new(ident.clone(), *column_type)
        })
        .collect::<Vec<_>>();
    Ok(DynProofPlan::new_table(table_ref, input_column_fields))
}

/// Convert a `TableScan` with filters but without fetch limit to a `DynProofPlan`
///
/// # Panics
/// Panics if there are no filters which should not happen if called from `logical_plan_to_proof_plan`
fn table_scan_to_filter(
    table_name: &TableReference,
    schemas: &impl SchemaAccessor,
    projection: &[usize],
    projected_schema: &DFSchema,
    filters: &[Expr],
) -> PlannerResult<DynProofPlan> {
    // Check if the table exists
    let table_ref = table_reference_to_table_ref(table_name)?;
    let input_schema = schemas.lookup_schema(&table_ref);
    // Get aliased expressions
    let aliased_dyn_proof_exprs =
        get_aliased_dyn_proof_exprs(&table_ref, projection, &input_schema, projected_schema)?;
    let required_columns = table_scan_get_required_columns(projection, filters, &input_schema);
    let input_column_fields = input_schema
        .iter()
        .filter(|(ident, _)| required_columns.contains(ident))
        .map(|(ident, column_type)| ColumnField::new(ident.clone(), *column_type))
        .collect::<Vec<_>>();
    let table_exec = DynProofPlan::new_table(table_ref, input_column_fields);
    let filter_proof_exprs = filters
        .iter()
        .map(|f| expr_to_proof_expr(f, &input_schema))
        .reduce(|a, b| Ok(DynProofExpr::try_new_and(a?, b?)?))
        .expect("At least one filter expression is required")?;
    Ok(DynProofPlan::new_filter(
        aliased_dyn_proof_exprs,
        table_exec,
        filter_proof_exprs,
    ))
}

/// Converts a [`datafusion::logical_expr::Projection`] to a [`DynProofPlan`]
fn projection_to_proof_plan(
    expr: &[Expr],
    input: &LogicalPlan,
    output_schema: &DFSchema,
    schemas: &impl SchemaAccessor,
) -> PlannerResult<DynProofPlan> {
    let input_plan = logical_plan_to_proof_plan(input, schemas)?;
    let input_schema = input_plan
        .get_column_result_fields()
        .iter()
        .map(|field| (field.name(), field.data_type()))
        .collect::<Vec<_>>();
    let aliased_exprs = expr
        .iter()
        .zip(output_schema.fields().iter())
        .map(|(e, field)| -> PlannerResult<AliasedDynProofExpr> {
            let proof_expr = expr_to_proof_expr(e, &input_schema)?;
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
#[expect(clippy::too_many_lines)]
fn aggregate_to_proof_plan(
    input: &LogicalPlan,
    group_expr: &[Expr],
    aggr_expr: &[Expr],
    schemas: &impl SchemaAccessor,
    alias_map: &IndexMap<String, String>,
) -> PlannerResult<DynProofPlan> {
    let input_plan = logical_plan_to_proof_plan(input, schemas)?;
    let input_schema = input_plan
        .get_column_result_fields()
        .iter()
        .map(|field| (field.name(), field.data_type()))
        .collect::<Vec<_>>();
    let dummy_table_ref = TableRef::from_names(None, "");
    // We keep an extra alias just in case there is no count in the aggregate expr list
    let mut inner_aliases = 0..=(group_expr.len() + aggr_expr.len());
    let (inner_group_by_exprs, group_by_exprs): (Vec<_>, Vec<_>) = group_expr
        .iter()
        .zip(&mut inner_aliases)
        .map(|(e, aggregate_alias)| -> PlannerResult<_> {
            let aggregate_alias: Ident = aggregate_alias.to_string().as_str().into();
            let aggregate_proof_expr = expr_to_proof_expr(e, &input_schema)?;
            let name_string = e.clone().unalias().display_name()?;
            let alias = alias_map.get(&name_string).ok_or(
                AggregatePlanError::MissingGroupExpressionAlias {
                    expression: name_string,
                },
            )?;
            let proof_expr = DynProofExpr::new_column(ColumnRef::new(
                dummy_table_ref.clone(),
                aggregate_alias.clone(),
                aggregate_proof_expr.data_type(),
            ));
            Ok((
                AliasedDynProofExpr {
                    expr: aggregate_proof_expr,
                    alias: aggregate_alias,
                },
                AliasedDynProofExpr {
                    expr: proof_expr,
                    alias: alias.as_str().into(),
                },
            ))
        })
        .collect::<PlannerResult<Vec<_>>>()?
        .into_iter()
        .unzip();
    let agg_aliased_proof_exprs: Vec<((AggregateFunc, DynProofExpr), Ident)> = aggr_expr
        .iter()
        .map(|e| {
            let expr = e.clone().unalias();
            match &expr {
                Expr::AggregateFunction(agg) => {
                    let name_string = expr.display_name()?;
                    let alias = alias_map.get(&name_string).ok_or(
                        AggregatePlanError::MissingAggregateExpressionAlias {
                            expression: name_string,
                        },
                    )?;
                    Ok((
                        aggregate_function_to_proof_expr(agg, &input_schema)?,
                        alias.as_str().into(),
                    ))
                }
                _ => Err(AggregatePlanError::UnexpectedAggregateExpression {
                    expression: expr.to_string(),
                }
                .into()),
            }
        })
        .collect::<PlannerResult<Vec<_>>>()?;
    let sum_tuples: Vec<_> = agg_aliased_proof_exprs
        .iter()
        .filter_map(|((a, expr), alias)| matches!(a, AggregateFunc::Sum).then_some((expr, alias)))
        .collect();
    let count_aliases: Vec<_> = agg_aliased_proof_exprs
        .iter()
        .filter_map(|((a, _), alias)| matches!(a, AggregateFunc::Count).then_some(alias))
        .collect();
    let (inner_sum_expr, sum_exprs): (Vec<_>, Vec<_>) = sum_tuples
        .into_iter()
        .zip(&mut inner_aliases)
        .map(|((expr, alias), inner_alias)| {
            let inner_alias: Ident = inner_alias.to_string().as_str().into();
            let proof_expr = DynProofExpr::new_column(ColumnRef::new(
                dummy_table_ref.clone(),
                inner_alias.clone(),
                expr.data_type(),
            ));
            (
                AliasedDynProofExpr {
                    expr: expr.clone(),
                    alias: inner_alias,
                },
                AliasedDynProofExpr {
                    expr: proof_expr,
                    alias: alias.clone(),
                },
            )
        })
        .unzip();
    let inner_count_alias: Ident = inner_aliases
        .next()
        .expect("Next item is confirmed to exist")
        .to_string()
        .as_str()
        .into();
    let count_exprs = count_aliases.into_iter().map(|alias| AliasedDynProofExpr {
        alias: alias.clone(),
        expr: DynProofExpr::new_column(ColumnRef::new(
            dummy_table_ref.clone(),
            inner_count_alias.clone(),
            ColumnType::BigInt,
        )),
    });
    let projection_exprs = group_by_exprs
        .into_iter()
        .chain(sum_exprs)
        .chain(count_exprs)
        .collect();
    let inner_aggregate_plan = DynProofPlan::try_new_aggregate(
        inner_group_by_exprs,
        inner_sum_expr,
        inner_count_alias,
        input_plan,
        DynProofExpr::new_literal(LiteralValue::Boolean(true)),
    )
    .map_err(AggregatePlanError::from)?;
    Ok(DynProofPlan::new_projection(
        projection_exprs,
        inner_aggregate_plan,
    ))
}

fn join_to_proof_plan(
    join: &Join,
    schema_accessor: &impl SchemaAccessor,
) -> PlannerResult<DynProofPlan> {
    if join.join_type != JoinType::Inner {
        return Err(JoinPlanError::UnsupportedJoinType {
            join_type: join.join_type,
        }
        .into());
    }
    if join.join_constraint != JoinConstraint::On {
        return Err(JoinPlanError::UnsupportedJoinConstraint {
            constraint: join.join_constraint,
        }
        .into());
    }
    let left_plan = Box::new(logical_plan_to_proof_plan(&join.left, schema_accessor)?);
    let right_plan = Box::new(logical_plan_to_proof_plan(&join.right, schema_accessor)?);
    let left_column_result_fields = left_plan
        .get_column_result_fields()
        .into_iter()
        .map(|c| c.name())
        .collect::<IndexSet<_>>();
    let right_column_result_fields = right_plan
        .get_column_result_fields()
        .into_iter()
        .map(|c| c.name())
        .collect::<IndexSet<_>>();
    let on_indices_and_idents = join
        .on
        .iter()
        .filter_map(|(left_expr, right_expr)| {
            Some(match (left_expr, right_expr) {
                (Expr::Column(col_a), Expr::Column(col_b)) if col_a.name == col_b.name => {
                    let column_id = Ident::new(col_a.name.clone());
                    Ok((
                        (
                            left_column_result_fields.get_index_of(&column_id)?,
                            right_column_result_fields.get_index_of(&column_id)?,
                        ),
                        column_id,
                    ))
                }
                _ => Err(JoinPlanError::UnsupportedPredicate {
                    left: left_expr.to_string(),
                    right: right_expr.to_string(),
                }
                .into()),
            })
        })
        .collect::<PlannerResult<Vec<_>>>()?;
    let (on_indices, join_idents): (Vec<(usize, usize)>, Vec<Ident>) =
        on_indices_and_idents.into_iter().unzip();
    let (left_indices, right_indices): (Vec<usize>, Vec<usize>) = on_indices.into_iter().unzip();
    let (left_indices_cloned, right_indices_cloned) = (left_indices.clone(), right_indices.clone());
    let left_other_column_idents = left_column_result_fields
        .clone()
        .into_iter()
        .enumerate()
        .filter_map(|(i, col_ident)| (!left_indices.contains(&i)).then_some(col_ident));
    let right_other_column_idents = right_column_result_fields
        .into_iter()
        .enumerate()
        .filter_map(|(i, col_ident)| (!right_indices.contains(&i)).then_some(col_ident));
    Ok(DynProofPlan::SortMergeJoin(SortMergeJoinExec::new(
        left_plan,
        right_plan,
        left_indices_cloned,
        right_indices_cloned,
        join_idents
            .into_iter()
            .chain(left_other_column_idents)
            .chain(right_other_column_idents)
            .collect(),
    )))
}

/// Visit a [`datafusion::logical_plan::LogicalPlan`] and return a [`DynProofPlan`]
#[expect(clippy::too_many_lines)]
pub fn logical_plan_to_proof_plan(
    plan: &LogicalPlan,
    schema_accessor: &impl SchemaAccessor,
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
                table_scan_to_proof_plan(table_name, schema_accessor, projection)
            } else {
                table_scan_to_filter(
                    table_name,
                    schema_accessor,
                    projection,
                    projected_schema,
                    filters,
                )
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
                .map(|expr| expr.clone().unalias().display_name())
                .collect::<Result<Vec<_>, _>>()?;
            let alias_map = name_strings
                .into_iter()
                .zip(schema.fields().iter())
                .map(|(name_string, field)| Ok((name_string, field.name().clone())))
                .collect::<PlannerResult<IndexMap<_, _>>>()?;
            aggregate_to_proof_plan(input, group_expr, aggr_expr, schema_accessor, &alias_map)
        }
        // Projection
        LogicalPlan::Projection(Projection {
            input,
            expr,
            schema,
            ..
        }) => projection_to_proof_plan(expr, input, schema, schema_accessor),
        // Filter
        LogicalPlan::Filter(Filter {
            input, predicate, ..
        }) => {
            let input_plan = logical_plan_to_proof_plan(input, schema_accessor)?;
            let input_schema = input_plan
                .get_column_result_fields()
                .iter()
                .map(|field| (field.name(), field.data_type()))
                .collect::<Vec<_>>();
            let filter_proof_expr = expr_to_proof_expr(predicate, &input_schema)?;
            let aliased_exprs = input_plan
                .get_column_result_fields()
                .iter()
                .map(|field| -> PlannerResult<AliasedDynProofExpr> {
                    let alias = field.name();
                    Ok(AliasedDynProofExpr {
                        expr: DynProofExpr::new_column(ColumnRef::new(
                            TableRef::from_names(None, "__filter_input__"), // Dummy table ref
                            alias.clone(),
                            field.data_type(),
                        )),
                        alias,
                    })
                })
                .collect::<PlannerResult<Vec<_>>>()?;
            Ok(DynProofPlan::new_filter(
                aliased_exprs,
                input_plan,
                filter_proof_expr,
            ))
        }
        // Limit
        LogicalPlan::Limit(Limit { input, fetch, skip }) => {
            let input_plan = logical_plan_to_proof_plan(input, schema_accessor)?;
            Ok(DynProofPlan::new_slice(input_plan, *skip, *fetch))
        }
        // Union
        LogicalPlan::Union(Union { inputs, schema: _ }) => {
            let input_plans = inputs
                .iter()
                .map(|input| logical_plan_to_proof_plan(input, schema_accessor))
                .collect::<PlannerResult<Vec<_>>>()?;
            Ok(DynProofPlan::try_new_union(input_plans)?)
        }
        LogicalPlan::Join(join) => join_to_proof_plan(join, schema_accessor),
        LogicalPlan::SubqueryAlias(SubqueryAlias { input, .. }) => {
            logical_plan_to_proof_plan(input, schema_accessor)
        }
        _ => Err(PlannerError::UnsupportedLogicalPlan {
            node: LogicalPlanNodeKind::from_unsupported_logical_plan(plan),
        }),
    }
}

impl LogicalPlanNodeKind {
    fn from_unsupported_logical_plan(plan: &LogicalPlan) -> Self {
        match plan {
            LogicalPlan::Window(_) => Self::Window,
            LogicalPlan::Sort(_) => Self::Sort,
            LogicalPlan::CrossJoin(_) => Self::CrossJoin,
            LogicalPlan::Repartition(_) => Self::Repartition,
            LogicalPlan::TableScan(_) => Self::TableScan,
            LogicalPlan::Subquery(_) => Self::Subquery,
            LogicalPlan::Statement(_) => Self::Statement,
            LogicalPlan::Values(_) => Self::Values,
            LogicalPlan::Explain(_) => Self::Explain,
            LogicalPlan::Analyze(_) => Self::Analyze,
            LogicalPlan::Extension(_) => Self::Extension,
            LogicalPlan::Distinct(_) => Self::Distinct,
            LogicalPlan::Prepare(_) => Self::Prepare,
            LogicalPlan::Dml(_) => Self::Dml,
            LogicalPlan::Ddl(_) => Self::Ddl,
            LogicalPlan::Copy(_) => Self::Copy,
            LogicalPlan::DescribeTable(_) => Self::DescribeTable,
            LogicalPlan::Unnest(_) => Self::Unnest,
            LogicalPlan::RecursiveQuery(_) => Self::RecursiveQuery,
            LogicalPlan::Projection(_)
            | LogicalPlan::Filter(_)
            | LogicalPlan::Aggregate(_)
            | LogicalPlan::Join(_)
            | LogicalPlan::Union(_)
            | LogicalPlan::EmptyRelation(_)
            | LogicalPlan::SubqueryAlias(_)
            | LogicalPlan::Limit(_) => {
                unreachable!("supported logical plan nodes are handled before classification")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{df_util::*, PoSqlTableSource};
    use ahash::AHasher;
    use alloc::{sync::Arc, vec};
    use arrow::datatypes::{DataType, Schema};
    use core::ops::Add;
    use datafusion::{
        common::{Column, ScalarValue},
        logical_expr::{
            expr::{AggregateFunction, AggregateFunctionDefinition, Alias},
            lit, not, BinaryExpr, DescribeTable, EmptyRelation, LogicalPlanBuilder, Operator,
            Partitioning, Prepare, RecursiveQuery, SetVariable, Statement, Subquery, TableScan,
            TableSource,
        },
        physical_plan,
    };
    use indexmap::{indexmap, indexmap_with_default};
    use proof_of_sql::{
        base::{
            database::{ColumnField, SchemaAccessorImpl},
            math::decimal::Precision,
        },
        sql::{
            proof_exprs::{ColumnExpr, TableExpr},
            proof_plans::AggregateExecError,
        },
    };

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
    fn SCHEMAS() -> impl SchemaAccessor {
        let schema: Vec<(Ident, ColumnType)> = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
        let table_ref = TableRef::new("", "table");
        let schema_accessor = indexmap_with_default! {
            AHasher;
            table_ref => schema
        };
        SchemaAccessorImpl::new(schema_accessor)
    }

    #[expect(non_snake_case)]
    fn ALIASED_FILTER_RESULTS() -> Vec<AliasedDynProofExpr> {
        vec![
            AliasedDynProofExpr {
                alias: "a".into(),
                expr: DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "a".into(),
                    ColumnType::BigInt,
                ))),
            },
            AliasedDynProofExpr {
                alias: "b".into(),
                expr: DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "b".into(),
                    ColumnType::Int,
                ))),
            },
            AliasedDynProofExpr {
                alias: "c".into(),
                expr: DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "c".into(),
                    ColumnType::VarChar,
                ))),
            },
            AliasedDynProofExpr {
                alias: "d".into(),
                expr: DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    TABLE_REF_TABLE(),
                    "d".into(),
                    ColumnType::Boolean,
                ))),
            },
        ]
    }

    #[expect(non_snake_case)]
    fn FILTER_EXEC() -> DynProofPlan {
        DynProofPlan::new_filter(
            ALIASED_FILTER_RESULTS(),
            TABLE_EXEC(),
            DynProofExpr::new_column(ColumnRef::new(
                TABLE_REF_TABLE(),
                "d".into(),
                ColumnType::Boolean,
            )),
        )
    }

    #[expect(non_snake_case)]
    fn TABLE_EXEC() -> DynProofPlan {
        DynProofPlan::new_table(
            TABLE_REF_TABLE(),
            SCHEMAS()
                .lookup_schema(&TABLE_REF_TABLE())
                .iter()
                .map(|(name, data_type)| ColumnField::new(name.clone(), *data_type))
                .collect(),
        )
    }

    #[expect(non_snake_case)]
    fn UNION_SCHEMAS() -> impl SchemaAccessor {
        SchemaAccessorImpl::new(indexmap_with_default! {AHasher;
            TableRef::new("", "table1") => vec![("a1".into(), ColumnType::BigInt),
                ("b1".into(), ColumnType::Int)],
            TableRef::new("", "table2") => vec![("a2".into(), ColumnType::BigInt),
                ("b2".into(), ColumnType::Int)],
            TableRef::new("schema", "table3") => vec![("a3".into(), ColumnType::BigInt),
                ("b3".into(), ColumnType::Int)],
        })
    }

    #[expect(non_snake_case)]
    fn EMPTY_SCHEMAS() -> impl SchemaAccessor {
        SchemaAccessorImpl::new(indexmap_with_default! {AHasher;})
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
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            (
                "d".into(),
                ColumnType::Decimal75(Precision::new(5).unwrap(), 1),
            ), // Unused column
        ];
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
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
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
            "table.a".to_string() => "a".to_string(),
            "SUM(table.b)".to_string() => "sum_b".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        let dummy_ref_table = TableRef::from_names(None, "");

        let projection_exprs = vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "0".into(),
                    ColumnType::BigInt,
                )),
                alias: "a".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "1".into(),
                    ColumnType::Int,
                )),
                alias: "sum_b".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table,
                    "2".into(),
                    ColumnType::BigInt,
                )),
                alias: "count_1".into(),
            },
        ];

        // Expected result
        let expected = DynProofPlan::new_projection(
            projection_exprs,
            DynProofPlan::try_new_aggregate(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "a".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "0".into(),
                }],
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "b".into(),
                        ColumnType::Int,
                    )),
                    alias: "1".into(),
                }],
                "2".into(),
                TABLE_EXEC(),
                DynProofExpr::new_literal(LiteralValue::Boolean(true)),
            )
            .unwrap(),
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
            "table.a".to_string() => "a".to_string(),
            "SUM(table.b)".to_string() => "sum_b".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        let dummy_ref_table = TableRef::from_names(None, "");

        let projection_exprs = vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "0".into(),
                    ColumnType::BigInt,
                )),
                alias: "a".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "1".into(),
                    ColumnType::Int,
                )),
                alias: "sum_b".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table,
                    "2".into(),
                    ColumnType::BigInt,
                )),
                alias: "count_1".into(),
            },
        ];

        // Expected result
        let expected = DynProofPlan::new_projection(
            projection_exprs,
            DynProofPlan::try_new_aggregate(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "a".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "0".into(),
                }],
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "b".into(),
                        ColumnType::Int,
                    )),
                    alias: "1".into(),
                }],
                "2".into(),
                FILTER_EXEC(),
                DynProofExpr::new_literal(LiteralValue::Boolean(true)),
            )
            .unwrap(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn we_cannot_aggregate_with_multiple_group_columns() {
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
            "table.a".to_string() => "a".to_string(),
            "table.c".to_string() => "c".to_string(),
            "SUM(table.b)".to_string() => "sum_b".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        // Test the function
        let err =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap_err();
        assert!(matches!(
            err,
            PlannerError::UnsupportedAggregatePlan {
                source: AggregatePlanError::AggregateExec {
                    source: AggregateExecError::UnsupportedGroupByExpressionCount { count: 2 }
                },
                ..
            }
        ));

        // Expected result
        let expected = DynProofPlan::try_new_group_by(
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

        assert!(expected.is_none());
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
            "table.a".to_string() => "a".to_string(),
            "SUM(table.b)".to_string() => "sum_b".to_string(),
            "SUM(table.d)".to_string() => "sum_d".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        let dummy_ref_table = TableRef::from_names(None, "");

        let projection_exprs = vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "0".into(),
                    ColumnType::BigInt,
                )),
                alias: "a".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "1".into(),
                    ColumnType::Int,
                )),
                alias: "sum_b".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "2".into(),
                    ColumnType::Boolean,
                )),
                alias: "sum_d".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table,
                    "3".into(),
                    ColumnType::BigInt,
                )),
                alias: "count_1".into(),
            },
        ];

        // Expected result
        let expected = DynProofPlan::new_projection(
            projection_exprs,
            DynProofPlan::try_new_aggregate(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "a".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "0".into(),
                }],
                vec![
                    AliasedDynProofExpr {
                        expr: DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "b".into(),
                            ColumnType::Int,
                        )),
                        alias: "1".into(),
                    },
                    AliasedDynProofExpr {
                        expr: DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "d".into(),
                            ColumnType::Boolean,
                        )),
                        alias: "2".into(),
                    },
                ],
                "3".into(),
                TABLE_EXEC(),
                DynProofExpr::new_literal(LiteralValue::Boolean(true)),
            )
            .unwrap(),
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
            "table.a".to_string() => "a".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        // Test the function
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();

        let dummy_ref_table = TableRef::from_names(None, "");

        let projection_exprs = vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "0".into(),
                    ColumnType::BigInt,
                )),
                alias: "a".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table,
                    "1".into(),
                    ColumnType::BigInt,
                )),
                alias: "count_1".into(),
            },
        ];

        // Expected result
        let expected = DynProofPlan::new_projection(
            projection_exprs,
            DynProofPlan::try_new_aggregate(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "a".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "0".into(),
                }],
                vec![], // No SUMs
                "1".into(),
                TABLE_EXEC(),
                DynProofExpr::new_literal(LiteralValue::Boolean(true)),
            )
            .unwrap(),
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
            group_expr[0].clone().unalias().display_name().unwrap() => "res".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(
            matches!(
                result,
                Err(PlannerError::UnsupportedAggregatePlan {
                    source: AggregatePlanError::UnexpectedAggregateExpression { .. },
                    ..
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn we_report_unsupported_grouping_types() {
        for data_type in [ColumnType::VarChar, ColumnType::VarBinary] {
            let table_ref = TableRef::new("", "table");
            let schemas = SchemaAccessorImpl::new(indexmap_with_default! {AHasher;
                table_ref => vec![("value".into(), data_type)]
            });
            let source: Arc<dyn TableSource> =
                Arc::new(PoSqlTableSource::new(vec![ColumnField::new(
                    "value".into(),
                    data_type,
                )]));
            let input_plan = LogicalPlan::TableScan(
                TableScan::try_new("table", source, Some(vec![0]), vec![], None).unwrap(),
            );
            let group_expr = vec![df_column("table", "value")];
            let alias_map = indexmap! {
                "table.value".to_string() => "value".to_string(),
            };

            let err = aggregate_to_proof_plan(&input_plan, &group_expr, &[], &schemas, &alias_map)
                .unwrap_err();

            assert!(
                matches!(
                    err,
                    PlannerError::UnsupportedAggregatePlan {
                        source: AggregatePlanError::AggregateExec {
                            source: AggregateExecError::UnsupportedGroupByExpressionType {
                                data_type: actual_type
                            }
                        },
                        ..
                    } if actual_type == data_type
                ),
                "{err:?}"
            );
        }
    }

    #[test]
    fn we_report_missing_group_expression_alias() {
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
        let group_expr = vec![df_column("table", "a")];
        let aggr_expr = vec![COUNT_1()];
        let alias_map = indexmap! {
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        let err =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap_err();

        assert!(matches!(
            err,
            PlannerError::UnsupportedAggregatePlan {
                source: AggregatePlanError::MissingGroupExpressionAlias { expression },
            } if expression == "table.a"
        ));
    }

    #[test]
    fn we_report_missing_aggregate_expression_alias() {
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
        let group_expr = vec![df_column("table", "a")];
        let aggr_expr = vec![COUNT_1()];
        let alias_map = indexmap! {
            "table.a".to_string() => "a".to_string(),
        };

        let err =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap_err();

        assert!(matches!(
            err,
            PlannerError::UnsupportedAggregatePlan {
                source: AggregatePlanError::MissingAggregateExpressionAlias { expression },
            } if expression == "COUNT(Int64(1))"
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
            "table.a".to_string() => "a".to_string(),
            aggr_expr[0].clone().unalias().display_name().unwrap() => "b_plus_c".to_string(),
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(
            matches!(
                result,
                Err(PlannerError::UnsupportedAggregatePlan {
                    source: AggregatePlanError::UnexpectedAggregateExpression { .. },
                    ..
                })
            ),
            "{result:?}"
        );
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
            "table.a".to_string() => "a".to_string(),
            "AVG(table.b)".to_string() => "avg_b".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        // Test the function - should return an error
        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map);
        assert!(
            matches!(
                result,
                Err(PlannerError::UnsupportedAggregateOperation { .. })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn we_can_aggregate_without_count_expression() {
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
            args: vec![df_column("table", "a")],
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
            name: "sum_a".to_string(),
        });

        // Create aggregate expressions without an explicit COUNT result.
        let aggr_expr = vec![aliased_sum1, aliased_sum2];

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
            "table.a".to_string() => "a".to_string(),
            "SUM(table.b)".to_string() => "sum_b".to_string(),
            "SUM(table.a)".to_string() => "sum_a".to_string(),
        };

        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();
        assert!(matches!(&result, DynProofPlan::Projection(_)));
        assert_eq!(
            result.get_column_result_fields(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("sum_b".into(), ColumnType::Int),
                ColumnField::new("sum_a".into(), ColumnType::BigInt),
            ]
        );
    }

    #[test]
    fn we_can_aggregate_with_fetch_limit() {
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
            "table.a".to_string() => "a".to_string(),
            "COUNT(Int64(1))".to_string() => "count_1".to_string(),
        };

        let result =
            aggregate_to_proof_plan(&input_plan, &group_expr, &aggr_expr, &SCHEMAS(), &alias_map)
                .unwrap();
        assert!(matches!(&result, DynProofPlan::Projection(_)));
        assert_eq!(
            result.get_column_result_fields(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("count_1".into(), ColumnType::BigInt),
            ]
        );
    }

    // EmptyRelation
    #[test]
    fn we_can_convert_empty_plan_to_proof_plan() {
        let empty_plan = LogicalPlan::EmptyRelation(EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(DFSchema::empty()),
        });
        let result = logical_plan_to_proof_plan(&empty_plan, &EMPTY_SCHEMAS()).unwrap();
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
        let expected = DynProofPlan::new_table(
            TABLE_REF_TABLE(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::Int),
                ColumnField::new("c".into(), ColumnType::VarChar),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "Table does not exist in schema accessor.")]
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
        let _ = logical_plan_to_proof_plan(&plan, &schemas);
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
            DynProofPlan::new_table(
                TABLE_REF_TABLE(),
                vec![
                    ColumnField::new("a".into(), ColumnType::BigInt),
                    ColumnField::new("b".into(), ColumnType::Int),
                    ColumnField::new("c".into(), ColumnType::VarChar),
                    ColumnField::new("d".into(), ColumnType::Boolean),
                ],
            ),
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
    #[should_panic(expected = "Table does not exist in schema accessor.")]
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
        logical_plan_to_proof_plan(&plan, &schemas).unwrap();
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
            DynProofPlan::new_table(
                TABLE_REF_TABLE(),
                vec![
                    ColumnField::new("a".into(), ColumnType::BigInt),
                    ColumnField::new("b".into(), ColumnType::Int),
                    ColumnField::new("c".into(), ColumnType::VarChar),
                    ColumnField::new("d".into(), ColumnType::Boolean),
                ],
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
                DynProofPlan::new_table(
                    TABLE_REF_TABLE(),
                    vec![
                        ColumnField::new("a".into(), ColumnType::BigInt),
                        ColumnField::new("b".into(), ColumnType::Int),
                        ColumnField::new("d".into(), ColumnType::Boolean),
                    ],
                ),
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
            DynProofPlan::new_table(
                TABLE_REF_TABLE(),
                vec![
                    ColumnField::new("a".into(), ColumnType::BigInt),
                    ColumnField::new("b".into(), ColumnType::Int),
                    ColumnField::new("d".into(), ColumnType::Boolean),
                ],
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
                DynProofPlan::new_table(
                    TABLE_REF_TABLE(),
                    vec![
                        ColumnField::new("a".into(), ColumnType::BigInt),
                        ColumnField::new("b".into(), ColumnType::Int),
                    ],
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
                DynProofPlan::new_table(
                    TABLE_REF_TABLE(),
                    vec![
                        ColumnField::new("a".into(), ColumnType::BigInt),
                        ColumnField::new("b".into(), ColumnType::Int),
                    ],
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
            DynProofPlan::new_table(
                TABLE_REF_TABLE(),
                vec![
                    ColumnField::new("a".into(), ColumnType::BigInt),
                    ColumnField::new("b".into(), ColumnType::Int),
                ],
            ),
            2,
            None,
        );
        assert_eq!(result, expected);
    }

    // Union
    #[test]
    fn we_can_convert_union_plan_to_proof_plan() {
        let plan = LogicalPlan::Union(Union {
            schema: Arc::new(df_schema(
                "table",
                vec![("a1", DataType::Int64), ("b1", DataType::Int32)],
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
        let expected = DynProofPlan::try_new_union(vec![
            DynProofPlan::new_table(
                TableRef::from_names(None, "table1"),
                vec![
                    ColumnField::new("a1".into(), ColumnType::BigInt),
                    ColumnField::new("b1".into(), ColumnType::Int),
                ],
            ),
            DynProofPlan::new_table(
                TableRef::from_names(None, "table2"),
                vec![
                    ColumnField::new("a2".into(), ColumnType::BigInt),
                    ColumnField::new("b2".into(), ColumnType::Int),
                ],
            ),
            DynProofPlan::new_table(
                TableRef::from_names(Some("schema"), "table3"),
                vec![
                    ColumnField::new("a3".into(), ColumnType::BigInt),
                    ColumnField::new("b3".into(), ColumnType::Int),
                ],
            ),
        ])
        .unwrap();
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

        let dummy_ref_table = TableRef::from_names(None, "");

        let projection_exprs = vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "0".into(),
                    ColumnType::BigInt,
                )),
                alias: "a".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "1".into(),
                    ColumnType::Int,
                )),
                alias: "SUM(table.b)".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table,
                    "2".into(),
                    ColumnType::BigInt,
                )),
                alias: "COUNT(Int64(1))".into(),
            },
        ];

        // Expected result
        let expected = DynProofPlan::new_projection(
            projection_exprs,
            DynProofPlan::try_new_aggregate(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "a".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "0".into(),
                }],
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "b".into(),
                        ColumnType::Int,
                    )),
                    alias: "1".into(),
                }],
                "2".into(),
                FILTER_EXEC(),
                DynProofExpr::new_literal(LiteralValue::Boolean(true)),
            )
            .unwrap(),
        );

        assert_eq!(result, expected);
    }

    // Aggregate + Projection
    #[expect(clippy::too_many_lines)]
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

        let dummy_ref_table = TableRef::from_names(None, "");

        let projection_exprs = vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "0".into(),
                    ColumnType::BigInt,
                )),
                alias: "a".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table.clone(),
                    "1".into(),
                    ColumnType::Int,
                )),
                alias: "SUM(table.b)".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    dummy_ref_table,
                    "2".into(),
                    ColumnType::BigInt,
                )),
                alias: "COUNT(Int64(1))".into(),
            },
        ];

        // Expected result
        let expected = DynProofPlan::new_projection(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TABLE_REF_TABLE(),
                        "a".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "a".into(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(None, ""),
                        "SUM(table.b)".into(),
                        ColumnType::Int,
                    )),
                    alias: "sum_b".into(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(None, ""),
                        "COUNT(Int64(1))".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "count_1".into(),
                },
            ],
            DynProofPlan::new_projection(
                projection_exprs,
                DynProofPlan::try_new_aggregate(
                    vec![AliasedDynProofExpr {
                        expr: DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "a".into(),
                            ColumnType::BigInt,
                        )),
                        alias: "0".into(),
                    }],
                    vec![AliasedDynProofExpr {
                        expr: DynProofExpr::new_column(ColumnRef::new(
                            TABLE_REF_TABLE(),
                            "b".into(),
                            ColumnType::Int,
                        )),
                        alias: "1".into(),
                    }],
                    "2".into(),
                    FILTER_EXEC(),
                    DynProofExpr::new_literal(LiteralValue::Boolean(true)),
                )
                .unwrap(),
            ),
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
        logical_plan_to_proof_plan(&proj_plan, &SCHEMAS()).unwrap();
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
            Err(PlannerError::UnsupportedLogicalPlan {
                node: LogicalPlanNodeKind::Prepare,
                ..
            })
        ));
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn we_classify_unsupported_logical_plan_nodes() {
        let empty_plan = LogicalPlanBuilder::empty(false).build().unwrap();
        let empty_schema = Arc::new(DFSchema::empty());
        let plans = [
            (
                LogicalPlanBuilder::from(empty_plan.clone())
                    .sort([lit(1).sort(true, false)])
                    .unwrap()
                    .build()
                    .unwrap(),
                LogicalPlanNodeKind::Sort,
            ),
            (
                LogicalPlanBuilder::from(empty_plan.clone())
                    .cross_join(empty_plan.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
                LogicalPlanNodeKind::CrossJoin,
            ),
            (
                LogicalPlanBuilder::from(empty_plan.clone())
                    .repartition(Partitioning::RoundRobinBatch(2))
                    .unwrap()
                    .build()
                    .unwrap(),
                LogicalPlanNodeKind::Repartition,
            ),
            (
                LogicalPlan::TableScan(
                    TableScan::try_new("table", TABLE_SOURCE(), None, vec![], None).unwrap(),
                ),
                LogicalPlanNodeKind::TableScan,
            ),
            (
                LogicalPlan::Subquery(Subquery {
                    subquery: Arc::new(empty_plan.clone()),
                    outer_ref_columns: vec![],
                }),
                LogicalPlanNodeKind::Subquery,
            ),
            (
                LogicalPlan::Statement(Statement::SetVariable(SetVariable {
                    variable: "key".to_string(),
                    value: "value".to_string(),
                    schema: empty_schema,
                })),
                LogicalPlanNodeKind::Statement,
            ),
            (
                LogicalPlanBuilder::values(vec![vec![lit(1)]])
                    .unwrap()
                    .build()
                    .unwrap(),
                LogicalPlanNodeKind::Values,
            ),
            (
                LogicalPlanBuilder::from(empty_plan.clone())
                    .explain(false, false)
                    .unwrap()
                    .build()
                    .unwrap(),
                LogicalPlanNodeKind::Explain,
            ),
            (
                LogicalPlanBuilder::from(empty_plan.clone())
                    .explain(false, true)
                    .unwrap()
                    .build()
                    .unwrap(),
                LogicalPlanNodeKind::Analyze,
            ),
            (
                LogicalPlanBuilder::from(empty_plan.clone())
                    .distinct()
                    .unwrap()
                    .build()
                    .unwrap(),
                LogicalPlanNodeKind::Distinct,
            ),
            (
                LogicalPlan::DescribeTable(DescribeTable {
                    schema: Arc::new(Schema::empty()),
                    output_schema: Arc::new(DFSchema::empty()),
                }),
                LogicalPlanNodeKind::DescribeTable,
            ),
            (
                LogicalPlan::RecursiveQuery(RecursiveQuery {
                    name: "recursive".to_string(),
                    static_term: Arc::new(empty_plan.clone()),
                    recursive_term: Arc::new(empty_plan),
                    is_distinct: false,
                }),
                LogicalPlanNodeKind::RecursiveQuery,
            ),
        ];

        for (plan, expected_node) in plans {
            let err = logical_plan_to_proof_plan(&plan, &EMPTY_SCHEMAS()).unwrap_err();
            assert!(matches!(
                err,
                PlannerError::UnsupportedLogicalPlan { node } if node == expected_node
            ));
        }
    }

    #[test]
    fn we_can_error_if_not_inner_join() {
        let input_plan = LogicalPlan::Prepare(Prepare {
            name: "not_a_real_plan".to_string(),
            data_types: vec![],
            input: Arc::new(LogicalPlan::EmptyRelation(EmptyRelation {
                produce_one_row: false,
                schema: Arc::new(DFSchema::empty()),
            })),
        });
        let plan = LogicalPlan::Join(Join {
            left: Arc::new(input_plan.clone()),
            right: Arc::new(input_plan),
            on: Vec::new(),
            filter: None,
            join_type: JoinType::Left,
            join_constraint: JoinConstraint::On,
            schema: Arc::new(DFSchema::empty()),
            null_equals_null: false,
        });
        let schemas = SCHEMAS();
        let join_err = logical_plan_to_proof_plan(&plan, &schemas).unwrap_err();
        assert!(matches!(
            join_err,
            PlannerError::UnsupportedJoinPlan {
                source: JoinPlanError::UnsupportedJoinType {
                    join_type: JoinType::Left
                },
            }
        ));
    }

    #[test]
    fn we_report_unsupported_join_constraint() {
        let input_plan = LogicalPlan::EmptyRelation(EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(DFSchema::empty()),
        });
        let plan = LogicalPlan::Join(Join {
            left: Arc::new(input_plan.clone()),
            right: Arc::new(input_plan),
            on: Vec::new(),
            filter: None,
            join_type: JoinType::Inner,
            join_constraint: JoinConstraint::Using,
            schema: Arc::new(DFSchema::empty()),
            null_equals_null: false,
        });
        let join_err = logical_plan_to_proof_plan(&plan, &EMPTY_SCHEMAS()).unwrap_err();

        assert!(matches!(
            join_err,
            PlannerError::UnsupportedJoinPlan {
                source: JoinPlanError::UnsupportedJoinConstraint {
                    constraint: JoinConstraint::Using
                },
            }
        ));
    }

    #[test]
    fn we_report_unsupported_join_predicate() {
        let input_plan = LogicalPlan::EmptyRelation(EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(DFSchema::empty()),
        });
        let plan = LogicalPlan::Join(Join {
            left: Arc::new(input_plan.clone()),
            right: Arc::new(input_plan),
            on: vec![(df_column("left", "a"), df_column("right", "b"))],
            filter: None,
            join_type: JoinType::Inner,
            join_constraint: JoinConstraint::On,
            schema: Arc::new(DFSchema::empty()),
            null_equals_null: false,
        });
        let join_err = logical_plan_to_proof_plan(&plan, &EMPTY_SCHEMAS()).unwrap_err();

        assert!(matches!(
            join_err,
            PlannerError::UnsupportedJoinPlan {
                source: JoinPlanError::UnsupportedPredicate { left, right },
            } if left == "left.a" && right == "right.b"
        ));
    }

    // Filter (LogicalPlan::Filter) tests - Happy paths
    #[test]
    fn we_can_convert_simple_nested_filters() {
        let table_scan = LogicalPlan::TableScan(
            TableScan::try_new("table", TABLE_SOURCE(), Some(vec![0, 1, 3]), vec![], None).unwrap(),
        );
        let inner_filter = LogicalPlan::Filter(
            Filter::try_new(
                df_column("table", "a").gt(Expr::Literal(ScalarValue::Int64(Some(0)))),
                Arc::new(table_scan),
            )
            .unwrap(),
        );
        let outer_filter = LogicalPlan::Filter(
            Filter::try_new(df_column("table", "d"), Arc::new(inner_filter)).unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&outer_filter, &schemas).unwrap();

        assert!(matches!(result, DynProofPlan::Filter(_)));
    }

    #[test]
    fn we_can_convert_deeply_nested_filters_with_complex_predicates() {
        let table_scan = LogicalPlan::TableScan(
            TableScan::try_new("table", TABLE_SOURCE(), Some(vec![0, 1, 3]), vec![], None).unwrap(),
        );
        // First filter: a > 0
        let filter1 = LogicalPlan::Filter(
            Filter::try_new(
                df_column("table", "a").gt(Expr::Literal(ScalarValue::Int64(Some(0)))),
                Arc::new(table_scan),
            )
            .unwrap(),
        );
        // Second filter: b < 100
        let filter2 = LogicalPlan::Filter(
            Filter::try_new(
                df_column("table", "b").lt(Expr::Literal(ScalarValue::Int32(Some(100)))),
                Arc::new(filter1),
            )
            .unwrap(),
        );
        // Third filter: d (boolean)
        let filter3 = LogicalPlan::Filter(
            Filter::try_new(df_column("table", "d"), Arc::new(filter2)).unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&filter3, &schemas).unwrap();

        assert!(matches!(result, DynProofPlan::Filter(_)));
    }

    // Error path tests for Filter
    #[test]
    fn we_cannot_convert_filter_with_unsupported_column() {
        let table_scan = LogicalPlan::TableScan(
            TableScan::try_new("table", TABLE_SOURCE(), Some(vec![0, 1]), vec![], None).unwrap(),
        );
        let filter_plan = LogicalPlan::Filter(
            Filter::try_new(
                df_column("table", "nonexistent").gt(Expr::Literal(ScalarValue::Int64(Some(0)))),
                Arc::new(table_scan),
            )
            .unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&filter_plan, &schemas);

        assert!(matches!(result, Err(PlannerError::ColumnNotFound)));
    }

    // table_scan_get_required_columns tests
    #[test]
    fn we_can_get_required_columns_from_projection_only() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
        let projection = vec![0, 2]; // Select columns 'a' and 'c'
        let filters = vec![];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["a".into(), "c".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_from_filters_only() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
        let projection = vec![];
        let filters = vec![
            df_column("table", "b").gt(Expr::Literal(ScalarValue::Int32(Some(10)))),
            df_column("table", "d"),
        ];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["b".into(), "d".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_from_projection_and_filters() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
        let projection = vec![0, 2]; // 'a' and 'c'
        let filters = vec![df_column("table", "b").gt(Expr::Literal(ScalarValue::Int32(Some(10))))];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["a".into(), "c".into(), "b".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_handle_overlapping_columns_in_projection_and_filters() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
        ];
        let projection = vec![0, 1]; // 'a' and 'b'
        let filters = vec![df_column("table", "b").gt(Expr::Literal(ScalarValue::Int32(Some(10))))];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        // 'b' should only appear once
        let expected: IndexSet<Ident> = ["a".into(), "b".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_with_complex_filter_expressions() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
        let projection = vec![0]; // Only 'a' in projection
        let filters = vec![df_column("table", "b")
            .gt(Expr::Literal(ScalarValue::Int32(Some(10))))
            .and(
                df_column("table", "c")
                    .eq(Expr::Literal(ScalarValue::Utf8(Some("test".to_string())))),
            )
            .and(df_column("table", "d"))];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into(), "d".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_with_empty_projection_and_filters() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
        ];
        let projection = vec![];
        let filters = vec![];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        assert!(result.is_empty());
    }

    #[test]
    fn we_can_get_required_columns_with_all_columns_projected() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
        ];
        let projection = vec![0, 1, 2]; // All columns
        let filters = vec![];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_with_out_of_order_projection() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
        let projection = vec![3, 1, 0]; // 'd', 'b', 'a' - out of order
        let filters = vec![];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["d".into(), "b".into(), "a".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_with_nested_filter_expressions() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
        ];
        let projection = vec![0];
        // NOT (b > 10)
        let filters = vec![Expr::Not(Box::new(
            df_column("table", "b").gt(Expr::Literal(ScalarValue::Int32(Some(10)))),
        ))];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["a".into(), "b".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_with_multiple_filters() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
        ];
        let projection = vec![0]; // Only 'a'
        let filters = vec![
            df_column("table", "b").gt(Expr::Literal(ScalarValue::Int32(Some(10)))),
            df_column("table", "c").eq(Expr::Literal(ScalarValue::Utf8(Some("test".to_string())))),
            df_column("table", "d"),
        ];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into(), "d".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_handle_projection_indices_that_skip_columns() {
        let input_schema = vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::Int),
            ("c".into(), ColumnType::VarChar),
            ("d".into(), ColumnType::Boolean),
            ("e".into(), ColumnType::Int),
        ];
        let projection = vec![0, 2, 4]; // 'a', 'c', 'e' - skipping 'b' and 'd'
        let filters = vec![df_column("table", "b").gt(Expr::Literal(ScalarValue::Int32(Some(5))))];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let expected: IndexSet<Ident> = ["a".into(), "c".into(), "e".into(), "b".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_required_columns_preserving_order() {
        let input_schema = vec![
            ("z".into(), ColumnType::BigInt),
            ("a".into(), ColumnType::Int),
            ("m".into(), ColumnType::VarChar),
        ];
        let projection = vec![0, 1]; // 'z', 'a'
        let filters =
            vec![df_column("table", "m")
                .eq(Expr::Literal(ScalarValue::Utf8(Some("test".to_string()))))];

        let result = table_scan_get_required_columns(&projection, &filters, &input_schema);
        let idents: Vec<Ident> = result.into_iter().collect();
        // Should preserve order: projection first, then filter columns
        assert_eq!(idents, vec!["z".into(), "a".into(), "m".into()]);
    }

    #[test]
    fn we_can_convert_subquery_plan_to_proof_plan() {
        let empty_plan = LogicalPlan::EmptyRelation(EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(DFSchema::empty()),
        });
        let subquery_plan =
            LogicalPlan::SubqueryAlias(SubqueryAlias::try_new(empty_plan.into(), "test").unwrap());
        let result = logical_plan_to_proof_plan(&subquery_plan, &EMPTY_SCHEMAS()).unwrap();
        assert_eq!(result, DynProofPlan::new_empty());
    }
}
