use crate::intermediate_ast::*;
use crate::Identifier;
use crate::SelectStatement;

pub fn equal<T: Into<Literal>>(name: &str, literal: T) -> Box<Expression> {
    Box::new(Expression::Equal {
        left: name.parse().unwrap(),
        right: Box::new(literal.into()),
    })
}

pub fn not(expr: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Not { expr })
}

pub fn and(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::And { left, right })
}

pub fn or(left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Or { left, right })
}

pub fn tab(schema: Option<&str>, name: &str) -> Box<TableExpression> {
    Box::new(TableExpression::Named {
        table: name.parse().unwrap(),
        schema: schema.map(|schema| schema.parse().unwrap()),
    })
}

pub fn col_res_all() -> ResultColumnExpr {
    ResultColumnExpr::AllColumns
}

pub fn col_res(name: &str, out_name: &str) -> ResultColumnExpr {
    ResultColumnExpr::SimpleColumn(ResultColumn {
        name: name.parse().unwrap(),
        alias: out_name.parse().unwrap(),
    })
}

pub fn cols_res(names: &[&str]) -> Vec<ResultColumnExpr> {
    names
        .iter()
        .map(|name| {
            ResultColumnExpr::SimpleColumn(ResultColumn {
                name: name.parse().unwrap(),
                alias: name.parse().unwrap(),
            })
        })
        .collect()
}

pub fn min_res(name: &str, alias: &str) -> ResultColumnExpr {
    ResultColumnExpr::AggColumn(AggExpr::Min(ResultColumn {
        name: name.parse().unwrap(),
        alias: alias.parse().unwrap(),
    }))
}

pub fn max_res(name: &str, alias: &str) -> ResultColumnExpr {
    ResultColumnExpr::AggColumn(AggExpr::Max(ResultColumn {
        name: name.parse().unwrap(),
        alias: alias.parse().unwrap(),
    }))
}

pub fn count_res(name: &str, alias: &str) -> ResultColumnExpr {
    ResultColumnExpr::AggColumn(AggExpr::Count(ResultColumn {
        name: name.parse().unwrap(),
        alias: alias.parse().unwrap(),
    }))
}

pub fn count_all_res(alias: &str) -> ResultColumnExpr {
    ResultColumnExpr::AggColumn(AggExpr::CountAll(alias.parse::<Identifier>().unwrap()))
}

pub fn query(
    columns: Vec<ResultColumnExpr>,
    tab: Box<TableExpression>,
    where_expr: Box<Expression>,
    group_by: Vec<Identifier>,
) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
        columns,
        from: vec![tab],
        where_expr: Some(where_expr),
        group_by,
    })
}

pub fn query_all(
    columns: Vec<ResultColumnExpr>,
    tab: Box<TableExpression>,
    group_by: Vec<Identifier>,
) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
        columns,
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
            direction: dir.clone(),
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
