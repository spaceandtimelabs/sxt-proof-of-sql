use super::{
    AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr,
    TableExpr,
};
use crate::base::{
    database::{ColumnRef, SchemaAccessor, TableRef, TestAccessor},
    scalar::ArkScalar,
};

pub fn col(tab: TableRef, name: &str, accessor: &TestAccessor) -> ColumnRef {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ColumnRef::new(tab, name, type_col)
}

pub fn equal<T: Into<ArkScalar>>(
    tab: TableRef,
    name: &str,
    val: T,
    accessor: &TestAccessor,
) -> Box<dyn BoolExpr> {
    Box::new(EqualsExpr::new(col(tab, name, accessor), val.into()))
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

pub fn col_result(tab: TableRef, name: &str, accessor: &TestAccessor) -> FilterResultExpr {
    FilterResultExpr::new(col(tab, name, accessor))
}

pub fn cols_result(
    tab: TableRef,
    names: &[&str],
    accessor: &TestAccessor,
) -> Vec<FilterResultExpr> {
    names
        .iter()
        .map(|name| col_result(tab, name, accessor))
        .collect()
}

pub fn filter(
    results: Vec<FilterResultExpr>,
    table: TableExpr,
    where_clause: Box<dyn BoolExpr>,
) -> FilterExpr {
    FilterExpr::new(results, table, where_clause)
}
