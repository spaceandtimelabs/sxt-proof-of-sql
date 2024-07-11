use super::{OrderByExpr, PostprocessingResult, PostprocessingStep, SelectExpr, SliceExpr};
use crate::base::{database::OwnedTable, scalar::Scalar};

/// An enum for nodes that can apply postprocessing to a `OwnedTable`.
#[derive(Debug, Clone)]
pub enum OwnedTablePostprocessing<S: Scalar> {
    /// Slice the `OwnedTable` with the given `SliceExpr`.
    Slice(SliceExpr<S>),
    /// Order the `OwnedTable` with the given `OrderByExpr`.
    OrderBy(OrderByExpr<S>),
    /// Select the `OwnedTable` with the given `SelectExpr`.
    Select(SelectExpr<S>),
}

impl<S: Scalar> PostprocessingStep<S> for OwnedTablePostprocessing<S> {
    /// Apply the postprocessing step to the `OwnedTable` and return the result.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        match self {
            OwnedTablePostprocessing::Slice(slice_expr) => slice_expr.apply(owned_table),
            OwnedTablePostprocessing::OrderBy(order_by_expr) => order_by_expr.apply(owned_table),
            OwnedTablePostprocessing::Select(select_expr) => select_expr.apply(owned_table),
        }
    }
}

impl<S: Scalar> OwnedTablePostprocessing<S> {
    /// Create a new `OwnedTablePostprocessing` with the given `SliceExpr`.
    pub fn new_slice(slice_expr: SliceExpr<S>) -> Self {
        Self::Slice(slice_expr)
    }
    /// Create a new `OwnedTablePostprocessing` with the given `OrderByExpr`.
    pub fn new_order_by(order_by_expr: OrderByExpr<S>) -> Self {
        Self::OrderBy(order_by_expr)
    }
    /// Create a new `OwnedTablePostprocessing` with the given `SelectExpr`.
    pub fn new_select(select_expr: SelectExpr<S>) -> Self {
        Self::Select(select_expr)
    }
}

/// Apply a list of postprocessing steps to an `OwnedTable`.
pub fn apply_postprocessing_steps<S: Scalar>(
    owned_table: OwnedTable<S>,
    postprocessing_steps: &[OwnedTablePostprocessing<S>],
) -> PostprocessingResult<OwnedTable<S>> {
    // Sadly try_fold() only works on Options
    let mut current_table = owned_table;
    for step in postprocessing_steps {
        current_table = step.apply(current_table)?;
    }
    Ok(current_table)
}
