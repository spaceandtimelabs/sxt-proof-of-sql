use super::{evaluate_expr, PostprocessingResult, PostprocessingStep};
use ahash::AHasher;
use alloc::vec::Vec;
use core::hash::BuildHasherDefault;
use datafusion::logical_expr::Expr;
use indexmap::IndexMap;
use proof_of_sql::base::{
    database::{OwnedColumn, OwnedTable},
    scalar::Scalar,
};
use sqlparser::ast::Ident;

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, Clone, PartialEq)]
pub struct SelectPostprocessing {
    /// The expressions we select
    exprs: Vec<Expr>,
}

impl SelectPostprocessing {
    /// Create a new `SelectPostprocessing` node.
    #[must_use]
    pub fn new(exprs: Vec<Expr>) -> Self {
        Self { exprs }
    }
}

impl<S: Scalar> PostprocessingStep<S> for SelectPostprocessing {
    /// Apply the select transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let cols: IndexMap<Ident, OwnedColumn<S>, BuildHasherDefault<AHasher>> = self
            .exprs
            .iter()
            .map(|expr| -> PostprocessingResult<(Ident, OwnedColumn<S>)> {
                let result_column = evaluate_expr(&owned_table, expr)?;
                Ok((expr.display_name()?.as_str().into(), result_column))
            })
            .collect::<PostprocessingResult<_>>()?;
        Ok(OwnedTable::try_new(cols)?)
    }
}
