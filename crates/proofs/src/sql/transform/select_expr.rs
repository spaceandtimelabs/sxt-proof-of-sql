#[allow(deprecated)]
use super::DataFrameExpr;
use super::{
    record_batch_expr::RecordBatchExpr,
    result_expr::{lazy_frame_to_record_batch, record_batch_to_lazy_frame},
};
use arrow::record_batch::RecordBatch;
use dyn_partial_eq::DynPartialEq;
use polars::prelude::{Expr, LazyFrame};
use serde::{Deserialize, Serialize};

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct SelectExpr {
    /// The schema of the resulting lazy frame
    result_schema: Vec<Expr>,
}

impl SelectExpr {
    /// TODO: add docs
    pub fn new(result_schema: Vec<Expr>) -> Self {
        assert!(!result_schema.is_empty());
        Self { result_schema }
    }
}

#[allow(deprecated)]
impl DataFrameExpr for SelectExpr {
    /// Apply the select transformation to the lazy frame
    fn lazy_transformation(&self, lazy_frame: LazyFrame, _: usize) -> LazyFrame {
        lazy_frame.select(&self.result_schema)
    }
}

#[typetag::serde]
impl RecordBatchExpr for SelectExpr {
    fn apply_transformation(&self, record_batch: RecordBatch) -> Option<RecordBatch> {
        let easy_result: Option<Vec<_>> = self
            .result_schema
            .iter()
            .cloned()
            .map(|expr| match expr {
                Expr::Alias(a, b) => match *a {
                    Expr::Column(c) if c == b => {
                        Some((b.to_owned(), record_batch.column_by_name(&b)?.to_owned()))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();

        if let Some(Ok(result)) = easy_result.map(RecordBatch::try_from_iter) {
            return Some(result);
        }
        let (lazy_frame, num_input_rows) = record_batch_to_lazy_frame(record_batch)?;
        #[allow(deprecated)]
        lazy_frame_to_record_batch(self.lazy_transformation(lazy_frame, num_input_rows))
    }
}
