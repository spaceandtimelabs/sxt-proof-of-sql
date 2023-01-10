use crate::intermediate_ast::*;
use crate::sql;
use crate::symbols::Name;

#[test]
fn we_can_parse_a_query_with_one_equals_filter_expression() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("SELECT A FROM SXT_TAB WHERE A = 3")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("a"),
                right: 3,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_two_result_columns() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("Select a,  b froM sxt_tab where C = 123")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![
                Box::new(ResultColumn::Expr {
                    expr: Name::from("a"),
                    output_name: None,
                }),
                Box::new(ResultColumn::Expr {
                    expr: Name::from("b"),
                    output_name: None,
                }),
            ],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("c"),
                right: 123,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_using_select_star() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("SELECT * FROM sxt_Tab WHERE A = 3")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::All)],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("a"),
                right: 3,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_using_multiple_select_star_expressions() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("SELECT a, *, b, c, * FROM sxt_Tab WHERE A = 3")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![
                Box::new(ResultColumn::Expr {
                    expr: Name::from("a"),
                    output_name: None,
                }),
                Box::new(ResultColumn::All),
                Box::new(ResultColumn::Expr {
                    expr: Name::from("b"),
                    output_name: None,
                }),
                Box::new(ResultColumn::Expr {
                    expr: Name::from("c"),
                    output_name: None,
                }),
                Box::new(ResultColumn::All),
            ],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("a"),
                right: 3,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_equals_filter_having_a_positive_literal() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = +4")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("b"),
                right: 4,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_equals_filter_having_a_negative_literal() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = -4")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("b"),
                right: -4,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_not_equals_filter_expression() {
    for not_equals_sign in ["!=", "<>"] {
        let parsed_ast = sql::SelectStatementParser::new()
            .parse(&("select a from sxt_tab where b".to_owned() + not_equals_sign + " -4"))
            .unwrap();

        let expected_ast = SelectStatement {
            expr: Box::new(SetExpression::Query {
                columns: vec![Box::new(ResultColumn::Expr {
                    expr: Name::from("a"),
                    output_name: None,
                })],
                from: vec![Box::new(TableExpression::Named {
                    table: Name::from("sxt_tab"),
                    schema: None,
                })],
                where_expr: Box::new(Expression::NotEqual {
                    left: Name::from("b"),
                    right: -4,
                }),
            }),
        };

        assert_eq!(parsed_ast, expected_ast);
    }
}

#[test]
fn we_can_parse_a_query_with_one_logical_not_filter_expression() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where not (b = 3)")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Not {
                expr: Box::new(Expression::Equal {
                    left: Name::from("b"),
                    right: 3,
                }),
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_logical_and_filter_expression() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) and (c = -2)")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::And {
                left: Box::new(Expression::Equal {
                    left: Name::from("b"),
                    right: 3,
                }),
                right: Box::new(Expression::Equal {
                    left: Name::from("c"),
                    right: -2,
                }),
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_logical_or_filter_expression() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) OR (c = -2)")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Or {
                left: Box::new(Expression::Equal {
                    left: Name::from("b"),
                    right: 3,
                }),
                right: Box::new(Expression::Equal {
                    left: Name::from("c"),
                    right: -2,
                }),
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_two_logical_and_not_filter_expressions() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) and (not (c = -2))")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::And {
                left: Box::new(Expression::Equal {
                    left: Name::from("b"),
                    right: 3,
                }),
                right: Box::new(Expression::Not {
                    expr: Box::new(Expression::Equal {
                        left: Name::from("c"),
                        right: -2,
                    }),
                }),
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_three_logical_not_and_or_filter_expressions() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where not ((b = 3) and  ((f = 45) or (c = -2)))")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Not {
                expr: Box::new(Expression::And {
                    left: Box::new(Expression::Equal {
                        left: Name::from("b"),
                        right: 3,
                    }),
                    right: Box::new(Expression::Or {
                        left: Box::new(Expression::Equal {
                            left: Name::from("f"),
                            right: 45,
                        }),
                        right: Box::new(Expression::Equal {
                            left: Name::from("c"),
                            right: -2,
                        }),
                    }),
                }),
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_the_minimum_i64_value_as_the_equal_filter_literal() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = -9223372036854775808")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("b"),
                right: -9223372036854775808,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_the_maximum_i64_value_as_the_equal_filter_literal() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where b = 9223372036854775807")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("b"),
                right: 9223372036854775807,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_and_rename_a_result_column_using_the_as_keyword() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a as a_rename from sxt_tab where b = 4")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: Some(Name::from("a_rename")),
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("b"),
                right: 4,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_and_rename_a_result_column_without_using_the_as_keyword() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a a_rename from sxt_tab where b = 4")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: Some(Name::from("a_rename")),
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: None,
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("b"),
                right: 4,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_not_with_more_precedence_priority_than_logical_and() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 3 and not b = 4")
        .unwrap();

    let expected_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (a = 3) and (not b = 4)")
        .unwrap();

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_cannot_parse_logical_not_with_more_precedence_priority_than_equal_operator() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where (not b) = 4")
        .is_err());
}

#[test]
fn we_can_parse_logical_and_with_more_precedence_priority_than_logical_or() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where a = -1 or c = -3 and a = 3")
        .unwrap();

    let expected_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where a = -1 or (c = -3 and a = 3)")
        .unwrap();

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_not_with_right_associativity() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where not not a = -1")
        .unwrap();

    let expected_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where not (not (a = -1))")
        .unwrap();

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_and_with_left_associativity() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where a = -1 and c = -3 and b = 3")
        .unwrap();

    let expected_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where ((a = -1) and (c = -3)) and (b = 3)")
        .unwrap();

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_or_with_left_associativity() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where a = -1 or c = -3 or b = 3")
        .unwrap();

    let expected_ast = sql::SelectStatementParser::new()
        .parse("select a from sxt_tab where ((a = -1) or (c = -3)) or (b = 3)")
        .unwrap();

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_identifiers_and_literals_with_as_much_parenthesis_as_necessary() {
    sql::SelectStatementParser::new()
        .parse("select (((a))) as F from ( (sxt_tab  )) where (((a = -1)) or c = -3) and (((((a = (((3)      ) ))))))")
        .unwrap();
}

#[test]
fn we_can_parse_a_query_with_one_schema_followed_by_a_table_name() {
    let parsed_ast = sql::SelectStatementParser::new()
        .parse("select a from eth.sxt_tab where b = 4")
        .unwrap();

    let expected_ast = SelectStatement {
        expr: Box::new(SetExpression::Query {
            columns: vec![Box::new(ResultColumn::Expr {
                expr: Name::from("a"),
                output_name: None,
            })],
            from: vec![Box::new(TableExpression::Named {
                table: Name::from("sxt_tab"),
                schema: Some(Name::from("eth")),
            })],
            where_expr: Box::new(Expression::Equal {
                left: Name::from("b"),
                right: 4,
            }),
        }),
    };

    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_query_with_two_schemas_followed_by_a_table_name() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from schema.name.tab")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_a_filter_value_smaller_than_min_i64_as_it_will_overflow() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b = -9223372036854775809")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_a_filter_value_bigger_than_max_i64_as_it_will_overflow() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from schema.tab where b = 9223372036854775808")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_select_tablename_followed_by_star() {
    assert!(sql::SelectStatementParser::new()
        .parse("select tab.* from tab")
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
fn we_cannot_parse_a_query_with_schemas_followed_by_column_and_table_names() {
    assert!(sql::SelectStatementParser::new()
        .parse("select tab.a from tab")
        .is_err());
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
fn we_cannot_parse_a_query_without_a_filter() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab")
        .is_err());
    assert!(sql::SelectStatementParser::new()
        .parse("select * from tab")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_a_subquery() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from (select a from tab where b = 4)")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_ending_with_a_semicolon() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b = 3;")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_having_the_limit_keyword() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b = 4 limit 3")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_gt() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b > 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_ge() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b >= 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_lt() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b < 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_le() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from tab where b <= 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_summing_the_result_columns() {
    assert!(sql::SelectStatementParser::new()
        .parse("select sum(a) from tab")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_group_by_keyword() {
    assert!(sql::SelectStatementParser::new()
        .parse("select b, sum(a) from tab group by b")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_inner_join_keyword() {
    assert!(sql::SelectStatementParser::new()
        .parse("select tab1.a from tab1 join tab2 on tab1.c = tab2.c where tab2.b > 4")
        .is_err());
}

// Case when
#[test]
fn we_cannot_parse_a_query_with_case_when_keyword() {
    assert!(sql::SelectStatementParser::new()
        .parse("select case when a == 2 then 3 else 5 from tab where b <= 4")
        .is_err());
}

//////////////////////
// Invalid SQLs
//////////////////////
#[test]
fn we_cannot_parse_a_query_missing_where_expressions() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from b where")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_where_keyword() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from b c = 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_from_table_name() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a from where c = 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_from_keyword() {
    assert!(sql::SelectStatementParser::new()
        .parse("select a b where c = 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_select_keyword() {
    assert!(sql::SelectStatementParser::new()
        .parse("a from b where c = 4")
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_select_result_columns() {
    assert!(sql::SelectStatementParser::new()
        .parse("select from b where c = 4")
        .is_err());
}
