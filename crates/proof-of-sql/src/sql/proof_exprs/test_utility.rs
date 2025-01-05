use super::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr};
use crate::base::{
    database::{ColumnRef, SchemaAccessor, TableRef},
    math::i256::I256,
    scalar::Scalar,
};
use proof_of_sql_parser::intermediate_ast::AggregationOperator;
use sqlparser::ast::{DataType, ExactNumberInfo, Expr, Ident, ObjectName, Value};

pub fn col_ref(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab, name.clone()).unwrap();
    ColumnRef::new(tab, name, type_col)
}

/// # Panics
/// Panics if:
/// - `accessor.lookup_column()` returns `None`, indicating the column is not found.
pub fn column(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> DynProofExpr {
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab, name.clone()).unwrap();
    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(tab, name, type_col)))
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
    DynProofExpr::new_literal(Expr::Value(Value::Boolean(val)))
}

pub fn const_smallint(val: i16) -> DynProofExpr {
    DynProofExpr::new_literal(Expr::Value(Value::Number(val.to_string(), false)))
}

pub fn const_int(val: i32) -> DynProofExpr {
    DynProofExpr::new_literal(Expr::Value(Value::Number(val.to_string(), false)))
}

pub fn const_bigint(val: i64) -> DynProofExpr {
    DynProofExpr::new_literal(Expr::Value(Value::Number(val.to_string(), false)))
}

pub fn const_int128(val: i128) -> DynProofExpr {
    DynProofExpr::new_literal(Expr::Value(Value::Number(val.to_string(), false)))
}

pub fn const_varchar(val: &str) -> DynProofExpr {
    DynProofExpr::new_literal(Expr::Value(Value::SingleQuotedString(val.to_string())))
}

/// Create a constant scalar value. Used if we don't want to specify column types.
pub fn const_scalar<S: Scalar, T: Into<S>>(val: T) -> DynProofExpr {
    let scalar_str = format!("scalar:{}", val.into());

    DynProofExpr::new_literal(Expr::TypedString {
        data_type: DataType::Custom(ObjectName(vec![Ident::new("scalar")]), vec![]),
        value: scalar_str,
    })
}

/// # Panics
/// Panics if:
/// - `Precision::new(precision)` fails, meaning the provided precision is invalid.
pub fn const_decimal75<T: Into<I256>>(precision: u8, scale: i8, val: T) -> DynProofExpr {
    let decimal_value = val.into();
    let decimal_str = format!("{decimal_value}e{scale}");
    DynProofExpr::new_literal(Expr::TypedString {
        data_type: DataType::Decimal(ExactNumberInfo::PrecisionAndScale(
            u64::from(precision),
            i64::from(scale).try_into().unwrap(),
        )),
        value: decimal_str,
    })
}

pub fn tab(tab: TableRef) -> TableExpr {
    TableExpr { table_ref: tab }
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
pub fn aliased_col_expr_plan(
    tab: TableRef,
    old_name: &str,
    new_name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab, old_name, accessor))),
        alias: new_name.into(),
    }
}

/// # Panics
/// Panics if:
/// - `name.parse()` fails to parse the provided column name.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
pub fn col_expr_plan(
    tab: TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab, name, accessor))),
        alias: name.into(),
    }
}

pub fn aliased_cols_expr_plan(
    tab: TableRef,
    names: &[(&str, &str)],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr> {
    names
        .iter()
        .map(|(old_name, new_name)| aliased_col_expr_plan(tab, old_name, new_name, accessor))
        .collect()
}

pub fn cols_expr_plan(
    tab: TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr> {
    names
        .iter()
        .map(|name| col_expr_plan(tab, name, accessor))
        .collect()
}

pub fn col_expr(tab: TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnExpr {
    ColumnExpr::new(col_ref(tab, name, accessor))
}

pub fn cols_expr(tab: TableRef, names: &[&str], accessor: &impl SchemaAccessor) -> Vec<ColumnExpr> {
    names
        .iter()
        .map(|name| col_expr(tab, name, accessor))
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
