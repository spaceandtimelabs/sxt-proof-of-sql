
use alloc::{boxed::Box, vec, vec::Vec};
use clap::Id;
use serde::Serialize;
use sqlparser::ast::{BinaryOperator, Expr, Ident, OrderBy, OrderByExpr, Query, SetExpr, Table, UnaryOperator, Value};

///
/// # Panics
///
/// This function will panic if`name`(if provided) cannot be parsed.
/// Construct an identifier from a str
pub fn assert_ident_valid(name: &str) {
    // TODO: implement
    assert!(name.len() > 0);
}
///
/// # Panics
///
/// This function will panic if`name`(if provided) cannot be parsed.
/// Construct an identifier from a str
#[must_use]
pub fn ident(name: &str) -> Ident {
    assert_ident_valid(name);
    Ident::from(name)
}

/// Construct a new boxed `Expr` A == B
#[must_use]
pub fn equal(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::Eq,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` A >= B
#[must_use]
pub fn ge(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::GtEq,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` A <= B
#[must_use]
pub fn le(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::LtEq,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` NOT P
#[must_use]
pub fn not(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::UnaryOp {
        op: UnaryOperator::Not,
        expr,
    })
}

/// Construct a new boxed `Expr` P AND Q
#[must_use]
pub fn and(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::And,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` P OR Q
#[must_use]
pub fn or(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::Or,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` A + B
#[must_use]
pub fn add(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::Plus,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` A - B
#[must_use]
pub fn sub(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::Minus,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` A * B
#[must_use]
pub fn mul(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::Multiply,
        left,
        right,
    })
}

/// Construct a new boxed `Expr` A / B
#[must_use]
pub fn div(left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::BinaryOp {
        op: BinaryOperator::Divide,
        left,
        right,
    })
}

/// Get table from schema and name.
///
/// If the schema is `None`, the table is assumed to be in the default schema.
/// # Panics
///
/// This function will panic if either the `name` or the `schema` (if provided) cannot be parsed as valid [Identifier]s.
#[must_use]
pub fn tab(schema: Option<&str>, name: &str) -> Box<Table> {
    Box::new(Table {
        table_name: name.into(),
        schema_name : schema.map(|s| s.to_owned()),
    })
}

/// Get column from name
///
/// # Panics
///
/// This function will panic if the `name` cannot be parsed into a valid column expression as valid [Identifier]s.
#[must_use]
pub fn col(name: &str) -> Box<Expr> {

    Box::new(Expr::Identifier(ident(name)))
}
pub trait IntoLiteral {
    fn into_literal(self) -> Value;
}
impl IntoLiteral for String {
    fn into_literal(self) -> Value {
        Value::SingleQuotedString(self)
    }
}
macro_rules! int_literal {
    ($ty:ty, $long:literal) => {

impl IntoLiteral for $ty {
    fn into_literal(self) -> Value {
        Value::Number(self.to_string(), $long)
    }
}
    };
}
int_literal!(u32, false);
int_literal!(i32, false);
int_literal!(u64, true);
int_literal!(i64, true);


/// Get literal from value
pub fn lit<L: IntoLiteral>(literal: L) -> Box<Expr> {
    Box::new(Expr::Value(literal.into_literal()))
}

/// Compute the sum of an expression
#[must_use]
pub fn sum(expr: Box<Expr>) -> Box<Query> {
    Box::new(Query {
        body: Box::new(SetExpr::Select(Box())),
        op: AggregationOperator::Sum,
        fetch   : None,
        for_clause: Some()
    })
}

/// Compute the minimum of an expression
#[must_use]
pub fn min(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Aggregation {
        op: AggregationOperator::Min,
        expr,
    })
}

/// Compute the maximum of an expression
#[must_use]
pub fn max(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Aggregation {
        op: AggregationOperator::Max,
        expr,
    })
}

/// Count the amount of non-null entries of expression
#[must_use]
pub fn count(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Aggregation {
        op: AggregationOperator::Count,
        expr,
    })
}

/// Count the rows
#[must_use]
pub fn count_all() -> Box<Expr> {
    count(Box::new(Expr::Wildcard))
}

/// An expression with an alias i.e. EXPR AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed as valid [Identifier]s.
#[must_use]
pub fn aliased_expr(expr: Box<Expr>, alias: &str) -> AliasedResultExpr {
    AliasedResultExpr {
        expr,
        alias: alias.parse().unwrap(),
    }
}

/// Select all columns from a table i.e. SELECT *
#[must_use]
pub fn col_res_all() -> SelectResultExpr {
    SelectResultExpr::ALL
}

/// Select one column from a table and give it an alias i.e. SELECT COL AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed as valid [Identifier]s.
#[must_use]
pub fn col_res(col_val: Box<Expr>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: col_val,
        alias: alias.parse().unwrap(),
    })
}

/// Select multiple columns from a table i.e. SELECT COL1, COL2, ...
#[must_use]
pub fn cols_res(names: &[&str]) -> Vec<SelectResultExpr> {
    names.iter().map(|name| col_res(col(name), name)).collect()
}

/// Compute the minimum of an expression and give it an alias i.e. SELECT MIN(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
#[must_use]
pub fn min_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: min(expr),
        alias: alias.parse().unwrap(),
    })
}

/// Compute the maximum of an expression and give it an alias i.e. SELECT MAX(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
#[must_use]
pub fn max_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: max(expr),
        alias: alias.parse().unwrap(),
    })
}

/// Compute the sum of an expression and give it an alias i.e. SELECT SUM(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
#[must_use]
pub fn sum_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: sum(expr),
        alias: alias.parse().unwrap(),
    })
}

/// Count the amount of non-null entries of expression and give it an alias i.e. SELECT COUNT(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
#[must_use]
pub fn count_res(expr: Box<Expr>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: count(expr),
        alias: alias.parse().unwrap(),
    })
}

/// Count rows and give the result an alias i.e. SELECT COUNT(*) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
#[must_use]
pub fn count_all_res(alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: Expr::Aggregation {
            op: AggregationOperator::Count,
            expr: Box::new(Expr::Wildcard),
        }
            .into(),
        alias: alias.parse().unwrap(),
    })
}

/// Generate a `SetExpression` of the kind SELECT COL1, COL2, ... FROM TAB WHERE EXPR GROUP BY ...
#[must_use]
pub fn query(
    result_exprs: Vec<SelectResultExpr>,
    tab: Box<TableExpression>,
    where_expr: Box<Expr>,
    group_by: Vec<Ident>,
) -> Box<SetExpr> {
    Box::new( ::Query {
        result_exprs,
        from: vec![tab],
        where_expr: Some(where_expr),
        group_by,
    })
}

/// Generate a `SetExpression` of the kind SELECT COL1, COL2, ... FROM TAB GROUP BY ...
///
/// Note that there is no WHERE clause.
#[must_use]
pub fn query_all(
    result_exprs: Vec<SelectResultExpr>,
    tab: Box<TableExpression>,
    group_by: Vec<Ident>,
) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
        result_exprs,
        from: vec![tab],
        where_expr: None,
        group_by,
    })
}

/// Generate a query of the kind SELECT ... ORDER BY ... [LIMIT ... OFFSET ...]
///
/// Note that `expr` is a boxed `SetExpression`
#[must_use]
pub fn select(
    expr: Box<SetExpression>,
    order_by: Vec<OrderBy>,
    slice: Option<Slice>,
) -> SelectStatement {
    SelectStatement {
        expr,
        order_by,
        slice,
    }
}

/// Order by one column i.e. ORDER BY ID [ASC|DESC]
///
/// # Panics
///
/// This function will panic if the `id` cannot be parsed into an identifier.
#[must_use]
pub fn order(id: &str, direction: OrderByDirection) -> OrderBy {
    OrderBy {
        exprs:  vec![OrderByExpr {
            expr: Expr::Identifier(Ident::from(id)),
            asc: Some(direction == OrderByDirection::Ascending),
            nulls_first: None,
            with_fill: None
        }],
        interpolate: None,
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderByDirection {
    Descending,
    Ascending
}
/// Order by multiple columns i.e. ORDER BY ID0 [ASC|DESC], ID1 [ASC|DESC], ...
///
/// # Panics
///
/// This function will panic if any of the `ids` cannot be parsed
/// into an identifier.
#[must_use]
pub fn orders(ids: &[&str], directions: &[OrderByDirection]) -> OrderBy {
    let exprs: Vec<OrderByExpr> = ids.iter()
        .zip(directions.iter())
        .map(|(id, dir)|OrderByExpr {
            expr: Expr::Identifier(Ident::from(id)),
            asc: Some(dir == OrderByDirection::Ascending),
            nulls_first: None,
            with_fill: None
        })
        .collect::<Vec<_>>();

    OrderBy {
        exprs,
        interpolate: None,
    }
}

/// Slice a query result using `LIMIT` and `OFFSET` clauses i.e. LIMIT N OFFSET M
#[must_use]
pub fn slice(number_rows: u64, offset_value: i64) -> Option<Slice> {
    Some(Slice {
        number_rows,
        offset_value,
    })
}

/// Group by clause with multiple columns i.e. GROUP BY ID0, ID1, ...
///
/// # Panics
///
/// This function will panic if any of the `ids` cannot be parsed
/// into an identifier.
#[must_use]
pub fn group_by(ids: &[&str]) -> Vec<Ident> {
    ids.iter().map(Ident::from).collect()
}
