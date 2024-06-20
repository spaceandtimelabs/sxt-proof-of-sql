#[allow(deprecated)]
#[cfg(feature = "polars")]
use super::DataFrameExpr;
use super::RecordBatchExpr;
#[cfg(feature = "polars")]
use super::{
    result_expr::{lazy_frame_to_record_batch, record_batch_to_lazy_frame},
    ToPolarsExpr,
};
use arrow::record_batch::RecordBatch;
use dyn_partial_eq::DynPartialEq;
#[cfg(feature = "polars")]
use polars::prelude::{Expr, LazyFrame};
use proof_of_sql_parser::intermediate_ast::{AliasedResultExpr, Expression};
use serde::{Deserialize, Serialize};

#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub enum SelectTerm {
    #[cfg(test)]
    #[cfg(feature = "polars")]
    Polars(Expr),
    AliasedResult(AliasedResultExpr),
    Result(Expression),
}
#[cfg(feature = "polars")]
impl ToPolarsExpr for SelectTerm {
    fn to_polars_expr(&self) -> Expr {
        match self {
            #[cfg(test)]
            Self::Polars(s) => s.to_polars_expr(),
            Self::AliasedResult(s) => s.to_polars_expr(),
            Self::Result(s) => s.to_polars_expr(),
        }
    }
}
#[cfg(test)]
#[cfg(feature = "polars")]
impl From<&Expr> for SelectTerm {
    fn from(value: &Expr) -> Self {
        Self::Polars(value.clone())
    }
}
impl From<&AliasedResultExpr> for SelectTerm {
    fn from(value: &AliasedResultExpr) -> Self {
        Self::AliasedResult(value.clone())
    }
}
impl From<&Expression> for SelectTerm {
    fn from(value: &Expression) -> Self {
        Self::Result(value.clone())
    }
}
#[cfg(test)]
impl From<&Box<Expression>> for SelectTerm {
    fn from(value: &Box<Expression>) -> Self {
        value.as_ref().into()
    }
}

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct SelectExpr {
    /// The schema of the resulting lazy frame
    result_schema: Vec<SelectTerm>,
}

impl SelectExpr {
    /// Create a new select expression from a slice that implements `Into<SelectTerm>`
    pub fn new(exprs: impl IntoIterator<Item = impl Into<SelectTerm>>) -> Self {
        let result_schema = Vec::from_iter(exprs.into_iter().map(Into::into));
        assert!(!result_schema.is_empty());
        Self { result_schema }
    }
}

#[allow(deprecated)]
#[cfg(feature = "polars")]
impl DataFrameExpr for SelectExpr {
    /// Apply the select transformation to the lazy frame
    fn lazy_transformation(&self, lazy_frame: LazyFrame, _: usize) -> LazyFrame {
        lazy_frame.select(&Vec::from_iter(
            self.result_schema.iter().map(ToPolarsExpr::to_polars_expr),
        ))
    }
}

#[typetag::serde]
impl RecordBatchExpr for SelectExpr {
    fn apply_transformation(&self, record_batch: RecordBatch) -> Option<RecordBatch> {
        let easy_result: Option<Vec<_>> = self
            .result_schema
            .iter()
            .map(|expr| match expr {
                SelectTerm::AliasedResult(AliasedResultExpr { expr, alias }) => match **expr {
                    Expression::Column(c) if &c == alias => {
                        Some((c, record_batch.column_by_name(c.as_str())?.to_owned()))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();

        if let Some(Ok(result)) = easy_result.map(RecordBatch::try_from_iter) {
            return Some(result);
        }
        #[cfg(feature = "polars")]
        {
            let (lazy_frame, num_input_rows) = record_batch_to_lazy_frame(record_batch)?;
            #[allow(deprecated)]
            lazy_frame_to_record_batch(self.lazy_transformation(lazy_frame, num_input_rows))
        }
        #[cfg(not(feature = "polars"))]
        {
            None
        }
    }
}
