use super::DataFrameExpr;

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
    pub fn new(result_schema: Vec<Expr>) -> Self {
        assert!(!result_schema.is_empty());
        Self { result_schema }
    }
}

#[typetag::serde]
impl DataFrameExpr for SelectExpr {
    /// Apply the select transformation to the lazy frame
    fn apply_transformation(&self, lazy_frame: LazyFrame, _: usize) -> LazyFrame {
        lazy_frame.select(&self.result_schema)
    }
}
