use super::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr};
use crate::base::{
    commitment::Commitment,
    database::{ColumnRef, LiteralValue, SchemaAccessor, TableRef},
    math::decimal::Precision,
};
use proof_of_sql_parser::intermediate_ast::AggregationOperator;

/// # Panics
/// Panics if:
/// - `name.parse()` fails, which means the provided string could not be parsed into the expected type (usually an `Identifier`).
pub fn col_ref(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    ColumnRef::new(tab, name, type_col)
}

/// # Panics
/// Panics if:
/// - `name.parse()` fails to parse the column name.
/// - `accessor.lookup_column()` returns `None`, indicating the column is not found.
pub fn column<C: Commitment>(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> DynProofExpr<C> {
    let name = name.parse().unwrap();
    let type_col = accessor.lookup_column(tab, name).unwrap();
    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(tab, name, type_col)))
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_equals()` returns an error.
pub fn equal<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_equals(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn lte<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_inequality(left, right, true).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn gte<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_inequality(left, right, false).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_not()` returns an error.
pub fn not<C: Commitment>(expr: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_not(expr).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_and()` returns an error.
pub fn and<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_and(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_or()` returns an error.
pub fn or<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_or(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_add()` returns an error.
pub fn add<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_add(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_subtract()` returns an error.
pub fn subtract<C: Commitment>(left: DynProofExpr<C>, right: DynProofExpr<C>) -> DynProofExpr<C> {
    DynProofExpr::try_new_subtract(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_multiply()` returns an error.
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
    DynProofExpr::new_literal(LiteralValue::VarChar(val.to_string()))
}

/// Create a constant scalar value. Used if we don't want to specify column types.
#[allow(dead_code)]
pub fn const_scalar<C: Commitment, T: Into<C::Scalar>>(val: T) -> DynProofExpr<C> {
    DynProofExpr::new_literal(LiteralValue::Scalar(val.into()))
}

/// # Panics
/// Panics if:
/// - `Precision::new(precision)` fails, meaning the provided precision is invalid.
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

/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
pub fn aliased_plan<C: Commitment>(expr: DynProofExpr<C>, alias: &str) -> AliasedDynProofExpr<C> {
    AliasedDynProofExpr {
        expr,
        alias: alias.parse().unwrap(),
    }
}

/// # Panics
/// Panics if:
/// - `old_name.parse()` or `new_name.parse()` fails to parse the provided column names.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
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

/// # Panics
/// Panics if:
/// - `name.parse()` fails to parse the provided column name.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
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

/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
pub fn sum_expr<C: Commitment>(expr: DynProofExpr<C>, alias: &str) -> AliasedDynProofExpr<C> {
    AliasedDynProofExpr {
        expr: DynProofExpr::new_aggregate(AggregationOperator::Sum, expr),
        alias: alias.parse().unwrap(),
    }
}
