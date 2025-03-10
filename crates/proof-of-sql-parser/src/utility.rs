use crate::{
    intermediate_ast::{
        AggregationOperator, AliasedResultExpr, BinaryOperator, Expression, Literal, OrderBy,
        OrderByDirection, SelectResultExpr, SetExpression, Slice, TableExpression, UnaryOperator,
    },
    Identifier, SelectStatement,
};
use alloc::{boxed::Box, vec, vec::Vec};

///
/// # Panics
///
/// This function will panic if `name` (if provided) cannot be parsed.
/// Construct an identifier from a str
#[must_use]
pub fn ident(name: &str) -> Identifier {
    name.parse().unwrap()
}

/// Construct a new boxed `Expression` A == B
#[must_use]
pub fn equal(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Equal,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A >= B
#[must_use]
pub fn ge(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    not(Box::new(Expression::Binary {
        op: BinaryOperator::LessThan,
        left,
        right,
    }))
}

/// Construct a new boxed `Expression` A > B
#[must_use]
pub fn gt(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::GreaterThan,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A <= B
#[must_use]
pub fn le(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    not(Box::new(Expression::Binary {
        op: BinaryOperator::GreaterThan,
        left,
        right,
    }))
}

/// Construct a new boxed `Expression` A < B
#[must_use]
pub fn lt(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::LessThan,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` NOT P
#[must_use]
pub fn not(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Unary {
        op: UnaryOperator::Not,
        expr,
    })
}

/// Construct a new boxed `Expression` P AND Q
#[must_use]
pub fn and(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::And,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` P OR Q
#[must_use]
pub fn or(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Or,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A + B
#[must_use]
pub fn add(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Add,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A - B
#[must_use]
pub fn sub(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Subtract,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A * B
#[must_use]
pub fn mul(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Multiply,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A / B
#[must_use]
pub fn div(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Division,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` IS NULL
#[must_use]
pub fn is_null(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::IsNull(expr))
}

/// Construct a new boxed `Expression` IS NOT NULL
#[must_use]
pub fn is_not_null(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::IsNotNull(expr))
}

/// Construct a new boxed `Expression` IS TRUE
#[must_use]
pub fn is_true(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::IsTrue(expr))
}

/// Get table from schema and name.
///
/// If the schema is `None`, the table is assumed to be in the default schema.
/// # Panics
///
/// This function will panic if either the `name` or the `schema` (if provided) cannot be parsed as valid [Identifier]s.
#[must_use]
pub fn tab(schema: Option<&str>, name: &str) -> Box<TableExpression> {
    Box::new(TableExpression::Named {
        table: name.parse().unwrap(),
        schema: schema.map(|schema| schema.parse().unwrap()),
    })
}

/// Get column from name
///
/// # Panics
///
/// This function will panic if the `name` cannot be parsed into a valid column expression as valid [Identifier]s.
#[must_use]
pub fn col(name: &str) -> Box<Expression> {
    Box::new(Expression::Column(name.parse().unwrap()))
}

/// Get literal from value
pub fn lit<L: Into<Literal>>(literal: L) -> Box<Expression> {
    Box::new(Expression::Literal(literal.into()))
}

/// Compute the sum of an expression
#[must_use]
pub fn sum(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Sum,
        expr,
    })
}

/// Compute the minimum of an expression
#[must_use]
pub fn min(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Min,
        expr,
    })
}

/// Compute the maximum of an expression
#[must_use]
pub fn max(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Max,
        expr,
    })
}

/// Count the amount of non-null entries of expression
#[must_use]
pub fn count(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Count,
        expr,
    })
}

/// Count the rows
#[must_use]
pub fn count_all() -> Box<Expression> {
    count(Box::new(Expression::Wildcard))
}

/// An expression with an alias i.e. EXPR AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed as valid [Identifier]s.
#[must_use]
pub fn aliased_expr(expr: Box<Expression>, alias: &str) -> AliasedResultExpr {
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
pub fn col_res(col_val: Box<Expression>, alias: &str) -> SelectResultExpr {
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
pub fn min_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
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
pub fn max_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
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
pub fn sum_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
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
pub fn count_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
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
        expr: Box::new(Expression::Aggregation {
            op: AggregationOperator::Count,
            expr: Box::new(Expression::Wildcard),
        }),
        alias: alias.parse().unwrap(),
    })
}

/// Generate a `SetExpression` of the kind SELECT COL1, COL2, ... FROM TAB WHERE EXPR GROUP BY ...
#[must_use]
pub fn query(
    result_exprs: Vec<SelectResultExpr>,
    tab: Box<TableExpression>,
    where_expr: Box<Expression>,
    group_by: Vec<Identifier>,
) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
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
    group_by: Vec<Identifier>,
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
pub fn order(id: &str, direction: OrderByDirection) -> Vec<OrderBy> {
    vec![OrderBy {
        expr: id.parse().unwrap(),
        direction,
    }]
}

/// Order by multiple columns i.e. ORDER BY ID0 [ASC|DESC], ID1 [ASC|DESC], ...
///
/// # Panics
///
/// This function will panic if any of the `ids` cannot be parsed
/// into an identifier.
#[must_use]
pub fn orders(ids: &[&str], directions: &[OrderByDirection]) -> Vec<OrderBy> {
    ids.iter()
        .zip(directions.iter())
        .map(|(id, dir)| OrderBy {
            expr: id.parse().unwrap(),
            direction: *dir,
        })
        .collect::<Vec<_>>()
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
pub fn group_by(ids: &[&str]) -> Vec<Identifier> {
    ids.iter().map(|id| id.parse().unwrap()).collect()
}
