use crate::sql::transform::RecordBatchExpr;
use arrow::record_batch::RecordBatch;
use dyn_partial_eq::DynPartialEq;
use serde::{Deserialize, Serialize};

/// A node representing a list of transformations to be applied to a `LazyFrame`.
#[derive(Debug, Default, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct CompositionExpr {
    transformations: Vec<Box<dyn RecordBatchExpr>>,
}

impl CompositionExpr {
    /// Create a new `CompositionExpr` node.
    pub fn new(transformation: Box<dyn RecordBatchExpr>) -> Self {
        Self {
            transformations: vec![transformation],
        }
    }

    /// Verify if the `CompositionExpr` node is empty.
    pub fn is_empty(&self) -> bool {
        self.transformations.is_empty()
    }

    /// Append a new transformation to the end of the current `CompositionExpr` node.
    pub fn add(&mut self, transformation: Box<dyn RecordBatchExpr>) {
        self.transformations.push(transformation);
    }
}

#[typetag::serde]
impl RecordBatchExpr for CompositionExpr {
    /// Apply the transformations to the `RecordBatch`.
    fn apply_transformation(&self, record_batch: RecordBatch) -> Option<RecordBatch> {
        let mut record_batch = record_batch;

        for transformation in self.transformations.iter() {
            record_batch = transformation.apply_transformation(record_batch)?;
        }

        Some(record_batch)
    }
}
