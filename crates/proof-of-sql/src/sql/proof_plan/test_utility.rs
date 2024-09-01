use super::{
    AliasedProvableExprPlan, ColumnExpr, DenseFilterExec, FilterExec, FilterResultExpr,
    GroupByExec, ProjectionExec, ProofPlan, ProvableExprPlan, TableExpr,
};
use crate::base::{
    commitment::Commitment,
    database::{SchemaAccessor, TableRef},
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
    where_clause: ProvableExprPlan<C>,
) -> ProofPlan<C> {
    ProofPlan::Filter(FilterExec::new(results, table, where_clause))
}

pub fn projection<C: Commitment>(
    results: Vec<AliasedProvableExprPlan<C>>,
    table: TableExpr,
) -> ProofPlan<C> {
    ProofPlan::Projection(ProjectionExec::new(results, table))
}

pub fn dense_filter<C: Commitment>(
    results: Vec<AliasedProvableExprPlan<C>>,
    table: TableExpr,
    where_clause: ProvableExprPlan<C>,
) -> ProofPlan<C> {
    ProofPlan::DenseFilter(DenseFilterExec::new(results, table, where_clause))
}

pub fn sum_expr<C: Commitment>(
    expr: ProvableExprPlan<C>,
    alias: &str,
) -> AliasedProvableExprPlan<C> {
    AliasedProvableExprPlan {
        expr: ProvableExprPlan::new_aggregate(AggregationOperator::Sum, expr),
        alias: alias.parse().unwrap(),
    }
}

pub fn group_by<C: Commitment>(
    group_by_exprs: Vec<ColumnExpr<C>>,
    sum_expr: Vec<AliasedProvableExprPlan<C>>,
    count_alias: &str,
    table: TableExpr,
    where_clause: ProvableExprPlan<C>,
) -> ProofPlan<C> {
    ProofPlan::GroupBy(GroupByExec::new(
        group_by_exprs,
        sum_expr,
        count_alias.parse().unwrap(),
        table,
        where_clause,
    ))
}
