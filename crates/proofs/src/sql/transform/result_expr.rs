use crate::base::database::{dataframe_to_record_batch, record_batch_to_dataframe};
use crate::sql::proof::TransformExpr;
use crate::sql::transform::DataFrameExpr;

use arrow::record_batch::RecordBatch;
use dyn_partial_eq::DynPartialEq;
use polars::prelude::IntoLazy;

/// The result expression is used to transform the results of a query
///
/// Note: both the `transformation` and `result_schema` are
/// mutually exclusive operations. So they must not be set at the same time.
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct ResultExpr {
    transformation: Box<dyn DataFrameExpr>,
}

impl ResultExpr {
    /// Create a new `ResultExpr` node with the provided transformation to be applied to the input record batch.
    pub fn new(transformation: Box<dyn DataFrameExpr>) -> Self {
        Self { transformation }
    }
}

impl TransformExpr for ResultExpr {
    /// Transform the `RecordBatch` result of a query using the `transformation` expression
    fn transform_results(&self, result_batch: RecordBatch) -> RecordBatch {
        transform_the_record_batch(result_batch, self.transformation.as_ref())
    }
}

/// Transform the input `RecordBatch` using the provided `transformation` expression
fn transform_the_record_batch(
    result_batch: RecordBatch,
    transformation: &dyn DataFrameExpr,
) -> RecordBatch {
    let lazy_frame = record_batch_to_dataframe(result_batch).lazy();
    let lazy_frame = transformation.apply_transformation(lazy_frame);

    dataframe_to_record_batch(
        lazy_frame
            .collect()
            .expect("All transformations must have been validated"),
    )
}
