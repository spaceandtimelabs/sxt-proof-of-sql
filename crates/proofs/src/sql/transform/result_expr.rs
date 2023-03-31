use crate::base::database::{dataframe_to_record_batch, record_batch_to_dataframe};
use crate::sql::proof::TransformExpr;
use crate::sql::transform::DataFrameExpr;

use arrow::record_batch::RecordBatch;
use dyn_partial_eq::DynPartialEq;
use polars::prelude::IntoLazy;

/// The result expression is used to transform the results of a query
#[derive(Default, Debug, DynPartialEq, PartialEq)]
pub struct ResultExpr {
    transformation: Option<Box<dyn DataFrameExpr>>,
}

impl ResultExpr {
    /// Create a new `ResultExpr` node
    pub fn new(transformation: Box<dyn DataFrameExpr>) -> Self {
        Self {
            transformation: Some(transformation),
        }
    }
}

impl TransformExpr for ResultExpr {
    /// Transform the `RecordBatch` result of a query using the `transformation` expression
    fn transform_results(&self, result: RecordBatch) -> RecordBatch {
        if self.transformation.is_none() {
            return result;
        }

        let lazy_frame = record_batch_to_dataframe(result).lazy();
        let transformation = self.transformation.as_ref().unwrap();
        let lazy_frame = transformation.apply_transformation(lazy_frame);

        dataframe_to_record_batch(
            lazy_frame
                .collect()
                .expect("All transformations should have been valid"),
        )
    }
}
