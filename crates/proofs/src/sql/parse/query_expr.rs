use super::{FilterExprBuilder, QueryContextBuilder, ResultExprBuilder};
use crate::{
    base::{commitment::Commitment, database::SchemaAccessor},
    sql::{ast::ProofPlan, parse::ConversionResult, transform::ResultExpr},
};
use proofs_sql::{intermediate_ast::SetExpression, Identifier, SelectStatement};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Serialize, Deserialize)]
pub struct QueryExpr<C: Commitment> {
    proof_expr: ProofPlan<C>,
    result: ResultExpr,
}

// Implements fmt::Debug to aid in debugging QueryExpr.
// Prints filter and result fields in a readable format.
impl<C: Commitment> fmt::Debug for QueryExpr<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QueryExpr \n[{:#?},\n{:#?}\n]",
            self.proof_expr, self.result
        )
    }
}

impl<C: Commitment> QueryExpr<C> {
    pub fn new(proof_expr: ProofPlan<C>, result: ResultExpr) -> Self {
        Self { proof_expr, result }
    }

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

        let filter = FilterExprBuilder::new(context.get_column_mapping())
            .add_table_expr(*context.get_table_ref())
            .add_where_expr(context.get_where_expr().clone())?
            .add_result_column_set(context.get_result_column_set())
            .build();

        let result_aliased_exprs = context.get_aliased_result_exprs()?;
        let result = ResultExprBuilder::default()
            .add_group_by_exprs(context.get_group_by_exprs(), result_aliased_exprs)
            .add_select_exprs(result_aliased_exprs)
            .add_order_by_exprs(context.get_order_by_exprs()?)
            .add_slice_expr(context.get_slice_expr())
            .build();

        Ok(Self {
            proof_expr: ProofPlan::Filter(filter),
            result,
        })
    }

    /// Immutable access to this query's provable filter expression.
    pub fn proof_expr(&self) -> &ProofPlan<C> {
        &self.proof_expr
    }

    /// Immutable access to this query's post-proof result transform expression.
    pub fn result(&self) -> &ResultExpr {
        &self.result
    }
}
