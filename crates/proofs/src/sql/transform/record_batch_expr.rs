use arrow::record_batch::RecordBatch;
use dyn_partial_eq::dyn_partial_eq;
use std::fmt::Debug;

/// A trait for nodes that can apply transformations to a `RecordBatch`.
#[typetag::serde(tag = "type")]
#[dyn_partial_eq]
pub trait RecordBatchExpr: Debug + Send + Sync {
    /// Apply the transformation to the `RecordBatch` and return the result.
    fn apply_transformation(&self, record_batch: RecordBatch) -> Option<RecordBatch>;
}

macro_rules! impl_record_batch_expr_for_data_frame_expr {
    ($t:ty) => {
        #[typetag::serde]
        impl crate::sql::transform::record_batch_expr::RecordBatchExpr for $t {
            fn apply_transformation(
                &self,
                record_batch: arrow::record_batch::RecordBatch,
            ) -> Option<arrow::record_batch::RecordBatch> {
                let (lazy_frame, num_input_rows) =
                    crate::sql::transform::result_expr::record_batch_to_lazy_frame(record_batch)?;
                #[allow(deprecated)]
                crate::sql::transform::result_expr::lazy_frame_to_record_batch(
                    self.lazy_transformation(lazy_frame, num_input_rows),
                )
            }
        }
    };
}

pub(crate) use impl_record_batch_expr_for_data_frame_expr;
