use super::{DynProofPlan, FilterExec, GroupByExec, ProjectionExec};
use crate::{
    base::commitment::Commitment,
    sql::proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
};

pub fn projection<C: Commitment>(
    results: Vec<AliasedDynProofExpr<C>>,
    table: TableExpr,
) -> DynProofPlan<C> {
    DynProofPlan::Projection(ProjectionExec::new(results, table))
}

pub fn filter<C: Commitment>(
    results: Vec<AliasedDynProofExpr<C>>,
    table: TableExpr,
    where_clause: DynProofExpr<C>,
) -> DynProofPlan<C> {
    DynProofPlan::Filter(FilterExec::new(results, table, where_clause))
}

/// # Panics
///
/// Will panic if `count_alias` cannot be parsed as a valid identifier.
pub fn group_by<C: Commitment>(
    group_by_exprs: Vec<ColumnExpr<C>>,
    sum_expr: Vec<AliasedDynProofExpr<C>>,
    count_alias: &str,
    table: TableExpr,
    where_clause: DynProofExpr<C>,
) -> DynProofPlan<C> {
    DynProofPlan::GroupBy(GroupByExec::new(
        group_by_exprs,
        sum_expr,
        count_alias.parse().unwrap(),
        table,
        where_clause,
    ))
}
