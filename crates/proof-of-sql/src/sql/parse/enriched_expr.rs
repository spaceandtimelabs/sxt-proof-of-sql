use super::DynProofExprBuilder;
use crate::{
    base::{database::ColumnRef, map::IndexMap},
    sql::proof_exprs::DynProofExpr,
};
use alloc::boxed::Box;
use proof_of_sql_parser::sqlparser::SqlAliasedResultExpr;
use sqlparser::ast::{Expr, Ident};

/// Enriched expression
///
/// An enriched expression consists of an `proof_of_sql_parser::intermediate_ast::AliasedResultExpr`
/// and an optional `DynProofExpr`.
/// If the `DynProofExpr` is `None`, the `EnrichedExpr` is not provable.
pub struct EnrichedExpr {
    /// The remaining expression after the provable expression plan has been extracted.
    pub residue_expression: SqlAliasedResultExpr,
    /// The extracted provable expression plan if it exists.
    pub dyn_proof_expr: Option<DynProofExpr>,
}

impl EnrichedExpr {
    /// Create a new `EnrichedExpr` with a provable expression.
    ///
    /// If the expression is not provable, the `dyn_proof_expr` will be `None`.
    /// Otherwise the `dyn_proof_expr` will contain the provable expression plan
    /// and the `residue_expression` will contain the remaining expression.
    pub fn new(
        expression: SqlAliasedResultExpr,
        column_mapping: &IndexMap<Ident, ColumnRef>,
    ) -> Self {
        // TODO: Using new_agg (ironically) disables aggregations in `QueryExpr` for now.
        // Re-enable aggregations when we add `GroupByExec` generalizations.
        let res_dyn_proof_expr =
            DynProofExprBuilder::new_agg(column_mapping).build(&expression.expr);
        match res_dyn_proof_expr {
            Ok(dyn_proof_expr) => {
                let alias = expression.alias;
                Self {
                    residue_expression: SqlAliasedResultExpr {
                        expr: Box::new(Expr::Identifier(alias.clone())),
                        alias,
                    },
                    dyn_proof_expr: Some(dyn_proof_expr),
                }
            }
            Err(_) => Self {
                residue_expression: expression,
                dyn_proof_expr: None,
            },
        }
    }

    /// Get the alias of the `EnrichedExpr`.
    ///
    /// Since we plan to support unaliased expressions in the future, this method returns an `Option`.
    #[allow(dead_code)]
    pub fn get_alias(&self) -> Option<Ident> {
        self.residue_expression
            .try_as_identifier()
            .map(|identifier| Ident::new(identifier.value.as_str()))
    }

    /// Is the `EnrichedExpr` provable
    #[allow(dead_code)]
    pub fn is_provable(&self) -> bool {
        self.dyn_proof_expr.is_some()
    }
}
