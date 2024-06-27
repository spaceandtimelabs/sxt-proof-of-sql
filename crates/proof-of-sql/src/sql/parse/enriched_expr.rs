use super::ProvableExprPlanBuilder;
use crate::{
    base::{commitment::Commitment, database::ColumnRef},
    sql::ast::ProvableExprPlan,
};
use proof_of_sql_parser::{
    intermediate_ast::{AliasedResultExpr, Expression},
    Identifier,
};
use std::collections::HashMap;
/// Enriched expression
///
/// An enriched expression consists of an `proof_of_sql_parser::intermediate_ast::AliasedResultExpr`
/// and an optional `ProvableExprPlan`.
/// If the `ProvableExprPlan` is `None`, the `EnrichedExpr` is not provable.
pub struct EnrichedExpr<C: Commitment> {
    /// The remaining expression after the provable expression plan has been extracted.
    pub residue_expression: AliasedResultExpr,
    /// The extracted provable expression plan if it exists.
    pub provable_expr_plan: Option<ProvableExprPlan<C>>,
}

impl<C: Commitment> EnrichedExpr<C> {
    /// Create a new `EnrichedExpr` with a provable expression.
    ///
    /// If the expression is not provable, the `provable_expr_plan` will be `None`.
    /// Otherwise the `provable_expr_plan` will contain the provable expression plan
    /// and the `residue_expression` will contain the remaining expression.
    pub fn new(
        expression: AliasedResultExpr,
        column_mapping: HashMap<Identifier, ColumnRef>,
        allow_aggregates: bool,
    ) -> Self {
        let res_provable_expr_plan = if allow_aggregates {
            ProvableExprPlanBuilder::new(&column_mapping).build(&expression.expr)
        } else {
            ProvableExprPlanBuilder::new_agg(&column_mapping).build(&expression.expr)
        };
        match res_provable_expr_plan {
            Ok(provable_expr_plan) => {
                let alias = expression.alias;
                Self {
                    residue_expression: AliasedResultExpr {
                        expr: Box::new(Expression::Column(alias)),
                        alias,
                    },
                    provable_expr_plan: Some(provable_expr_plan),
                }
            }
            Err(_) => Self {
                residue_expression: expression,
                provable_expr_plan: None,
            },
        }
    }

    /// Get the alias of the `EnrichedExpr`.
    ///
    /// Since we plan to support unaliased expressions in the future, this method returns an `Option`.
    #[allow(dead_code)]
    pub fn get_alias(&self) -> Option<&Identifier> {
        self.residue_expression.try_as_identifier()
    }

    /// Is the `EnrichedExpr` provable
    #[allow(dead_code)]
    pub fn is_provable(&self) -> bool {
        self.provable_expr_plan.is_some()
    }
}
