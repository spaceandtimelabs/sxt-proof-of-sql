use super::{
    column_as_column_ref, scalar_value_as_literal_value, table_reference_as_table_ref,
    PlannerError, PlannerResult,
};
use alloc::vec::Vec;
use core::ops::Range;
use datafusion::{
    common::DFSchema,
    logical_expr::{BinaryExpr, Expr, LogicalPlan, Operator, TableScan},
    sql::{sqlparser::ast::Ident, TableReference},
};
use proof_of_sql::{
    base::{
        database::{ColumnRef, ColumnType, LiteralValue},
        map::IndexMap,
    },
    sql::{
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, TableExpr},
        proof_plans::{DynProofPlan, EmptyExec, FilterExec, ProjectionExec},
    },
};

/// Visit an [`datafusion::expr::Expr`] and return a [`DynProofExpr`]
pub(crate) fn visit_expr(expr: &Expr, schema: &DFSchema) -> PlannerResult<DynProofExpr> {
    match expr {
        Expr::Column(col) => Ok(DynProofExpr::new_column(column_as_column_ref(col, schema)?)),
        Expr::BinaryExpr(BinaryExpr { left, right, op }) => {
            let left_proof_expr = visit_expr(left, schema)?;
            let right_proof_expr = visit_expr(right, schema)?;
            match op {
                Operator::Eq => Ok(DynProofExpr::try_new_equals(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                Operator::Lt => Ok(DynProofExpr::try_new_inequality(
                    left_proof_expr,
                    right_proof_expr,
                    true,
                )?),
                Operator::Gt => Ok(DynProofExpr::try_new_inequality(
                    left_proof_expr,
                    right_proof_expr,
                    false,
                )?),
                Operator::LtEq => Ok(DynProofExpr::try_new_not(
                    DynProofExpr::try_new_inequality(left_proof_expr, right_proof_expr, false)?,
                )?),
                Operator::GtEq => Ok(DynProofExpr::try_new_not(
                    DynProofExpr::try_new_inequality(left_proof_expr, right_proof_expr, true)?,
                )?),
                Operator::Plus => Ok(DynProofExpr::try_new_add(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                Operator::Minus => Ok(DynProofExpr::try_new_subtract(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                Operator::Multiply => Ok(DynProofExpr::try_new_multiply(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                _ => {
                    panic!("Unsupported binary operator: {:?}", op);
                }
            }
        }
        Expr::Literal(val) => Ok(DynProofExpr::new_literal(scalar_value_as_literal_value(
            val,
        )?)),
        Expr::Not(expr) => {
            let proof_expr = visit_expr(expr, schema)?;
            Ok(DynProofExpr::try_new_not(proof_expr)?)
        }
        _ => {
            panic!("Unsupported expression: {:?}", expr);
        }
    }
}

/// Visit a [`datafusion::logical_plan::LogicalPlan`] and return a [`DynProofPlan`]
pub(crate) fn visit_plan(
    plan: &LogicalPlan,
    schemas: &IndexMap<TableReference, DFSchema>,
) -> PlannerResult<DynProofPlan> {
    match plan {
        LogicalPlan::EmptyRelation { .. } => Ok(DynProofPlan::Empty(EmptyExec {})),
        LogicalPlan::TableScan(TableScan {
            table_name,
            projection,
            projected_schema,
            filters,
            ..
        }) => {
            // Check if the table exists
            let table_index = schemas.keys().position(|t| t == table_name);
            if table_index.is_none() {
                panic!("Table not found");
            }
            let table_ref = table_reference_as_table_ref(table_name);
            let table_expr = TableExpr { table_ref };
            let input_schema = schemas.get(table_name).expect("Table should exist");
            // Get the aliased dyn proof exprs
            let num_input_columns = input_schema.columns().len();
            let projection_indexes =
                projection.unwrap_or_else(|| (0..num_input_columns).collect::<Vec<_>>());
            let aliased_dyn_proof_exprs = projection_indexes
                .iter()
                .enumerate()
                .map(|(output_index, input_index)| {
                    // Get output column name / alias
                    let alias: Ident = projected_schema.field(output_index).name().as_str().into();
                    let input_column_name: Ident =
                        input_schema.field(*input_index).name().as_str().into();
                    let data_type = input_schema.field(*input_index).data_type();
                    let expr = DynProofExpr::new_column(ColumnRef::new(
                        table_ref,
                        input_column_name,
                        ColumnType::try_from(data_type).expect("Unsupported data type"),
                    ));
                    AliasedDynProofExpr { expr, alias }
                })
                .collect::<Vec<_>>();
            // Process filter
            if filters.len() > 1 {
                panic!("Multiple filters not supported");
            }
            let filter_proof_exprs = filters
                .iter()
                .map(|f| visit_expr(f, input_schema))
                .collect::<PlannerResult<Vec<_>>>()?;
            let num_filters = filter_proof_exprs.len();
            // Filter
            let mut consolidated_filter_proof_expr = if num_filters == 0 {
                DynProofExpr::new_literal(LiteralValue::Boolean(true))
            } else {
                filter_proof_exprs[0]
            };
            for i in 0..filter_proof_exprs.len() - 1 {
                consolidated_filter_proof_expr = DynProofExpr::try_new_and(
                    consolidated_filter_proof_expr,
                    filter_proof_exprs[i + 1],
                )?;
            }
            Ok(DynProofPlan::Filter(FilterExec::new(
                aliased_dyn_proof_exprs,
                table_expr,
                consolidated_filter_proof_expr,
            )))
        }
        _ => {
            panic!("Unsupported plan: {:?}", plan);
        }
    }
}
