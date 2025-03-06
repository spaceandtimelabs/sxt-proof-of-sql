use super::{expr_to_proof_expr, table_reference_to_table_ref, PlannerError, PlannerResult};
use alloc::vec::Vec;
use datafusion::{
    common::DFSchema,
    logical_expr::{Expr, LogicalPlan, TableScan},
    sql::{sqlparser::ast::Ident, TableReference},
};
use indexmap::IndexMap;
use proof_of_sql::{
    base::database::{ColumnField, ColumnRef, ColumnType, TableRef},
    sql::{
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, ProofExpr, TableExpr},
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
    projection: Option<Vec<usize>>,
    input_schema: &DFSchema,
    output_schema: &DFSchema,
) -> PlannerResult<Vec<AliasedDynProofExpr>> {
    let num_input_columns = input_schema.columns().len();
    let projection_indexes =
        projection.unwrap_or_else(|| (0..num_input_columns).collect::<Vec<_>>());
    projection_indexes
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
    projection: Option<Vec<usize>>,
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
    let column_fields = aliased_dyn_proof_exprs
        .iter()
        .map(|aliased_expr| {
            ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
        })
        .collect::<Vec<_>>();
    let table_exec = DynProofPlan::new_table(table_ref, column_fields);
    Ok(DynProofPlan::new_projection(
        aliased_dyn_proof_exprs,
        table_exec,
    ))
}

/// Convert a `TableScan` with filters but without fetch limit to a `DynProofPlan`
fn table_scan_to_filter(
    table_name: &TableReference,
    schemas: &IndexMap<TableReference, DFSchema>,
    projection: Option<Vec<usize>>,
    projected_schema: &DFSchema,
    filters: &[Expr],
) -> PlannerResult<DynProofPlan> {
    // Check if the table exists
    let table_ref = table_reference_to_table_ref(table_name)?;
    let table_expr = TableExpr {
        table_ref: table_ref.clone(),
    };
    let input_schema = schemas
        .get(table_name)
        .ok_or_else(|| PlannerError::TableNotFound {
            table_name: table_name.to_string(),
        })?;
    // Get aliased expressions
    let aliased_dyn_proof_exprs =
        get_aliased_dyn_proof_exprs(&table_ref, projection, input_schema, projected_schema)?;
    // Process filter
    let filter_proof_exprs = filters
        .iter()
        .map(|f| expr_to_proof_expr(f, input_schema))
        .collect::<PlannerResult<Vec<_>>>()?;
    // Filter
    let consolidated_filter_proof_expr = filter_proof_exprs[1..]
        .iter()
        .cloned()
        .try_fold(filter_proof_exprs[0].clone(), DynProofExpr::try_new_and)?;
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
        // No filter or fetch limit
        LogicalPlan::TableScan(TableScan {
            table_name,
            projection,
            projected_schema,
            filters,
            fetch: None,
            ..
        }) if filters.is_empty() => {
            table_scan_to_projection(table_name, schemas, projection.clone(), projected_schema)
        }
        // Filter but no fetch limit
        LogicalPlan::TableScan(TableScan {
            table_name,
            projection,
            projected_schema,
            filters,
            fetch: None,
            ..
        }) if !filters.is_empty() => table_scan_to_filter(
            table_name,
            schemas,
            projection.clone(),
            projected_schema,
            filters,
        ),
        _ => Err(PlannerError::UnsupportedLogicalPlan { plan: plan.clone() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{df_util::*, PoSqlTableSource};
    use alloc::{sync::Arc, vec};
    use arrow::datatypes::DataType;
    use datafusion::logical_expr::{EmptyRelation, Prepare, TableScan, TableSource};
    use indexmap::indexmap;

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
        let result = get_aliased_dyn_proof_exprs(
            &table_ref,
            Some(vec![1, 2]),
            &input_schema,
            &output_schema,
        )
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
            get_aliased_dyn_proof_exprs(&table_ref, None, &input_schema, &output_schema).unwrap();
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
            get_aliased_dyn_proof_exprs(
                &table_ref,
                Some(vec![1, 2, 3]),
                &input_schema,
                &output_schema,
            ),
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
            TableScan::try_new("table", TABLE_SOURCE(), None, vec![], None).unwrap(),
        );
        let schemas = SCHEMAS();
        let result = logical_plan_to_proof_plan(&plan, &schemas).unwrap();
        let expected = DynProofPlan::new_projection(
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
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn we_cannot_convert_table_scan_plan_to_proof_plan_without_filter_or_fetch_limit_if_bad_schemas(
    ) {
        let plan = LogicalPlan::TableScan(
            TableScan::try_new("table", TABLE_SOURCE(), None, vec![], None).unwrap(),
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
