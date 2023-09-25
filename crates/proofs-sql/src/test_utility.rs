use crate::intermediate_ast::*;
use crate::Identifier;
use crate::SelectStatement;
use std::ops;

pub fn equal<T: Into<Literal>>(name: &str, literal: T) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Equal,
        left: Box::new(Expression::Column(name.parse().unwrap())),
        right: Box::new(Expression::Literal(literal.into())),
    })
}

pub fn not(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Unary {
        op: UnaryOperator::Not,
        expr,
    })
}

pub fn and(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::And,
        left,
        right,
    })
}

pub fn or(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary {
        op: BinaryOperator::Or,
        left,
        right,
    })
}

pub fn tab(schema: Option<&str>, name: &str) -> Box<TableExpression> {
    Box::new(TableExpression::Named {
        table: name.parse().unwrap(),
        schema: schema.map(|schema| schema.parse().unwrap()),
    })
}

pub fn col(name: &str) -> Box<Expression> {
    Box::new(Expression::Column(name.parse().unwrap()))
}

pub fn lit<L: Into<Literal>>(literal: L) -> Box<Expression> {
    Box::new(Expression::Literal(literal.into()))
}

pub fn col_res_all() -> SelectResultExpr {
    SelectResultExpr::ALL
}

pub fn col_res(col_val: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: col_val,
        alias: alias.parse().unwrap(),
    })
}

pub fn cols_res(names: &[&str]) -> Vec<SelectResultExpr> {
    names.iter().map(|name| col_res(col(name), name)).collect()
}

pub fn min_res(expr: Box<Expression>, alias: &str) -> SelectResultExpr {
    SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
        expr: Box::new(Expression::Aggregation {
            op: AggregationOperator::Min,
            expr,
        }),
        alias: alias.parse().unwrap(),
    })
}

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

pub fn order(id: &str, direction: OrderByDirection) -> Vec<OrderBy> {
    vec![OrderBy {
        expr: id.parse().unwrap(),
        direction,
    }]
}

pub fn orders(ids: &[&str], directions: &[OrderByDirection]) -> Vec<OrderBy> {
    ids.iter()
        .zip(directions.iter())
        .map(|(id, dir)| OrderBy {
            expr: id.parse().unwrap(),
            direction: *dir,
        })
        .collect::<Vec<_>>()
}

pub fn slice(number_rows: u64, offset_value: i64) -> Option<Slice> {
    Some(Slice {
        number_rows,
        offset_value,
    })
}

pub fn group_by(ids: &[&str]) -> Vec<Identifier> {
    ids.iter().map(|id| id.parse().unwrap()).collect()
}

impl ops::Add<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn add(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Add,
            left: self,
            right: rhs,
        })
    }
}

impl ops::Mul<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn mul(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Multiply,
            left: self,
            right: rhs,
        })
    }
}

impl ops::Div<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn div(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Division,
            left: self,
            right: rhs,
        })
    }
}

impl ops::Sub<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn sub(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Subtract,
            left: self,
            right: rhs,
        })
    }
}
