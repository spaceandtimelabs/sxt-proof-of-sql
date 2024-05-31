#[allow(deprecated)]
use super::DataFrameExpr;
use super::ToPolarsExpr;
use crate::base::database::{INT128_PRECISION, INT128_SCALE};
use dyn_partial_eq::DynPartialEq;
use polars::prelude::{col, DataType, Expr, GetOutput, LazyFrame, NamedFrom, Series};
use proofs_sql::{intermediate_ast::AliasedResultExpr, Identifier};
use serde::{Deserialize, Serialize};

/// A group by expression
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct GroupByExpr {
    /// A list of aggregation column expressions
    agg_exprs: Vec<Expr>,

    /// A list of group by column expressions
    by_exprs: Vec<Expr>,
}

impl GroupByExpr {
    /// Create a new group by expression containing the group by and aggregation expressions
    pub fn new(by_ids: &[Identifier], aliased_exprs: &[AliasedResultExpr]) -> Self {
        let by_exprs = Vec::from_iter(by_ids.iter().map(|id| col(id.as_str())));
        let agg_exprs = Vec::from_iter(aliased_exprs.iter().map(ToPolarsExpr::to_polars_expr));
        assert!(!agg_exprs.is_empty(), "Agg expressions must not be empty");
        assert!(
            !by_exprs.is_empty(),
            "Group by expressions must not be empty"
        );

        Self {
            by_exprs,
            agg_exprs,
        }
    }
}

super::impl_record_batch_expr_for_data_frame_expr!(GroupByExpr);
#[allow(deprecated)]
impl DataFrameExpr for GroupByExpr {
    fn lazy_transformation(&self, lazy_frame: LazyFrame, num_input_rows: usize) -> LazyFrame {
        // TODO: polars currently lacks support for min/max aggregation in data frames
        // with either zero or one element when a group by operation is applied.
        // We remove the group by clause to temporarily work around this limitation.
        // Issue created to track progress: https://github.com/pola-rs/polars/issues/11232
        if num_input_rows == 0 {
            return lazy_frame.select(&self.agg_exprs).limit(0);
        }

        if num_input_rows == 1 {
            return lazy_frame.select(&self.agg_exprs);
        }

        // Add invalid column aliases to group by expressions so that we can
        // exclude them from the final result.
        let by_expr_aliases = (0..self.by_exprs.len())
            .map(|pos| "#$".to_owned() + pos.to_string().as_str())
            .collect::<Vec<_>>();

        let by_exprs: Vec<_> = self
            .by_exprs
            .clone()
            .into_iter()
            .zip(by_expr_aliases.iter())
            .map(|(expr, alias)| expr.alias(alias.as_str()))
            // TODO: remove this mapping once Polars supports decimal columns inside group by
            // Issue created to track progress: https://github.com/pola-rs/polars/issues/11078
            .map(group_by_map_to_utf8_if_decimal)
            .collect();

        // We use `groupby_stable` instead of `groupby`
        // to avoid non-deterministic results with our tests.
        lazy_frame
            .group_by_stable(&by_exprs)
            .agg(&self.agg_exprs)
            .select(&[col("*").exclude(by_expr_aliases)])
    }
}

pub(crate) fn group_by_map_i128_to_utf8(v: i128) -> String {
    // use big end to allow
    // skipping leading zeros
    v.to_be_bytes()
        .into_iter()
        // skip leading zeros
        .skip_while(|x| *x == 0)
        // in the worst case scenario,
        // 16 bytes per decimal
        // is mapped to 32 bytes per char
        // this is not ideal.
        // but keeping as it is for now
        // since this must be a temporary solution.
        .map(char::from)
        // Using `Binary` type would consume less space
        // But it would be an issue when we try to convert
        // the result data frame into a record batch
        // since we don't support this data type.
        .collect::<String>()
}

// Polars doesn't support Decimal columns inside group by.
// So we need to remap them to the supported UTF8 type.
fn group_by_map_to_utf8_if_decimal(expr: Expr) -> Expr {
    expr.map(
        |series| match series.dtype().clone() {
            DataType::Decimal(Some(INT128_PRECISION), Some(INT128_SCALE)) => {
                let utf8_data: Vec<_> = series
                    .decimal()
                    .unwrap()
                    .into_no_null_iter()
                    .map(group_by_map_i128_to_utf8)
                    .collect();
                Ok(Some(Series::new(series.name(), &utf8_data)))
            }
            _ => Ok(Some(series)),
        },
        GetOutput::from_type(DataType::Utf8),
    )
}
