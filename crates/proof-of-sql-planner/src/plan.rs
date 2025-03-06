use super::{
    df_schema_to_column_fields, expr_to_proof_expr, table_reference_to_table_ref, PlannerError,
    PlannerResult,
};
use alloc::vec::Vec;
use datafusion::{
    common::DFSchema,
    logical_expr::{Expr, Limit, LogicalPlan, Projection, TableScan, Union},
    sql::{sqlparser::ast::Ident, TableReference},
};
use indexmap::IndexMap;
use proof_of_sql::{
    base::database::{ColumnRef, ColumnType, TableRef},
    sql::{
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, TableExpr},
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
        // Projection
        LogicalPlan::Projection(Projection {
            input,
            expr,
            schema,
            ..
        }) => {
            let input_plan = logical_plan_to_proof_plan(input, schemas)?;
            let input_schema = input.schema();
            let aliased_exprs = expr
                .iter()
                .zip(schema.fields().into_iter())
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
    use datafusion::logical_expr::{
        not, BinaryExpr, EmptyRelation, Operator, Prepare, TableScan, TableSource,
    };
    use indexmap::indexmap;
    use proof_of_sql::base::database::ColumnField;

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
