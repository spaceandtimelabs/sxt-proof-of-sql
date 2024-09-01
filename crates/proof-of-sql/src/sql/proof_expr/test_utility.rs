use super::{
    AliasedProvableExprPlan, ColumnExpr, ProvableExprPlan, TableExpr,
};
use crate::base::{
    commitment::Commitment,
    database::{ColumnRef, LiteralValue, SchemaAccessor, TableRef},
    math::decimal::Precision,
};

pub fn col_ref(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ColumnRef::new(tab, name, type_col)
}

pub fn column<C: Commitment>(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> ProvableExprPlan<C> {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ProvableExprPlan::Column(ColumnExpr::new(ColumnRef::new(tab, name, type_col)))
}

pub fn equal<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_equals(left, right).unwrap()
}

pub fn lte<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_inequality(left, right, true).unwrap()
}

pub fn gte<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_inequality(left, right, false).unwrap()
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

pub fn add<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_add(left, right).unwrap()
}

pub fn subtract<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_subtract(left, right).unwrap()
}

pub fn multiply<C: Commitment>(
    left: ProvableExprPlan<C>,
    right: ProvableExprPlan<C>,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::try_new_multiply(left, right).unwrap()
}

pub fn const_bool<C: Commitment>(val: bool) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::Boolean(val))
}

pub fn const_smallint<C: Commitment>(val: i16) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::SmallInt(val))
}

pub fn const_int<C: Commitment>(val: i32) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::Int(val))
}

pub fn const_bigint<C: Commitment>(val: i64) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::BigInt(val))
}

pub fn const_int128<C: Commitment>(val: i128) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::Int128(val))
}

pub fn const_varchar<C: Commitment>(val: &str) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::VarChar((
        val.to_string(),
        C::Scalar::from(val),
    )))
}

/// Create a constant scalar value. Used if we don't want to specify column types.
#[allow(dead_code)]
pub fn const_scalar<C: Commitment, T: Into<C::Scalar>>(val: T) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::Scalar(val.into()))
}

pub fn const_decimal75<C: Commitment, T: Into<C::Scalar>>(
    precision: u8,
    scale: i8,
    val: T,
) -> ProvableExprPlan<C> {
    ProvableExprPlan::new_literal(LiteralValue::Decimal75(
        Precision::new(precision).unwrap(),
        scale,
        val.into(),
    ))
}

pub fn tab(tab: TableRef) -> TableExpr {
    TableExpr { table_ref: tab }
}

pub fn aliased_plan<C: Commitment>(
    expr: ProvableExprPlan<C>,
    alias: &str,
) -> AliasedProvableExprPlan<C> {
    AliasedProvableExprPlan {
        expr,
        alias: alias.parse().unwrap(),
    }
}

pub fn aliased_col_expr_plan<C: Commitment>(
    tab: TableRef,
    old_name: &str,
    new_name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedProvableExprPlan<C> {
    AliasedProvableExprPlan {
        expr: ProvableExprPlan::Column(ColumnExpr::<C>::new(col_ref(tab, old_name, accessor))),
        alias: new_name.parse().unwrap(),
    }
}

pub fn col_expr_plan<C: Commitment>(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedProvableExprPlan<C> {
    AliasedProvableExprPlan {
        expr: ProvableExprPlan::Column(ColumnExpr::<C>::new(col_ref(tab, name, accessor))),
        alias: name.parse().unwrap(),
    }
}

pub fn aliased_cols_expr_plan<C: Commitment>(
    tab: TableRef,
    names: &[(&str, &str)],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedProvableExprPlan<C>> {
    names
        .iter()
        .map(|(old_name, new_name)| aliased_col_expr_plan(tab, old_name, new_name, accessor))
        .collect()
}

pub fn cols_expr_plan<C: Commitment>(
    tab: TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedProvableExprPlan<C>> {
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
