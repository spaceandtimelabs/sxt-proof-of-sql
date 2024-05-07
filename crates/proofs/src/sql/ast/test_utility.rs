use super::{
    ColumnExpr, DenseFilterExpr, FilterExpr, FilterResultExpr, GroupByExpr, ProofPlan,
    ProvableExprPlan, TableExpr,
};
use crate::base::{
    commitment::Commitment,
    database::{ColumnField, ColumnRef, ColumnType, SchemaAccessor, TableRef},
};

pub fn col(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ColumnRef::new(tab, name, type_col)
}

pub fn equal<C: Commitment, T: Into<C::Scalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &impl SchemaAccessor,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_equals(col(tab, name, accessor), val.into())
}

pub fn lte<C: Commitment, T: Into<C::Scalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &impl SchemaAccessor,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_inequality(col(tab, name, accessor), val.into(), true)
}

pub fn not<C: Commitment>(expr: ProvableExprPlan<C>) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_not(expr).unwrap()
}

pub fn and<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_and(left, right).unwrap()
}

pub fn or<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_or(left, right).unwrap()
}

pub fn const_v<C: Commitment>(val: bool) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_const_bool(val)
}

pub fn tab(tab: TableRef) -> TableExpr {
    TableExpr { table_ref: tab }
}

pub fn col_result(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> FilterResultExpr {
    FilterResultExpr::new(col(tab, name, accessor))
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
    ProofPlan::Filter(FilterExpr::new(results, table, where_clause))
}

pub fn col_expr<C: Commitment>(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> ColumnExpr<C> {
    ColumnExpr::<C>::new(col(tab, name, accessor))
}

pub fn cols_expr<C: Commitment>(
    tab: TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<ColumnExpr<C>> {
    names
        .iter()
        .map(|name| col_expr(tab, name, accessor))
        .collect()
}

pub fn dense_filter<C: Commitment>(
    results: Vec<ColumnExpr<C>>,
    table: TableExpr,
    where_clause: ProvableExprPlan<C>,
) -> DenseFilterExpr<C> {
    DenseFilterExpr::new(results, table, where_clause)
}

pub fn sum_expr<C: Commitment>(
    tab: TableRef,
    name: &str,
    alias: &str,
    column_type: ColumnType,
    accessor: &impl SchemaAccessor,
) -> (ColumnExpr<C>, ColumnField) {
    (
        col_expr(tab, name, accessor),
        ColumnField::new(alias.parse().unwrap(), column_type),
    )
}

pub fn sums_expr<C: Commitment>(
    tab: TableRef,
    names: &[&str],
    aliases: &[&str],
    column_types: &[ColumnType],
    accessor: &impl SchemaAccessor,
) -> Vec<(ColumnExpr<C>, ColumnField)> {
    names
        .iter()
        .zip(aliases.iter().zip(column_types.iter()))
        .map(|(name, (alias, column_type))| sum_expr(tab, name, alias, *column_type, accessor))
        .collect()
}

pub fn group_by<C: Commitment>(
    group_by_exprs: Vec<ColumnExpr<C>>,
    sum_expr: Vec<(ColumnExpr<C>, ColumnField)>,
    count_alias: &str,
    table: TableExpr,
    where_clause: ProvableExprPlan<C>,
) -> ProofPlan<C> {
    ProofPlan::GroupBy(GroupByExpr::new(
        group_by_exprs,
        sum_expr,
        count_alias.parse().unwrap(),
        table,
        where_clause,
    ))
}
