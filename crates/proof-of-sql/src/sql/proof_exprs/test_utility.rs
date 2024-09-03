use super::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr};
use crate::base::{
    commitment::Commitment,
    database::{ColumnRef, LiteralValue, SchemaAccessor, TableRef},
    math::decimal::Precision,
};
use proof_of_sql_parser::intermediate_ast::AggregationOperator;

pub fn col_ref(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ColumnRef::new(tab, name, type_col)
}

pub fn column<C: Commitment>(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> DynProofExpr<C> {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(tab, name, type_col)))
}

pub fn equal<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_equals(left, right).unwrap()
}

pub fn lte<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_inequality(left, right, true).unwrap()
}

pub fn gte<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_inequality(left, right, false).unwrap()
}

pub fn not<C: Commitment>(expr: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_not(expr).unwrap()
}

pub fn and<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_and(left, right).unwrap()
}

pub fn or<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_or(left, right).unwrap()
}

pub fn add<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_add(left, right).unwrap()
}

pub fn subtract<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_subtract(left, right).unwrap()
}

pub fn multiply<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_multiply(left, right).unwrap()
}

pub fn const_bool<C: Commitment>(val: bool) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::Boolean(val))
}

pub fn const_smallint<C: Commitment>(val: i16) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::SmallInt(val))
}

pub fn const_int<C: Commitment>(val: i32) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::Int(val))
}

pub fn const_bigint<C: Commitment>(val: i64) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::BigInt(val))
}

pub fn const_int128<C: Commitment>(val: i128) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::Int128(val))
}

pub fn const_varchar<C: Commitment>(val: &str) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::VarChar((
        val.to_string(),
        C::Scalar::from(val),
    )))
}

/// Create a constant scalar value. Used if we don't want to specify column types.
#[allow(dead_code)]
pub fn const_scalar<C: Commitment, T: Into<C::Scalar>>(val: T) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::Scalar(val.into()))
}

pub fn const_decimal75<C: Commitment, T: Into<C::Scalar>>(
    precision: u8,
    scale: i8,
    val: T,
) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::Decimal75(
        Precision::new(precision).unwrap(),
        scale,
        val.into(),
    ))
}

pub fn tab(tab: TableRef) -> TableExpr {
    TableExpr { table_ref: tab }
}

pub fn aliased_plan<C: Commitment>(expr: DynProofExpr<C>, alias: &str) -> AliasedDynProofExpr<C> {
    AliasedDynProofExpr {
        expr,
        alias: alias.parse().unwrap(),
    }
}

pub fn aliased_col_expr_plan<C: Commitment>(
    tab: TableRef,
    old_name: &str,
    new_name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr<C> {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::<C>::new(col_ref(tab, old_name, accessor))),
        alias: new_name.parse().unwrap(),
    }
}

pub fn col_expr_plan<C: Commitment>(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr<C> {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::<C>::new(col_ref(tab, name, accessor))),
        alias: name.parse().unwrap(),
    }
}

pub fn aliased_cols_expr_plan<C: Commitment>(
    tab: TableRef,
    names: &[(&str, &str)],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr<C>> {
    names
        .iter()
        .map(|(old_name, new_name)| aliased_col_expr_plan(tab, old_name, new_name, accessor))
        .collect()
}

pub fn cols_expr_plan<C: Commitment>(
    tab: TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr<C>> {
    names
        .iter()
        .map(|name| col_expr_plan(tab, name, accessor))
        .collect()
}

pub fn col_expr<C: Commitment>(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> ColumnExpr<C> {
    ColumnExpr::<C>::new(col_ref(tab, name, accessor))
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

pub fn sum_expr<C: Commitment>(expr: DynProofExpr<C>, alias: &str) -> AliasedDynProofExpr<C> {
    AliasedDynProofExpr {
        expr: DynProofExpr::new_aggregate(AggregationOperator::Sum, expr),
        alias: alias.parse().unwrap(),
    }
}
