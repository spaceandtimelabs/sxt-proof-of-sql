use crate::{
    base::database::{dataframe_to_record_batch, record_batch_to_dataframe},
    sql::{proof::TransformExpr, transform::RecordBatchExpr},
};
use arrow::record_batch::RecordBatch;
use dyn_partial_eq::DynPartialEq;
use polars::prelude::{IntoLazy, LazyFrame};
use serde::{Deserialize, Serialize};

/// The result expression is used to transform the results of a query
///
/// Note: both the `transformation` and `result_schema` are
/// mutually exclusive operations. So they must not be set at the same time.
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct ResultExpr {
    transformation: Box<dyn RecordBatchExpr>,
}

impl ResultExpr {
    /// Create a new `ResultExpr` node with the provided transformation to be applied to the input record batch.
    pub fn new(transformation: Box<dyn RecordBatchExpr>) -> Self {
        Self { transformation }
    }
}

pub(super) fn record_batch_to_lazy_frame(result_batch: RecordBatch) -> Option<(LazyFrame, usize)> {
    let num_input_rows = result_batch.num_rows();
    let df = record_batch_to_dataframe(result_batch)?;
    Some((df.lazy(), num_input_rows))
}
pub(super) fn lazy_frame_to_record_batch(lazy_frame: LazyFrame) -> Option<RecordBatch> {
    // We're currently excluding NULLs in post-processing due to a lack of
    // prover support, aiming to avoid future complexities.
    // The drawback is that users won't get NULL results in aggregations on empty data.
    // For example, the query `SELECT MAX(i), COUNT(i), SUM(i), MIN(i) FROM table WHERE s = 'nonexist'`
    // will now omit the entire row (before, it would return `null, 0, 0, null`).
    // This choice is acceptable, as `SELECT MAX(i), COUNT(i), SUM(i) FROM table WHERE s = 'nonexist' GROUP BY f`
    // has the same outcome.
    //
    // TODO: Revisit if we add NULL support to the prover.
    let lazy_frame = lazy_frame.drop_nulls(None);

    dataframe_to_record_batch(lazy_frame.collect().ok()?)
}

impl TransformExpr for ResultExpr {
    /// Transform the `RecordBatch` result of a query using the `transformation` expression
    fn transform_results(&self, result_batch: RecordBatch) -> Option<RecordBatch> {
        self.transformation.apply_transformation(result_batch)
    }
}
