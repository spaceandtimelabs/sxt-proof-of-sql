use crate::{intermediate_ast::*, Identifier, SelectStatement};
use alloc::{boxed::Box, vec, vec::Vec};

///
/// # Panics
///
/// This function will panic if`name`(if provided) cannot be parsed.
/// Construct an identifier from a str
pub fn ident(name: &str) -> Identifier {
    name.parse().unwrap()
}

/// Construct a new boxed `Expression` A == B
pub fn equal(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Equal,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A >= B
pub fn ge(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::GreaterThanOrEqual,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A <= B
pub fn le(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::LessThanOrEqual,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` NOT P
pub fn not(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Unary {
        op: UnaryOperator::Not,
        expr,
    })
}

/// Construct a new boxed `Expression` P AND Q
pub fn and(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::And,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` P OR Q
pub fn or(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Or,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A + B
pub fn add(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Add,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A - B
pub fn sub(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Subtract,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A * B
pub fn mul(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Multiply,
        left,
        right,
    })
}

/// Construct a new boxed `Expression` A / B
pub fn div(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Division,
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
pub fn col(name: &str) -> Box<Expression> {
    Box::new(Expression::Column(name.parse().unwrap()))
}

/// Get literal from value
pub fn lit<L: Into<Literal>>(literal: L) -> Box<Expression> {
    Box::new(Expression::Literal(literal.into()))
}

/// Compute the sum of an expression
pub fn sum(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Sum,
        expr,
    })
}

/// Compute the minimum of an expression
pub fn min(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Min,
        expr,
    })
}

/// Compute the maximum of an expression
pub fn max(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Max,
        expr,
    })
}

/// Count the amount of non-null entries of expression
pub fn count(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Aggregation {
        op: AggregationOperator::Count,
        expr,
    })
}

/// Count the rows
pub fn count_all() -> Box<Expression> {
    count(Box::new(Expression::Wildcard))
}

/// An expression with an alias i.e. EXPR AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed as valid [Identifier]s.
pub fn aliased_expr(expr: Box<Expression>, alias: &str) -> AliasedResultExpr {
    AliasedResultExpr {
        expr,
        alias: alias.parse().unwrap(),
    }
}

/// Select all columns from a table i.e. SELECT *
pub fn col_res_all() -> SelectResultExpr {
    SelectResultExpr::ALL
}

/// Select one column from a table and give it an alias i.e. SELECT COL AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed as valid [Identifier]s.
pub fn col_res(col_val: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: col_val,
        alias: alias.parse().unwrap(),
    })
}

/// Select multiple columns from a table i.e. SELECT COL1, COL2, ...
pub fn cols_res(names: &[&str]) -> Vec<SelectResultExpr> {
    names.iter().map(|name| col_res(col(name), name)).collect()
}

/// Compute the minimum of an expression and give it an alias i.e. SELECT MIN(EXPR) AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed.
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
pub fn count_all_res(alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: Expression::Aggregation {
            op: AggregationOperator::Count,
            expr: Box::new(Expression::Wildcard),
        }
        .into(),
        alias: alias.parse().unwrap(),
    })
}

/// Generate a `SetExpression` of the kind SELECT COL1, COL2, ... FROM TAB WHERE EXPR GROUP BY ...
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
pub fn group_by(ids: &[&str]) -> Vec<Identifier> {
    ids.iter().map(|id| id.parse().unwrap()).collect()
}
