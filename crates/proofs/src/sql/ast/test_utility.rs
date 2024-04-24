use super::{
    BoolExprPlan, ColumnExpr, DenseFilterExpr, FilterExpr, FilterResultExpr, GroupByExpr,
    ProofPlan, TableExpr,
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
) -> BoolExprPlan<C> {
    BoolExprPlan::new_equals(col(tab, name, accessor), val.into())
}

pub fn lte<C: Commitment, T: Into<C::Scalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &impl SchemaAccessor,
) -> BoolExprPlan<C> {
    BoolExprPlan::new_inequality(col(tab, name, accessor), val.into(), true)
}

pub fn gte<C: Commitment, T: Into<C::Scalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &impl SchemaAccessor,
) -> BoolExprPlan<C> {
    BoolExprPlan::new_inequality(col(tab, name, accessor), val.into(), false)
}

pub fn not<C: Commitment>(expr: BoolExprPlan<C>) -> BoolExprPlan<C> {
    BoolExprPlan::new_not(expr)
}

pub fn and<C: Commitment>(left: BoolExprPlan<C>, right: BoolExprPlan<C>) -> BoolExprPlan<C> {
    BoolExprPlan::new_and(left, right)
}

pub fn or<C: Commitment>(left: BoolExprPlan<C>, right: BoolExprPlan<C>) -> BoolExprPlan<C> {
    BoolExprPlan::new_or(left, right)
}

pub fn const_v<C: Commitment>(val: bool) -> BoolExprPlan<C> {
    BoolExprPlan::new_const_bool(val)
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
    where_clause: BoolExprPlan<C>,
) -> ProofPlan<C> {
    ProofPlan::Filter(FilterExpr::new(results, table, where_clause))
}

pub fn col_expr(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnExpr {
    ColumnExpr::new(col(tab, name, accessor))
}

pub fn cols_expr(tab: TableRef, names: &[&str], accessor: &impl SchemaAccessor) -> Vec<ColumnExpr> {
    names
        .iter()
        .map(|name| col_expr(tab, name, accessor))
        .collect()
}

pub fn dense_filter<C: Commitment>(
    results: Vec<ColumnExpr>,
    table: TableExpr,
    where_clause: BoolExprPlan<C>,
) -> DenseFilterExpr<C> {
    DenseFilterExpr::new(results, table, where_clause)
}

pub fn sum_expr(
    tab: TableRef,
    name: &str,
    alias: &str,
    column_type: ColumnType,
    accessor: &impl SchemaAccessor,
) -> (ColumnExpr, ColumnField) {
    (
        col_expr(tab, name, accessor),
        ColumnField::new(alias.parse().unwrap(), column_type),
    )
}

pub fn sums_expr(
    tab: TableRef,
    names: &[&str],
    aliases: &[&str],
    column_types: &[ColumnType],
    accessor: &impl SchemaAccessor,
) -> Vec<(ColumnExpr, ColumnField)> {
    names
        .iter()
        .zip(aliases.iter().zip(column_types.iter()))
        .map(|(name, (alias, column_type))| sum_expr(tab, name, alias, *column_type, accessor))
        .collect()
}

pub fn group_by<C: Commitment>(
    group_by_exprs: Vec<ColumnExpr>,
    sum_expr: Vec<(ColumnExpr, ColumnField)>,
    count_alias: &str,
    table: TableExpr,
    where_clause: BoolExprPlan<C>,
) -> GroupByExpr<C> {
    GroupByExpr::new(
        group_by_exprs,
        sum_expr,
        count_alias.parse().unwrap(),
        table,
        where_clause,
    )
}
