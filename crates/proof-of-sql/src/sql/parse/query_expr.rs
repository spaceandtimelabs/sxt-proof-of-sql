use super::{EnrichedExpr, FilterExecBuilder, QueryContextBuilder};
use crate::{
    base::{commitment::Commitment, database::SchemaAccessor},
    sql::{
        parse::ConversionResult,
        postprocessing::{
            GroupByPostprocessing, OrderByPostprocessing, OwnedTablePostprocessing,
            SelectPostprocessing, SlicePostprocessing,
        },
        proof_plans::{DynProofPlan, GroupByExec},
    },
};
use proof_of_sql_parser::{intermediate_ast::SetExpression, Identifier, SelectStatement};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Serialize, Deserialize)]
/// A `QueryExpr` represents a Proof of SQL query that can be executed against a database.
/// It consists of a `DynProofPlan` for provable components and a vector of `OwnedTablePostprocessing` for the rest.
pub struct QueryExpr<C: Commitment> {
    proof_expr: DynProofPlan<C>,
    postprocessing: Vec<OwnedTablePostprocessing>,
}

// Implements fmt::Debug to aid in debugging QueryExpr.
// Prints filter and postprocessing fields in a readable format.
impl<C: Commitment> fmt::Debug for QueryExpr<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QueryExpr \n[{:#?},\n{:#?}\n]",
            self.proof_expr, self.postprocessing
        )
    }
}

impl<C: Commitment> QueryExpr<C> {
    /// Creates a new `QueryExpr` with the given `DynProofPlan` and `OwnedTablePostprocessing`.
    pub fn new(proof_expr: DynProofPlan<C>, postprocessing: Vec<OwnedTablePostprocessing>) -> Self {
        Self {
            proof_expr,
            postprocessing,
        }
    }

    /// Parse an intermediate AST `SelectStatement` into a `QueryExpr`.
    pub fn try_new(
        ast: SelectStatement,
        default_schema: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Self> {
        let context = match *ast.expr {
            SetExpression::Query {
                result_exprs,
                from,
                where_expr,
                group_by,
            } => QueryContextBuilder::new(schema_accessor)
                .visit_table_expr(from, default_schema)
                .visit_group_by_exprs(group_by)?
                .visit_result_exprs(result_exprs)?
                .visit_where_expr(where_expr)?
                .visit_order_by_exprs(ast.order_by)
                .visit_slice_expr(ast.slice)
                .build()?,
        };
        let result_aliased_exprs = context.get_aliased_result_exprs()?.to_vec();
        let group_by = context.get_group_by_exprs();

        // Figure out the basic postprocessing steps.
        let mut postprocessing = vec![];
        let order_bys = context.get_order_by_exprs()?;
        if !order_bys.is_empty() {
            postprocessing.push(OwnedTablePostprocessing::new_order_by(
                OrderByPostprocessing::new(order_bys.clone()),
            ));
        }
        if let Some(slice) = context.get_slice_expr() {
            postprocessing.push(OwnedTablePostprocessing::new_slice(
                SlicePostprocessing::new(Some(slice.number_rows), Some(slice.offset_value)),
            ));
        }
        if context.has_agg() {
            if let Some(group_by_expr) = Option::<GroupByExec<C>>::try_from(&context)? {
                Ok(Self {
                    proof_expr: DynProofPlan::GroupBy(group_by_expr),
                    postprocessing,
                })
            } else {
                let raw_enriched_exprs = result_aliased_exprs
                    .iter()
                    .map(|aliased_expr| EnrichedExpr {
                        residue_expression: aliased_expr.clone(),
                        dyn_proof_expr: None,
                    })
                    .collect::<Vec<_>>();
                let filter = FilterExecBuilder::new(context.get_column_mapping())
                    .add_table_expr(*context.get_table_ref())
                    .add_where_expr(context.get_where_expr().clone())?
                    .add_result_columns(&raw_enriched_exprs)
                    .build();

                let group_by_postprocessing =
                    GroupByPostprocessing::try_new(group_by.to_vec(), result_aliased_exprs)?;
                postprocessing.insert(
                    0,
                    OwnedTablePostprocessing::new_group_by(group_by_postprocessing.clone()),
                );
                let remainder_exprs = group_by_postprocessing.remainder_exprs();
                // Check whether we need to do select postprocessing.
                // That is, if any of them is not simply a column reference.
                if remainder_exprs
                    .iter()
                    .any(|expr| expr.try_as_identifier().is_none())
                {
                    postprocessing.insert(
                        1,
                        OwnedTablePostprocessing::new_select(SelectPostprocessing::new(
                            remainder_exprs.to_vec(),
                        )),
                    );
                }
                Ok(Self {
                    proof_expr: DynProofPlan::Filter(filter),
                    postprocessing,
                })
            }
        } else {
            // No group by, so we need to do a dense filter.
            let column_mapping = context.get_column_mapping();
            let enriched_exprs = result_aliased_exprs
                .iter()
                .map(|aliased_expr| EnrichedExpr::new(aliased_expr.clone(), column_mapping.clone()))
                .collect::<Vec<_>>();
            let select_exprs = enriched_exprs
                .iter()
                .map(|enriched_expr| enriched_expr.residue_expression.clone())
                .collect::<Vec<_>>();
            let filter = FilterExecBuilder::new(context.get_column_mapping())
                .add_table_expr(*context.get_table_ref())
                .add_where_expr(context.get_where_expr().clone())?
                .add_result_columns(&enriched_exprs)
                .build();
            // Check whether we need to do select postprocessing.
            if select_exprs
                .iter()
                .any(|expr| expr.try_as_identifier().is_none())
            {
                postprocessing.insert(
                    0,
                    OwnedTablePostprocessing::new_select(SelectPostprocessing::new(select_exprs)),
                );
            }
            Ok(Self {
                proof_expr: DynProofPlan::Filter(filter),
                postprocessing,
            })
        }
    }

    /// Immutable access to this query's provable filter expression.
    pub fn proof_expr(&self) -> &DynProofPlan<C> {
        &self.proof_expr
    }

    /// Immutable access to this query's post-proof result transform expression.
    pub fn postprocessing(&self) -> &[OwnedTablePostprocessing] {
        &self.postprocessing
    }
}
