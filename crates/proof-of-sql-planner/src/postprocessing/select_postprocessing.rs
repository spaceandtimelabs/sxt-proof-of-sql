use super::{evaluate_expr, PostprocessingResult, PostprocessingStep};
use proof_of_sql::base::{
    database::{OwnedColumn, OwnedTable},
    map::IndexMap,
    scalar::Scalar,
};
use alloc::vec::Vec;
use datafusion::logical_expr::Alias;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectPostprocessing {
    /// The aliased expressions we select
    aliased_exprs: Vec<Alias>,
}

impl SelectPostprocessing {
    /// Create a new `SelectPostprocessing` node.
    #[must_use]
    pub fn new(aliased_exprs: Vec<Alias>) -> Self {
        Self { aliased_exprs }
    }
}

impl<S: Scalar> PostprocessingStep<S> for SelectPostprocessing {
    /// Apply the select transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let cols: IndexMap<Ident, OwnedColumn<S>> = self
            .aliased_exprs
            .iter()
            .map(
                |aliased_expr| -> PostprocessingResult<(Ident, OwnedColumn<S>)> {
                    let result_column = evaluate_expr(&owned_table, &aliased_expr.expr)?;
                    Ok((aliased_expr.name.into(), result_column))
                },
            )
            .collect::<PostprocessingResult<_>>()?;
        Ok(OwnedTable::try_new(cols)?)
    }
}
