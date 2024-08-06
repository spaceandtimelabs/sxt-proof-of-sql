use super::{OrderByExpr, PostprocessingResult, PostprocessingStep, SliceExpr};
use crate::base::{database::OwnedTable, scalar::Scalar};

/// An enum for nodes that can apply postprocessing to a `OwnedTable`.
#[derive(Debug, Clone)]
pub enum OwnedTablePostprocessing {
    /// Slice the `OwnedTable` with the given `SliceExpr`.
    Slice(SliceExpr),
    /// Order the `OwnedTable` with the given `OrderByExpr`.
    OrderBy(OrderByExpr),
}

impl<S: Scalar> PostprocessingStep<S> for OwnedTablePostprocessing {
    /// Apply the postprocessing step to the `OwnedTable` and return the result.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        match self {
            OwnedTablePostprocessing::Slice(slice_expr) => slice_expr.apply(owned_table),
            OwnedTablePostprocessing::OrderBy(order_by_expr) => order_by_expr.apply(owned_table),
        }
    }
}

impl OwnedTablePostprocessing {
    /// Create a new `OwnedTablePostprocessing` with the given `SliceExpr`.
    pub fn new_slice(slice_expr: SliceExpr) -> Self {
        Self::Slice(slice_expr)
    }
    /// Create a new `OwnedTablePostprocessing` with the given `OrderByExpr`.
    pub fn new_order_by(order_by_expr: OrderByExpr) -> Self {
        Self::OrderBy(order_by_expr)
    }
}

/// Apply a list of postprocessing steps to an `OwnedTable`.
pub fn apply_postprocessing_steps<S: Scalar>(
    owned_table: OwnedTable<S>,
    postprocessing_steps: &[OwnedTablePostprocessing],
) -> PostprocessingResult<OwnedTable<S>> {
    // Sadly try_fold() only works on Options
    let mut current_table = owned_table;
    for step in postprocessing_steps {
        current_table = step.apply(current_table)?;
    }
    Ok(current_table)
}
