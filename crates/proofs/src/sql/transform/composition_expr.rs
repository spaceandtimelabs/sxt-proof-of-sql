use crate::sql::transform::DataFrameExpr;
use dyn_partial_eq::DynPartialEq;
use polars::prelude::LazyFrame;
use serde::{Deserialize, Serialize};

/// A node representing a list of transformations to be applied to a `LazyFrame`.
#[derive(Debug, Default, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct CompositionExpr {
    transformations: Vec<Box<dyn DataFrameExpr>>,
}

impl CompositionExpr {
    /// Create a new `CompositionExpr` node.
    pub fn new(transformation: Box<dyn DataFrameExpr>) -> Self {
        Self {
            transformations: vec![transformation],
        }
    }

    /// Verify if the `CompositionExpr` node is empty.
    pub fn is_empty(&self) -> bool {
        self.transformations.is_empty()
    }

    /// Append a new transformation to the end of the current `CompositionExpr` node.
    pub fn add(&mut self, transformation: Box<dyn DataFrameExpr>) {
        self.transformations.push(transformation);
    }
}

#[typetag::serde]
impl DataFrameExpr for CompositionExpr {
    fn is_identity(&self) -> bool {
        self.transformations.iter().all(|t| t.is_identity())
    }
    /// Apply the transformations to the `LazyFrame`.
    fn apply_transformation(&self, lazy_frame: LazyFrame, num_input_rows: usize) -> LazyFrame {
        let mut lazy_frame = lazy_frame;

        for transformation in self.transformations.iter() {
            lazy_frame = transformation.apply_transformation(lazy_frame, num_input_rows);
        }

        lazy_frame
    }
}
