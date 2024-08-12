use super::{PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{OwnedColumn, OwnedTable},
    scalar::Scalar,
};
use indexmap::IndexMap;
use proof_of_sql_parser::{intermediate_ast::AliasedResultExpr, Identifier};
use serde::{Deserialize, Serialize};

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectPostprocessing {
    /// The aliased result expressions we select
    aliased_result_exprs: Vec<AliasedResultExpr>,
}

impl SelectPostprocessing {
    /// Create a new `SelectPostprocessing` node.
    pub fn new(aliased_result_exprs: Vec<AliasedResultExpr>) -> Self {
        Self {
            aliased_result_exprs,
        }
    }
}

impl<S: Scalar> PostprocessingStep<S> for SelectPostprocessing {
    /// Apply the select transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let cols: IndexMap<Identifier, OwnedColumn<S>> = self
            .aliased_result_exprs
            .iter()
            .map(
                |aliased_result_expr| -> PostprocessingResult<(Identifier, OwnedColumn<S>)> {
                    let result_column = owned_table.evaluate(&aliased_result_expr.expr)?;
                    Ok((aliased_result_expr.alias, result_column))
                },
            )
            .collect::<PostprocessingResult<_>>()?;
        Ok(OwnedTable::try_new(cols)?)
    }
}
