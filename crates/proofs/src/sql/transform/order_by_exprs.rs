use super::DataFrameExpr;
use proofs_sql::intermediate_ast::{OrderBy, OrderByDirection};

use dyn_partial_eq::DynPartialEq;
use polars::prelude::col;
use polars::prelude::LazyFrame;

/// A node representing a list of `OrderBy` expressions.
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct OrderByExprs {
    by_exprs: Vec<OrderBy>,
}

impl OrderByExprs {
    /// Create a new `OrderByExprs` node.
    pub fn new(by_exprs: Vec<OrderBy>) -> Self {
        Self { by_exprs }
    }
}

impl DataFrameExpr for OrderByExprs {
    /// Sort the `LazyFrame` by the `OrderBy` expressions.
    fn apply_transformation(&self, lazy_frame: LazyFrame) -> LazyFrame {
        assert!(!self.by_exprs.is_empty());

        let maintain_order = true;
        let nulls_last = false;
        let reverse: Vec<_> = self
            .by_exprs
            .iter()
            .map(|v| v.direction == OrderByDirection::Desc)
            .collect();
        let by_column: Vec<_> = self.by_exprs.iter().map(|v| col(v.expr.name())).collect();

        lazy_frame.sort_by_exprs(by_column, reverse, nulls_last, maintain_order)
    }
}
