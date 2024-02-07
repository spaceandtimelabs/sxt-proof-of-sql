use super::{
    BoolExprPlan, ColumnExpr, DenseFilterExpr, FilterExpr, FilterResultExpr, GroupByExpr,
    ProofPlan, TableExpr,
};
use crate::base::{
    commitment::Commitment,
    database::{ColumnField, ColumnRef, ColumnType, SchemaAccessor, TableRef},
    scalar::ArkScalar,
};
use curve25519_dalek::RistrettoPoint;

pub fn col(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ColumnRef::new(tab, name, type_col)
}

pub fn equal<T: Into<ArkScalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &impl SchemaAccessor,
) -> BoolExprPlan<RistrettoPoint> {
    BoolExprPlan::new_equals(col(tab, name, accessor), val.into())
}

pub fn lte<T: Into<ArkScalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &impl SchemaAccessor,
) -> BoolExprPlan<RistrettoPoint> {
    BoolExprPlan::new_inequality(col(tab, name, accessor), val.into(), true)
}

pub fn gte<T: Into<ArkScalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &impl SchemaAccessor,
) -> BoolExprPlan<RistrettoPoint> {
    BoolExprPlan::new_inequality(col(tab, name, accessor), val.into(), false)
}

pub fn not(expr: BoolExprPlan<RistrettoPoint>) -> BoolExprPlan<RistrettoPoint> {
    BoolExprPlan::new_not(expr)
}

pub fn and(
    left: BoolExprPlan<RistrettoPoint>,
    right: BoolExprPlan<RistrettoPoint>,
) -> BoolExprPlan<RistrettoPoint> {
    BoolExprPlan::new_and(left, right)
}

pub fn or(
    left: BoolExprPlan<RistrettoPoint>,
    right: BoolExprPlan<RistrettoPoint>,
) -> BoolExprPlan<RistrettoPoint> {
    BoolExprPlan::new_or(left, right)
}

pub fn const_v(val: bool) -> BoolExprPlan<RistrettoPoint> {
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

pub fn dense_filter(
    results: Vec<ColumnExpr>,
    table: TableExpr,
    where_clause: BoolExprPlan<RistrettoPoint>,
) -> DenseFilterExpr {
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

pub fn group_by(
    group_by_exprs: Vec<ColumnExpr>,
    sum_expr: Vec<(ColumnExpr, ColumnField)>,
    count_alias: &str,
    table: TableExpr,
    where_clause: BoolExprPlan<RistrettoPoint>,
) -> GroupByExpr {
    GroupByExpr::new(
        group_by_exprs,
        sum_expr,
        count_alias.parse().unwrap(),
        table,
        where_clause,
    )
}
