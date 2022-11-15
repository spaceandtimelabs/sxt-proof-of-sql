use crate::intermediate_ast::*;
use crate::sql;
use crate::symbols::Name;

#[test]
fn we_can_parse_one_column() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 3")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let where_expr = Box::new(Expression::Equal {
        left: Name::from("a"),
        right: 3,
    });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn we_can_parse_a_namespaced_table() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from eth.sxt_tab where a = -3")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: Some(Name::from("eth")),
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let where_expr = Box::new(Expression::Equal {
        left: Name::from("a"),
        right: -3,
    });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn we_can_parse_two_columns() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a,  b from sxt_tab where c = 123")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![
        Box::new(ResultColumn::Expr {
            expr: Name::from("a"),
        }),
        Box::new(ResultColumn::Expr {
            expr: Name::from("b"),
        }),
    ];

    let where_expr = Box::new(Expression::Equal {
        left: Name::from("c"),
        right: 123,
    });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

// Filter
#[test]
fn filter_one_positive_cond() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = +4")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let where_expr = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: 4,
    });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

// Filter
#[test]
fn filter_one_negative_cond() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = -4")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let where_expr = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: -4,
    });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn filter_two_cond_and() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) and (c = -2)")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let left = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: 3,
    });

    let right = Box::new(Expression::Equal {
        left: Name::from("c"),
        right: -2,
    });

    let where_expr = Box::new(Expression::And { left, right });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn filter_two_cond_or() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) OR (c = -2)")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let left = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: 3,
    });

    let right = Box::new(Expression::Equal {
        left: Name::from("c"),
        right: -2,
    });

    let where_expr = Box::new(Expression::Or { left, right });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn filter_two_cond_and_not() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) and (not (c = -2))")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let left = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: 3,
    });

    let expr = Box::new(Expression::Equal {
        left: Name::from("c"),
        right: -2,
    });

    let right = Box::new(Expression::Not { expr });

    let where_expr = Box::new(Expression::And { left, right });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn filter_three_cond_not_and_or() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where not ((b = 3) and  ((f = 45) or (c = -2)))")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let equal_left = Box::new(Expression::Equal {
        left: Name::from("f"),
        right: 45,
    });

    let equal_right = Box::new(Expression::Equal {
        left: Name::from("c"),
        right: -2,
    });

    let or_right = Box::new(Expression::Or {
        left: equal_left,
        right: equal_right,
    });

    let equal_left = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: 3,
    });

    let and_expr = Box::new(Expression::And {
        left: equal_left,
        right: or_right,
    });

    let where_expr = Box::new(Expression::Not { expr: and_expr });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn filter_i64_min_value() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = -9223372036854775808")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let where_expr = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: -9223372036854775808,
    });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn filter_i64_max_value() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = 9223372036854775807")
        .unwrap();

    let from = vec![Box::new(TableExpression::Named {
        table: Name::from("sxt_tab"),
        namespace: None,
    })];

    let columns = vec![Box::new(ResultColumn::Expr {
        expr: Name::from("a"),
    })];

    let where_expr = Box::new(Expression::Equal {
        left: Name::from("b"),
        right: 9223372036854775807,
    });

    let expr = Box::new(SetExpression::Query {
        columns,
        from,
        where_expr,
    });

    let expected_ast = SelectStatement { expr };

    assert_eq!(expected_ast, parsed_ast);
}

#[test]
fn more_than_one_namespace_in_the_table_wil_fail() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from namespace.name.tab")
        .is_err());
}

#[test]
fn filter_value_smaller_than_min_i64_will_overflow() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b = -9223372036854775809")
        .is_err());
}

#[test]
fn filter_value_bigger_than_max_i64_will_overflow() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from namespace.tab where b = 9223372036854775808")
        .is_err());
}

#[test]
fn select_star_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select * from namespace.tab")
        .is_err());
}

#[test]
fn select_tablename_star_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select tab.* from namespace.tab")
        .is_err());
}

// Unparsables
// Unparsables consist of the following categories
// 1. Queries we don't support yet but plan to support in the future.
// 2. Valid queries that are out of scope.
// 3. Invalid queries.

//////////////////////
// Not supported yet
// The following are valid queries that will be gradually enabled as our PoSQL engine is built.
// We ignore the exact LALRPOP error type since it changes as LARPOP is upgraded
// and is outside our control.
//////////////////////

#[test]
fn select_constant_not_supported() {
    assert!(sql::SelectStatementParser::new().parse("select 2").is_err());
}

#[test]
fn query_having_namespaced_columns_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select tab.a from eth.tab")
        .is_err());
    assert!(sql::SelectStatementParser::new()
        .parse("select eth.tab.a from eth.tab")
        .is_err());
    assert!(sql::SelectStatementParser::new()
        .parse("select a from eth.tab where tab.b = 3")
        .is_err());
}

#[test]
fn aliasing_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a as b from tab")
        .is_err());
}

#[test]
fn subquery_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from (select a from namespace.tab where b > 4)")
        .is_err());
}

#[test]
fn semicolon_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab;")
        .is_err());
}

#[test]
fn limit_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from name.tab where b = 4 limit 3")
        .is_err());
}

#[test]
fn filter_gt_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b > 4")
        .is_err());
}

#[test]
fn filter_le_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b <= 4")
        .is_err());
}

#[test]
fn sum_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select sum(a) from some_namespace.tab")
        .is_err());
}

#[test]
fn groupby_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select b, sum(a) from tab group by b")
        .is_err());
}

#[test]
fn inner_join_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select tab1.a from tab1 join tab2 on tab1.c = tab2.c where tab2.b > 4")
        .is_err());
}

// Case when
#[test]
fn casewhen_not_supported() {
    assert!(sql::SelectStatementParser::new()
        .parse("select case when a == 2 then 3 else 5 from tab where b <= 4")
        .is_err());
}

//////////////////////
// Invalid SQLs
//////////////////////
#[test]
fn query_missing_where_expressions_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from b where")
        .is_err());
}

#[test]
fn query_missing_where_keyword_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from b c = 4")
        .is_err());
}

#[test]
fn query_missing_from_table_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from where c = 4")
        .is_err());
}

#[test]
fn query_missing_from_keyword_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a b where c = 4")
        .is_err());
}

#[test]
fn query_missing_select_keyword_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("a from b where c = 4")
        .is_err());
}

#[test]
fn query_missing_select_result_column_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select from b where c = 4")
        .is_err());
}

#[test]
fn query_missing_select_result_semicolumn_will_error_out() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a c from b where c = 4")
        .is_err());
}
