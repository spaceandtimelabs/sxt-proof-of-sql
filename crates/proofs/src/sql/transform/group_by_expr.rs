use super::DataFrameExpr;

use dyn_partial_eq::DynPartialEq;
use polars::prelude::Expr;
use polars::prelude::LazyFrame;

/// A group by expression
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct GroupByExpr {
    /// A list of aggregation column expressions
    agg_exprs: Vec<Expr>,

    /// A list of group by column expressions
    by_exprs: Vec<Expr>,
}

impl GroupByExpr {
    /// Create a new group by expression containing the group by and aggregation expressions
    pub fn new(by_exprs: Vec<Expr>, agg_exprs: Vec<Expr>) -> Self {
        assert!(
            !by_exprs.is_empty(),
            "Group by expressions must not be empty"
        );

        Self {
            by_exprs,
            agg_exprs,
        }
    }
}

impl DataFrameExpr for GroupByExpr {
    fn apply_transformation(&self, lazy_frame: LazyFrame) -> LazyFrame {
        // We use `groupby_stable` instead of `groupby`
        // to avoid non-deterministic results with our tests.
        lazy_frame
            .groupby_stable(&self.by_exprs)
            .agg(&self.agg_exprs)
    }
}
