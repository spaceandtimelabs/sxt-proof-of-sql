use polars::prelude::LazyFrame;
use std::fmt::Debug;

/// A trait for nodes that can apply transformations to a `LazyFrame`.
#[deprecated = "Use `RecordBatchExpr` instead"]
pub trait DataFrameExpr: Debug + Send + Sync {
    /// Apply the transformation to the `LazyFrame` and return the result.
    fn lazy_transformation(&self, lazy_frame: LazyFrame, num_input_rows: usize) -> LazyFrame;
}
