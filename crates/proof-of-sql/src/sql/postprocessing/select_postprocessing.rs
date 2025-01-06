use super::{PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{OwnedColumn, OwnedTable},
    map::IndexMap,
    scalar::Scalar,
};
use alloc::vec::Vec;
use proof_of_sql_parser::intermediate_ast::AliasedResultExpr;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{Expr, Ident};

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectPostprocessing {
    /// The aliased result expressions we select
    aliased_result_exprs: Vec<AliasedResultExpr>,
}

impl SelectPostprocessing {
    /// Create a new `SelectPostprocessing` node.
    #[must_use]
    pub fn new(aliased_result_exprs: Vec<AliasedResultExpr>) -> Self {
        Self {
            aliased_result_exprs,
        }
    }
}

impl<S: Scalar> PostprocessingStep<S> for SelectPostprocessing {
    /// Apply the select transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let cols: IndexMap<Ident, OwnedColumn<S>> = self
            .aliased_result_exprs
            .iter()
            .map(
                |aliased_result_expr| -> PostprocessingResult<(Ident, OwnedColumn<S>)> {
                    let sql_expr: Expr = (*aliased_result_expr.expr).clone().into();
                    let result_column = owned_table.evaluate(&sql_expr)?;
                    Ok((aliased_result_expr.alias.into(), result_column))
                },
            )
            .collect::<PostprocessingResult<_>>()?;
        Ok(OwnedTable::try_new(cols)?)
    }
}
