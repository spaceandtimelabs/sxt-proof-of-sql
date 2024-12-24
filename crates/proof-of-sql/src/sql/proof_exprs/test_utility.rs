use super::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr};
use crate::base::{
    database::{ColumnRef, LiteralValue, SchemaAccessor, TableRef},
    math::{decimal::Precision, i256::I256},
    scalar::Scalar,
};
use proof_of_sql_parser::intermediate_ast::AggregationOperator;
use sqlparser::ast::Ident;

pub fn col_ref<S: Into<TableRef>>(tab: S, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let table_ref = tab.into();
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab.clone(), name.clone()).unwrap();
    ColumnRef::new(tab, name, type_col)
}

/// # Panics
/// Panics if:
/// - `accessor.lookup_column()` returns `None`, indicating the column is not found.
pub fn column<S: Into<TableRef>>(
    tab: S,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> DynProofExpr {
    let table_ref = tab.into();
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab.clone(), name.clone()).unwrap();
    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(tab.clone(), name, type_col)))
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_equals()` returns an error.
pub fn equal(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_equals(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn lte(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_inequality(left, right, true).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn gte(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_inequality(left, right, false).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_not()` returns an error.
pub fn not(expr: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_not(expr).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_and()` returns an error.
pub fn and(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_and(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_or()` returns an error.
pub fn or(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_or(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_add()` returns an error.
pub fn add(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_add(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_subtract()` returns an error.
pub fn subtract(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_subtract(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_multiply()` returns an error.
pub fn multiply(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_multiply(left, right).unwrap()
}

pub fn const_bool(val: bool) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Boolean(val))
}

pub fn const_smallint(val: i16) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::SmallInt(val))
}

pub fn const_int(val: i32) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int(val))
}

pub fn const_bigint(val: i64) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::BigInt(val))
}

pub fn const_int128(val: i128) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int128(val))
}

pub fn const_varchar(val: &str) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::VarChar(val.to_string()))
}

/// Create a constant scalar value. Used if we don't want to specify column types.
pub fn const_scalar<S: Scalar, T: Into<S>>(val: T) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Scalar(val.into().into()))
}

/// # Panics
/// Panics if:
/// - `Precision::new(precision)` fails, meaning the provided precision is invalid.
pub fn const_decimal75<T: Into<I256>>(precision: u8, scale: i8, val: T) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Decimal75(
        Precision::new(precision).unwrap(),
        scale,
        val.into(),
    ))
}

pub fn tab<S: Into<TableRef>>(tab: S) -> TableExpr {
    TableExpr {
        table_ref: tab.into(),
    }
}

/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
pub fn aliased_plan(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr,
        alias: alias.into(),
    }
}

/// # Panics
/// Panics if:
/// - `old_name.parse()` or `new_name.parse()` fails to parse the provided column names.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
pub fn aliased_col_expr_plan<S: Into<TableRef>>(
    tab: S,
    old_name: &str,
    new_name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    let tab = tab.into();
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab.clone(), old_name, accessor))),
        alias: new_name.into(),
    }
}

/// # Panics
/// Panics if:
/// - `name.parse()` fails to parse the provided column name.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
pub fn col_expr_plan<S: Into<TableRef>>(
    tab: S,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    let tab = tab.into();
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab.clone(), name, accessor))),
        alias: name.into(),
    }
}

pub fn aliased_cols_expr_plan<S: Into<TableRef>>(
    tab: S,
    names: &[(&str, &str)],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr> {
    let tab = tab.into();
    names
        .iter()
        .map(|(old_name, new_name)| {
            aliased_col_expr_plan(tab.clone(), old_name, new_name, accessor)
        })
        .collect()
}

pub fn cols_expr_plan<S: Into<TableRef>>(
    tab: S,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr> {
    let tab = tab.into();
    names
        .iter()
        .map(|name| col_expr_plan(tab.clone(), name, accessor))
        .collect()
}

pub fn col_expr<S: Into<TableRef>>(
    tab: S,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> ColumnExpr {
    let tab = tab.into();
    ColumnExpr::new(col_ref(tab.clone(), name, accessor))
}

pub fn cols_expr<S: Into<TableRef>>(
    tab: S,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<ColumnExpr> {
    let tab = tab.into();
    names
        .iter()
        .map(|name| col_expr(tab.clone(), name, accessor))
        .collect()
}

/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
pub fn sum_expr(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::new_aggregate(AggregationOperator::Sum, expr),
        alias: alias.into(),
    }
}
