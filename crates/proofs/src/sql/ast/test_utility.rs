use super::{
    AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr,
    TableExpr,
};
use crate::base::database::SchemaAccessor;
use crate::base::database::{ColumnRef, TableRef, TestAccessor};
use crate::base::scalar::ToScalar;

pub fn col(tab: TableRef, name: &str, accessor: &TestAccessor) -> ColumnRef {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ColumnRef::new(tab, name, type_col)
}

pub fn equal<T: ToScalar>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &TestAccessor,
) -> Box<dyn BoolExpr> {
    Box::new(EqualsExpr::new(col(tab, name, accessor), val.to_scalar()))
}

pub fn not(expr: Box<dyn BoolExpr>) -> Box<dyn BoolExpr> {
    Box::new(NotExpr::new(expr))
}

pub fn and(left: Box<dyn BoolExpr>, right: Box<dyn BoolExpr>) -> Box<dyn BoolExpr> {
    Box::new(AndExpr::new(left, right))
}

pub fn or(left: Box<dyn BoolExpr>, right: Box<dyn BoolExpr>) -> Box<dyn BoolExpr> {
    Box::new(OrExpr::new(left, right))
}

pub fn const_v(val: bool) -> Box<dyn BoolExpr> {
    Box::new(ConstBoolExpr::new(val))
}

pub fn tab(tab: TableRef) -> TableExpr {
    TableExpr { table_ref: tab }
}

pub fn col_result(
    tab: TableRef,
    name: &str,
    out_name: &str,
    accessor: &TestAccessor,
) -> FilterResultExpr {
    FilterResultExpr::new(col(tab, name, accessor), out_name.parse().unwrap())
}

pub fn cols_result(
    tab: TableRef,
    names: &[&str],
    accessor: &TestAccessor,
) -> Vec<FilterResultExpr> {
    names
        .iter()
        .map(|name| col_result(tab, name, name, accessor))
        .collect()
}

pub fn filter(
    results: Vec<FilterResultExpr>,
    table: TableExpr,
    where_clause: Box<dyn BoolExpr>,
) -> Box<FilterExpr> {
    Box::new(FilterExpr::new(results, table, where_clause))
}
