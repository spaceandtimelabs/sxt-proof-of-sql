use super::DataFrameExpr;
// use polars::lazy::dsl::AggExpr;
use proofs_sql::intermediate_ast::ResultColumn;

use dyn_partial_eq::DynPartialEq;
use polars::prelude::col;
use polars::prelude::Expr;
use polars::prelude::LazyFrame;
use std::collections::HashSet;

/// The select expression used to select, reorder, and apply alias transformations to the columns of a lazy frame
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct SelectExpr {
    /// The schema of the resulting lazy frame
    result_schema: Vec<Expr>,
}

impl SelectExpr {
    /// This function creates a new SelectExpr node for the lazy frame,
    /// so that the lazy frame column named `result_schema[i].name`
    /// is mapped to the named `result_schema[i].alias` column.
    pub fn new(result_schema: Vec<ResultColumn>) -> Self {
        let mut result_schema_set = HashSet::new();

        assert!(!result_schema.is_empty());

        let result_schema = result_schema
            .iter()
            .map(|id| {
                let alias = id.alias.as_str();

                assert!(
                    result_schema_set.insert(alias),
                    "Duplicated alias not allowed: {alias}"
                );

                col(id.name.as_str()).alias(id.alias.as_str())
            })
            .collect::<Vec<_>>();

        Self { result_schema }
    }
}

impl DataFrameExpr for SelectExpr {
    /// Apply the select transformation to the lazy frame
    fn apply_transformation(&self, lazy_frame: LazyFrame) -> LazyFrame {
        lazy_frame.select(&self.result_schema)
    }
}
