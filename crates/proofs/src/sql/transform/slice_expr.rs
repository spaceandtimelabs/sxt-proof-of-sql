#[allow(deprecated)]
use super::DataFrameExpr;
use dyn_partial_eq::DynPartialEq;
use polars::prelude::LazyFrame;
use serde::{Deserialize, Serialize};

/// A `SliceExpr` represents a slice of a `LazyFrame`.
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct SliceExpr {
    /// number of rows to return
    ///
    /// - if u64::MAX, specify all rows
    number_rows: u64,

    /// number of rows to skip
    ///
    /// - if 0, specify the first row as starting point
    /// - if negative, specify the offset from the end
    ///   (e.g. -1 is the last row, -2 is the second to last row, etc.)
    offset_value: i64,
}

impl SliceExpr {
    /// Create a new `SliceExpr` with the given `number_rows` and `offset`.
    pub fn new(number_rows: u64, offset_value: i64) -> Self {
        Self {
            number_rows,
            offset_value,
        }
    }
}

super::record_batch_expr::impl_record_batch_expr_for_data_frame_expr!(SliceExpr);
#[allow(deprecated)]
impl DataFrameExpr for SliceExpr {
    /// Apply the slice transformation to the given `LazyFrame`.
    fn lazy_transformation(&self, lazy_frame: LazyFrame, _: usize) -> LazyFrame {
        lazy_frame.slice(self.offset_value, self.number_rows)
    }
}
