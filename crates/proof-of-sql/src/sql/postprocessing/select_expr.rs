use super::{PostprocessingError, PostprocessingEvaluator, PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{OwnedColumn, OwnedTable},
    scalar::Scalar,
};
use indexmap::IndexMap;
use proof_of_sql_parser::{Identifier, intermediate_ast::AliasedResultExpr};
use serde::{Deserialize, Serialize};

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectExpr<S: Scalar> {
    /// The aliased result expressions we select
    aliased_result_exprs: Vec<AliasedResultExpr>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Scalar> SelectExpr<S> {
    /// Create a new `SelectExpr` node.
    pub fn new(aliased_result_exprs: Vec<AliasedResultExpr>) -> Self {
        Self {
            aliased_result_exprs,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Scalar> PostprocessingStep<S> for SelectExpr<S> {
    /// Apply the select transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let cols: IndexMap<Identifier, OwnedColumn<S>> = self.aliased_result_exprs
            .iter()
            .map(|aliased_result_expr| -> PostprocessingResult<(Identifier, OwnedColumn<S>)> {
                let evaluator = PostprocessingEvaluator::new(owned_table.clone(), false);
                let result_column = evaluator.build(aliased_result_expr.expr)?;
                Ok((aliased_result_expr.alias, result_column))
            })
            .collect::<PostprocessingResult<_>>()?;
        OwnedTable::try_new(cols)
    }
}
