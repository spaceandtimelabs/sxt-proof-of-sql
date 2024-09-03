use super::{
    DenseFilterExec, DynProofPlan, FilterExec, FilterResultExpr, GroupByExec, ProjectionExec,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{SchemaAccessor, TableRef},
    },
    sql::proof_exprs::{test_utility::*, AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
};
use proof_of_sql_parser::intermediate_ast::AggregationOperator;

pub fn col_result(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> FilterResultExpr {
    FilterResultExpr::new(col_ref(tab, name, accessor))
}

pub fn cols_result(
    tab: TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<FilterResultExpr> {
    names
        .iter()
        .map(|name| col_result(tab, name, accessor))
        .collect()
}

pub fn filter<C: Commitment>(
    results: Vec<FilterResultExpr>,
    table: TableExpr,
    where_clause: DynProofExpr<C>,
) -> DynProofPlan<C> {
    DynProofPlan::Filter(FilterExec::new(results, table, where_clause))
}

pub fn projection<C: Commitment>(
    results: Vec<AliasedDynProofExpr<C>>,
    table: TableExpr,
) -> DynProofPlan<C> {
    DynProofPlan::Projection(ProjectionExec::new(results, table))
}

pub fn dense_filter<C: Commitment>(
    results: Vec<AliasedDynProofExpr<C>>,
    table: TableExpr,
    where_clause: DynProofExpr<C>,
) -> DynProofPlan<C> {
    DynProofPlan::DenseFilter(DenseFilterExec::new(results, table, where_clause))
}

pub fn sum_expr<C: Commitment>(expr: DynProofExpr<C>, alias: &str) -> AliasedDynProofExpr<C> {
    AliasedDynProofExpr {
        expr: DynProofExpr::new_aggregate(AggregationOperator::Sum, expr),
        alias: alias.parse().unwrap(),
    }
}

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
