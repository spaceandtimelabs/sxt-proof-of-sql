use crate::intermediate_ast::*;
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

pub fn col_res_all() -> Box<ResultColumn> {
    Box::new(ResultColumn::All)
}

pub fn col_res(name: &str, out_name: &str) -> Box<ResultColumn> {
    Box::new(ResultColumn::Expr {
        expr: name.parse().unwrap(),
        output_name: out_name.parse().unwrap(),
    })
}

pub fn cols_res(names: &[&str]) -> Vec<Box<ResultColumn>> {
    names
        .iter()
        .map(|name| {
            Box::new(ResultColumn::Expr {
                expr: name.parse().unwrap(),
                output_name: name.parse().unwrap(),
            })
        })
        .collect()
}

pub fn query(
    columns: Vec<Box<ResultColumn>>,
    tab: Box<TableExpression>,
    where_expr: Box<Expression>,
) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
        columns,
        from: vec![tab],
        where_expr: Some(where_expr),
    })
}

pub fn query_all(columns: Vec<Box<ResultColumn>>, tab: Box<TableExpression>) -> Box<SetExpression> {
    Box::new(SetExpression::Query {
        columns,
        from: vec![tab],
        where_expr: None,
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
