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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Identifier;

    #[test]
    fn test_ident() {
        let identifier = ident("test_id");
        assert_eq!(identifier.as_str(), "test_id");
    }

    #[test]
    fn test_binary_operations() {
        let left = Box::new(Expression::Column(Identifier::new("a")));
        let right = Box::new(Expression::Column(Identifier::new("b")));

        // Test equal
        let eq = equal(left.clone(), right.clone());
        assert!(matches!(
            *eq,
            Expression::Binary {
                op: BinaryOperator::Equal,
                left: _,
                right: _
            }
        ));

        // Test greater than or equal
        let ge_expr = ge(left.clone(), right.clone());
        assert!(matches!(
            *ge_expr,
            Expression::Unary {
                op: UnaryOperator::Not,
                expr: _
            }
        ));

        // Test greater than
        let gt_expr = gt(left.clone(), right.clone());
        assert!(matches!(
            *gt_expr,
            Expression::Binary {
                op: BinaryOperator::GreaterThan,
                left: _,
                right: _
            }
        ));

        // Test less than or equal
        let le_expr = le(left.clone(), right.clone());
        assert!(matches!(
            *le_expr,
            Expression::Unary {
                op: UnaryOperator::Not,
                expr: _
            }
        ));

        // Test less than
        let lt_expr = lt(left.clone(), right.clone());
        assert!(matches!(
            *lt_expr,
            Expression::Binary {
                op: BinaryOperator::LessThan,
                left: _,
                right: _
            }
        ));

        // Test not
        let not_expr = not(left.clone());
        assert!(matches!(
            *not_expr,
            Expression::Unary {
                op: UnaryOperator::Not,
                expr: _
            }
        ));

        // Test and
        let and_expr = and(left.clone(), right.clone());
        assert!(matches!(
            *and_expr,
            Expression::Binary {
                op: BinaryOperator::And,
                left: _,
                right: _
            }
        ));

        // Test or
        let or_expr = or(left.clone(), right.clone());
        assert!(matches!(
            *or_expr,
            Expression::Binary {
                op: BinaryOperator::Or,
                left: _,
                right: _
            }
        ));

        // Test add
        let add_expr = add(left.clone(), right.clone());
        assert!(matches!(
            *add_expr,
            Expression::Binary {
                op: BinaryOperator::Add,
                left: _,
                right: _
            }
        ));

        // Test sub
        let sub_expr = sub(left.clone(), right.clone());
        assert!(matches!(
            *sub_expr,
            Expression::Binary {
                op: BinaryOperator::Subtract,
                left: _,
                right: _
            }
        ));

        // Test mul
        let mul_expr = mul(left.clone(), right.clone());
        assert!(matches!(
            *mul_expr,
            Expression::Binary {
                op: BinaryOperator::Multiply,
                left: _,
                right: _
            }
        ));

        // Test div
        let div_expr = div(left, right);
        assert!(matches!(
            *div_expr,
            Expression::Binary {
                op: BinaryOperator::Division,
                left: _,
                right: _
            }
        ));
    }

    #[test]
    fn test_table_operations() {
        // Test tab with schema
        let table_with_schema = tab(Some("schema"), "table");
        assert!(matches!(
            *table_with_schema,
            TableExpression::Named {
                schema: Some(_),
                table: _
            }
        ));

        // Test tab without schema
        let table_without_schema = tab(None, "table");
        assert!(matches!(
            *table_without_schema,
            TableExpression::Named {
                schema: None,
                table: _
            }
        ));
    }

    #[test]
    fn test_column_operations() {
        // Test col
        let column = col("test_col");
        assert!(matches!(*column, Expression::Column(_)));

        // Test lit
        let literal = lit(42);
        assert!(matches!(*literal, Expression::Literal(_)));
    }

    #[test]
    fn test_aggregation_operations() {
        let expr = Box::new(Expression::Column(Identifier::new("test_col")));

        // Test sum
        let sum_expr = sum(expr.clone());
        assert!(matches!(
            *sum_expr,
            Expression::Aggregation {
                op: AggregationOperator::Sum,
                expr: _
            }
        ));

        // Test min
        let min_expr = min(expr.clone());
        assert!(matches!(
            *min_expr,
            Expression::Aggregation {
                op: AggregationOperator::Min,
                expr: _
            }
        ));

        // Test max
        let max_expr = max(expr.clone());
        assert!(matches!(
            *max_expr,
            Expression::Aggregation {
                op: AggregationOperator::Max,
                expr: _
            }
        ));

        // Test count
        let count_expr = count(expr);
        assert!(matches!(
            *count_expr,
            Expression::Aggregation {
                op: AggregationOperator::Count,
                expr: _
            }
        ));

        // Test count_all
        let count_all_expr = count_all();
        assert!(matches!(
            *count_all_expr,
            Expression::Aggregation {
                op: AggregationOperator::Count,
                expr: _
            }
        ));
    }

    #[test]
    fn test_result_expressions() {
        // Test aliased_expr
        let expr = Box::new(Expression::Column(Identifier::new("test_col")));
        let aliased = aliased_expr(expr.clone(), "alias");
        assert_eq!(aliased.alias.as_str(), "alias");

        // Test col_res_all
        let all = col_res_all();
        assert!(matches!(all, SelectResultExpr::ALL));

        // Test col_res
        let col_result = col_res(expr.clone(), "alias");
        assert!(matches!(col_result, SelectResultExpr::AliasedResultExpr(_)));

        // Test cols_res
        let cols = cols_res(&["col1", "col2"]);
        assert_eq!(cols.len(), 2);
        assert!(matches!(cols[0], SelectResultExpr::AliasedResultExpr(_)));
    }

    #[test]
    fn test_aggregation_results() {
        let expr = Box::new(Expression::Column(Identifier::new("test_col")));

        // Test min_res
        let min_result = min_res(expr.clone(), "alias");
        assert!(matches!(min_result, SelectResultExpr::AliasedResultExpr(_)));

        // Test max_res
        let max_result = max_res(expr.clone(), "alias");
        assert!(matches!(max_result, SelectResultExpr::AliasedResultExpr(_)));

        // Test sum_res
        let sum_result = sum_res(expr.clone(), "alias");
        assert!(matches!(sum_result, SelectResultExpr::AliasedResultExpr(_)));

        // Test count_res
        let count_result = count_res(expr, "alias");
        assert!(matches!(
            count_result,
            SelectResultExpr::AliasedResultExpr(_)
        ));

        // Test count_all_res
        let count_all_result = count_all_res("alias");
        assert!(matches!(
            count_all_result,
            SelectResultExpr::AliasedResultExpr(_)
        ));
    }

    #[test]
    fn test_query_builders() {
        let expr = Box::new(Expression::Column(Identifier::new("test_col")));
        let table = tab(None, "table");
        let where_expr = equal(expr.clone(), lit(42));
        let group_by = vec![Identifier::new("group_col")];

        // Test query
        let query_expr = query(
            vec![col_res(expr.clone(), "alias")],
            table.clone(),
            where_expr,
            group_by.clone(),
        );
        assert!(matches!(*query_expr, SetExpression::Query { .. }));

        // Test query_all
        let query_all_expr = query_all(vec![col_res(expr, "alias")], table, group_by);
        assert!(matches!(*query_all_expr, SetExpression::Query { .. }));
    }

    #[test]
    fn test_order_and_slice() {
        // Test order
        let order_expr = order("col", OrderByDirection::Asc);
        assert_eq!(order_expr.len(), 1);
        assert_eq!(order_expr[0].direction, OrderByDirection::Asc);

        // Test orders
        let orders_expr = orders(
            &["col1", "col2"],
            &[OrderByDirection::Asc, OrderByDirection::Desc],
        );
        assert_eq!(orders_expr.len(), 2);
        assert_eq!(orders_expr[0].direction, OrderByDirection::Asc);
        assert_eq!(orders_expr[1].direction, OrderByDirection::Desc);

        // Test slice
        let slice_expr = slice(10, 5);
        assert!(slice_expr.is_some());
        let slice_data = slice_expr.unwrap();
        assert_eq!(slice_data.number_rows, 10);
        assert_eq!(slice_data.offset_value, 5);
    }

    #[test]
    fn test_group_by() {
        let group_by_expr = group_by(&["col1", "col2"]);
        assert_eq!(group_by_expr.len(), 2);
        assert_eq!(group_by_expr[0].as_str(), "col1");
        assert_eq!(group_by_expr[1].as_str(), "col2");
    }
}
