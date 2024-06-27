use super::{
    AliasedProvableExprPlan, ColumnExpr, DenseFilterExpr, FilterExpr, FilterResultExpr,
    GroupByExpr, ProofPlan, ProvableExprPlan, TableExpr,
};
use crate::base::{
    commitment::Commitment,
    database::{ColumnField, ColumnRef, ColumnType, LiteralValue, SchemaAccessor, TableRef},
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
    ProofPlan::Filter(FilterExpr::new(results, table, where_clause))
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

pub fn dense_filter<C: Commitment>(
    results: Vec<AliasedProvableExprPlan<C>>,
    table: TableExpr,
    where_clause: ProvableExprPlan<C>,
) -> ProofPlan<C> {
    ProofPlan::DenseFilter(DenseFilterExpr::new(results, table, where_clause))
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
