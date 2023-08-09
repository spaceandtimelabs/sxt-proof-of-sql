use super::DataFrameExpr;
// use polars::lazy::dsl::AggExpr;
use proofs_sql::{
    intermediate_ast::{AggExpr, ResultColumn},
    Identifier,
};

use dyn_partial_eq::DynPartialEq;
use polars::prelude::col;
use polars::prelude::Expr;
use polars::prelude::LazyFrame;
use std::collections::HashSet;

/// A prefix to add to the group by alias column not appearing in the final select result clause.
///
/// Note: this prefix is used to avoid name collisions with the aggregation column aliases.
/// For example: `select count(a) as a from table group by a`
///
/// Note: this prefix must never appear in a column identifier alias as it doesn't contain valid characters.
const NON_RESULT_BY_EXPR_PREFIX: &str = "#$";

/// A group by expression
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct GroupByExpr {
    /// A list of aggregation column expressions
    agg_exprs: Vec<Expr>,

    /// A list of group by column expressions
    by_exprs: Vec<Expr>,
}

impl GroupByExpr {
    /// Create a new group by expression containing the group by and aggregation expressions to transform the lazy frame.
    ///
    /// Parameters:
    ///
    /// - `by_exprs`: A non-empty list of group by expressions. Each element is composed by a
    ///    tuple where the first element is the `column name` and the second is an `alias in potential`.
    ///    If this second element is `None`, then the column is filtered out by some consecutive `SelectExpr` transformation.
    ///    If this second element is not `None`, then the column is selected by some consecutive `SelectExpr` transformation.
    ///
    /// - `agg_exprs`: A list of aggregation expressions.
    ///
    /// Note: Duplicated aliases are not allowed.
    pub fn new(by_exprs: Vec<(Identifier, Option<Identifier>)>, agg_exprs: Vec<AggExpr>) -> Self {
        let (by_exprs, by_exprs_set, count_by_expr_aliased) = by_exprs_to_polars_exprs(by_exprs);

        assert!(
            count_by_expr_aliased + agg_exprs.len() > 0,
            "No result column expressions found"
        );

        let agg_exprs = agg_exprs_to_polars_exprs(agg_exprs, &by_exprs_set);

        Self {
            by_exprs,
            agg_exprs,
        }
    }
}

impl DataFrameExpr for GroupByExpr {
    fn apply_transformation(&self, lazy_frame: LazyFrame) -> LazyFrame {
        // We use `groupby_stable` instead of `groupby` to avoid non-deterministic results with our tests.
        lazy_frame
            .groupby_stable(&self.by_exprs)
            .agg(&self.agg_exprs)
    }
}

/// Convert a list of group by expressions to a list of polars group by expressions
fn by_exprs_to_polars_exprs(
    by_exprs: Vec<(Identifier, Option<Identifier>)>,
) -> (Vec<Expr>, HashSet<String>, usize) {
    let mut count_by_expr_aliased = 0;
    let mut by_exprs_set = HashSet::new();

    assert!(!by_exprs.is_empty());

    let by_exprs = by_exprs
        .iter()
        .map(|(name, alias)| {
            // To avoid name collisions with the aggregation column aliases, we add a `NON_RESULT_BY_EXPR_PREFIX` prefix to the alias
            let alias = alias
                .map(|id| {
                    count_by_expr_aliased += 1;

                    id.as_str().to_string()
                })
                .unwrap_or(NON_RESULT_BY_EXPR_PREFIX.to_owned() + name.as_str());
            let by_expr_col = col(name.as_str()).alias(&alias);

            assert!(
                by_exprs_set.insert(alias.to_string()),
                "Duplicated group by alias not allowed: {alias}"
            );

            by_expr_col
        })
        .collect::<Vec<_>>();

    (by_exprs, by_exprs_set, count_by_expr_aliased)
}

/// Convert a list of aggregation expressions to a list of polars aggregation expressions
fn agg_exprs_to_polars_exprs(agg_exprs: Vec<AggExpr>, by_exprs_set: &HashSet<String>) -> Vec<Expr> {
    let mut agg_exprs_set = HashSet::new();

    let agg_exprs = agg_exprs
        .iter()
        .map(|agg_expr| {
            let (agg_expr_col, alias) = match agg_expr {
                AggExpr::Max(ResultColumn { name, alias }) => {
                    (col(name.as_str()).max().alias(alias.as_str()), alias)
                }
                AggExpr::Min(ResultColumn { name, alias }) => {
                    (col(name.as_str()).min().alias(alias.as_str()), alias)
                }
                AggExpr::Sum(ResultColumn { name, alias }) => {
                    // Note that the following aggregation `sum` may result in overflow.
                    // In debug mode, Polars will raise a panic if an overflow occurs,
                    // while in release mode, it will silently return the overflowed result.
                    (col(name.as_str()).sum().alias(alias.as_str()), alias)
                }
                AggExpr::Count(ResultColumn { name, alias }) => {
                    (col(name.as_str()).count().alias(alias.as_str()), alias)
                }
                _ => panic!("Unsupported aggregation expression: {:#?}", agg_expr),
            };

            assert!(
                agg_exprs_set.insert(*alias),
                "Duplicated aggregation alias not allowed: {alias}"
            );
            assert!(
                !by_exprs_set.contains(alias.as_str()),
                "Duplicated aggregation alias not allowed: {alias}"
            );

            agg_expr_col
        })
        .collect::<Vec<_>>();

    agg_exprs
}
