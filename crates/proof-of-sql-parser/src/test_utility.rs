use crate::{intermediate_ast::*, Identifier, SelectStatement};

/// A == B
pub fn equal(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Equal,
        left,
        right,
    })
}

/// A >= B
pub fn ge(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::GreaterThanOrEqual,
        left,
        right,
    })
}

/// A <= B
pub fn le(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::LessThanOrEqual,
        left,
        right,
    })
}

/// NOT P
pub fn not(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Unary {
        op: UnaryOperator::Not,
        expr,
    })
}

/// P AND Q
pub fn and(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::And,
        left,
        right,
    })
}

/// P OR Q
pub fn or(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Or,
        left,
        right,
    })
}

/// Get table from schema and name
pub fn tab(schema: Option<&str>, name: &str) -> Box<TableExpression> {
    Box::new(TableExpression::Named {
        table: name.parse().unwrap(),
        schema: schema.map(|schema| schema.parse().unwrap()),
    })
}

/// Get column from name
pub fn col(name: &str) -> Box<Expression> {
    Box::new(Expression::Column(name.parse().unwrap()))
}

/// Get literal from value
pub fn lit<L: Into<Literal>>(literal: L) -> Box<Expression> {
    Box::new(Expression::Literal(literal.into()))
}

/// SELECT *
pub fn col_res_all() -> SelectResultExpr {
    SelectResultExpr::ALL
}

/// SELECT COL AS ALIAS
pub fn col_res(col_val: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: col_val,
        alias: alias.parse().unwrap(),
    })
}

/// SELECT COL1, COL2, ...
pub fn cols_res(names: &[&str]) -> Vec<SelectResultExpr> {
    names.iter().map(|name| col_res(col(name), name)).collect()
}

/// SELECT MIN(EXPR) AS ALIAS
pub fn min_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: Box::new(Expression::Aggregation {
            op: AggregationOperator::Min,
            expr,
        }),
        alias: alias.parse().unwrap(),
    })
}

/// SELECT MAX(EXPR) AS ALIAS
pub fn max_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: Expression::Aggregation {
            op: AggregationOperator::Max,
            expr,
        }
        .into(),
        alias: alias.parse().unwrap(),
    })
}

/// SELECT SUM(EXPR) AS ALIAS
pub fn sum_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: Expression::Aggregation {
            op: AggregationOperator::Sum,
            expr,
        }
        .into(),
        alias: alias.parse().unwrap(),
    })
}

/// SELECT COUNT(EXPR) AS ALIAS
pub fn count_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: Expression::Aggregation {
            op: AggregationOperator::Count,
            expr,
        }
        .into(),
        alias: alias.parse().unwrap(),
    })
}

/// SELECT COUNT(*) AS ALIAS
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

/// SELECT COL1, COL2, ... FROM TAB WHERE EXPR GROUP BY ...
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

/// SELECT COL1, COL2, ... FROM TAB GROUP BY ...
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

/// SELECT ... ORDER BY ... [LIMIT ... OFFSET ...]
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

/// ORDER BY ID [ASC|DESC]
pub fn order(id: &str, direction: OrderByDirection) -> Vec<OrderBy> {
    vec![OrderBy {
        expr: id.parse().unwrap(),
        direction,
    }]
}

/// ORDER BY ID0 [ASC|DESC], ID1 [ASC|DESC], ...
pub fn orders(ids: &[&str], directions: &[OrderByDirection]) -> Vec<OrderBy> {
    ids.iter()
        .zip(directions.iter())
        .map(|(id, dir)| OrderBy {
            expr: id.parse().unwrap(),
            direction: *dir,
        })
        .collect::<Vec<_>>()
}

/// LIMIT N OFFSET M
pub fn slice(number_rows: u64, offset_value: i64) -> Option<Slice> {
    Some(Slice {
        number_rows,
        offset_value,
    })
}

/// GROUP BY ID0, ID1, ...
pub fn group_by(ids: &[&str]) -> Vec<Identifier> {
    ids.iter().map(|id| id.parse().unwrap()).collect()
}
