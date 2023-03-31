use dyn_partial_eq::dyn_partial_eq;
use polars::prelude::LazyFrame;
use std::fmt::Debug;

/// A trait for nodes that can apply transformations to a `LazyFrame`.
#[dyn_partial_eq]
pub trait DataFrameExpr: Debug + Send + Sync {
    fn apply_transformation(&self, lazy_frame: LazyFrame) -> LazyFrame;
}
