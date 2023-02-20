use crate::test_utility::*;
use crate::SelectStatement;

#[test]
fn we_can_parse_a_query_with_one_equals_filter_expression() {
    let ast = "SELECT A FROM SXT_TAB WHERE A = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(cols_res(&["a"]), tab(None, "sxt_tab"), equal("a", 3)));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_two_result_columns() {
    let ast = "Select a,  b froM sxt_tab where C = 123"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a", "b"]),
        tab(None, "sxt_tab"),
        equal("c", 123),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_using_select_star() {
    let ast = "SELECT * FROM sxt_Tab WHERE A = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        vec![col_res_all()],
        tab(None, "sxt_tab"),
        equal("a", 3),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_using_multiple_select_star_expressions() {
    let ast = "SELECT a, *, b, c, * FROM sxt_Tab WHERE A = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        vec![
            col_res("a", "a"),
            col_res_all(),
            col_res("b", "b"),
            col_res("c", "c"),
            col_res_all(),
        ],
        tab(None, "sxt_tab"),
        equal("a", 3),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_equals_filter_having_a_positive_literal() {
    let ast = "select a from sxt_tab where b = +4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(cols_res(&["a"]), tab(None, "sxt_tab"), equal("b", 4)));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_equals_filter_having_a_negative_literal() {
    let ast = "select a from sxt_tab where b = -4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        equal("b", -4),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_not_equals_filter_expression() {
    for not_equals_sign in ["!=", "<>"] {
        let ast = ("select a from sxt_tab where b".to_owned() + not_equals_sign + " -4")
            .parse::<SelectStatement>()
            .unwrap();
        let expected_ast = select(query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            not(equal("b", -4)),
        ));
        assert_eq!(ast, expected_ast);
    }
}

#[test]
fn we_can_parse_a_query_with_one_logical_not_filter_expression() {
    let ast = "select a from sxt_tab where not (b = 3)"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        not(equal("b", 3)),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_logical_and_filter_expression() {
    let ast = "select a from sxt_tab where (b = 3) and (c = -2)"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        and(equal("b", 3), equal("c", -2)),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_logical_or_filter_expression() {
    let ast = "select a from sxt_tab where (b = 3) or (c = -2)"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        or(equal("b", 3), equal("c", -2)),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_two_logical_and_not_filter_expressions() {
    let ast = "select a from sxt_tab where (b = 3) and (not (c = -2))"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        and(equal("b", 3), not(equal("c", -2))),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_three_logical_not_and_or_filter_expressions() {
    let ast = "select a from sxt_tab where not ((b = 3) and  ((f = 45) or (c = -2)))"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        not(and(equal("b", 3), or(equal("f", 45), equal("c", -2)))),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_the_minimum_i64_value_as_the_equal_filter_literal() {
    let ast = "select a from sxt_tab where b = -9223372036854775808"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        equal("b", -9223372036854775808_i64),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_the_maximum_i64_value_as_the_equal_filter_literal() {
    let ast = "select a from sxt_tab where b = 9223372036854775807"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(None, "sxt_tab"),
        equal("b", 9223372036854775807_i64),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_and_rename_a_result_column_using_the_as_keyword() {
    let ast = "select a as a_rename from sxt_tab where b = 4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        vec![col_res("a", "a_rename")],
        tab(None, "sxt_tab"),
        equal("b", 4),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_and_rename_a_result_column_without_using_the_as_keyword() {
    let parsed_ast = "select a a_rename from sxt_tab where b = 4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        vec![col_res("a", "a_rename")],
        tab(None, "sxt_tab"),
        equal("b", 4),
    ));
    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_not_with_more_precedence_priority_than_logical_and() {
    let parsed_ast = "select a from sxt_tab where a = 3 and not b = 4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = "select a from sxt_tab where (a = 3) and (not b = 4)"
        .parse::<SelectStatement>()
        .unwrap();
    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_cannot_parse_logical_not_with_more_precedence_priority_than_equal_operator() {
    assert!("select a from sxt_tab where (not b) = 4"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_can_parse_logical_and_with_more_precedence_priority_than_logical_or() {
    let ast = "select a from sxt_tab where a = -1 or c = -3 and a = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = "select a from sxt_tab where a = -1 or (c = -3 and a = 3)"
        .parse::<SelectStatement>()
        .unwrap();
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_logical_not_with_right_associativity() {
    let ast = "select a from sxt_tab where not not a = -1"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = "select a from sxt_tab where not (not (a = -1))"
        .parse::<SelectStatement>()
        .unwrap();
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_logical_and_with_left_associativity() {
    let ast = "select a from sxt_tab where a = -1 and c = -3 and b = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = "select a from sxt_tab where ((a = -1) and (c = -3)) and (b = 3)"
        .parse::<SelectStatement>()
        .unwrap();
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_logical_or_with_left_associativity() {
    let ast = "select a from sxt_tab where a = -1 or c = -3 or b = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = "select a from sxt_tab where ((a = -1) or (c = -3)) or (b = 3)"
        .parse::<SelectStatement>()
        .unwrap();
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_identifiers_and_literals_with_as_much_parenthesis_as_necessary() {
    let ast = "select (((a))) as F from ( (sxt_tab  )) where (((a = -1)) or c = -3) and (((((a = (((3)      ) ))))))"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        vec![col_res("a", "F")],
        tab(None, "sxt_tab"),
        and(or(equal("a", -1), equal("c", -3)), equal("a", 3)),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_schema_followed_by_a_table_name() {
    let ast = "select a from eth.sxt_tab where b = 4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(query(
        cols_res(&["a"]),
        tab(Some("eth"), "sxt_tab"),
        equal("b", 4),
    ));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_without_a_filter() {
    let ast = "select a from tab".parse::<SelectStatement>().unwrap();
    let expected_ast = select(query_all(cols_res(&["a"]), tab(None, "tab")));
    assert_eq!(ast, expected_ast);

    let ast = "select * from eth.tab".parse::<SelectStatement>().unwrap();
    let expected_ast = select(query_all(vec![col_res_all()], tab(Some("eth"), "tab")));
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_query_with_two_schemas_followed_by_a_table_name() {
    assert!("select a from schema.Identifier.tab"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_a_filter_value_smaller_than_min_i64_as_it_will_overflow() {
    assert!("select a from tab where b = -9223372036854775809"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_a_filter_value_bigger_than_max_i64_as_it_will_overflow() {
    assert!("select a from schema.tab where b = 9223372036854775808"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_select_tablename_followed_by_star() {
    assert!("select tab.* from tab".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_a_query_with_schemas_followed_by_column_and_table_names() {
    assert!("select tab.a from tab".parse::<SelectStatement>().is_err());
    assert!("select tab.a from eth.tab"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select eth.tab.a from eth.tab"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select a from eth.tab where tab.b = 3"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_a_subquery() {
    assert!("select a from (select a from tab where b = 4)"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_ending_with_a_semicolon() {
    assert!("select a from tab where b = 3;"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_having_the_limit_keyword() {
    assert!("select a from tab where b = 4 limit 3"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_gt() {
    assert!("select a from tab where b > 4"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_ge() {
    assert!("select a from tab where b >= 4"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_lt() {
    assert!("select a from tab where b < 4"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_filter_le() {
    assert!("select a from tab where b <= 4"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_summing_the_result_columns() {
    assert!("select sum(a) from tab".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_a_query_with_group_by_keyword() {
    assert!("select b, sum(a) from tab group by b"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_inner_join_keyword() {
    assert!(
        "select tab1.a from tab1 join tab2 on tab1.c = tab2.c where tab2.b > 4"
            .parse::<SelectStatement>()
            .is_err()
    );
}

// Case when
#[test]
fn we_cannot_parse_a_query_with_case_when_keyword() {
    assert!(
        "select case when a == 2 then 3 else 5 from tab where b <= 4"
            .parse::<SelectStatement>()
            .is_err()
    );
}

//////////////////////
// Invalid SQLs
//////////////////////
#[test]
fn we_cannot_parse_a_query_missing_where_expressions() {
    assert!("select a from b where".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_where_keyword() {
    assert!("select a from b c = 4".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_from_table_name() {
    assert!("select a from where c = 4"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_from_keyword() {
    assert!("select a b where c = 4".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_select_keyword() {
    assert!("a from b where c = 4".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_a_query_missing_select_result_columns() {
    assert!("select from b where c = 4"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_queries_with_long_identifiers() {
    // Long table names should be rejected
    assert!("SELECT A FROM AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA WHERE A = 3".parse::<SelectStatement>().is_err());

    // Long column names should be rejected
    assert!("SELECT A FROM A WHERE AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA = 3".parse::<SelectStatement>().is_err());

    // Long column aliases should be rejected
    assert!("SELECT A AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA FROM A WHERE A = 3".parse::<SelectStatement>().is_err());

    // Long columns names shouldn't be interpreted as a short column and a short alias
    // Whitespace matters: "AAAAAA" != ("AAA AAA" or "AAA AS AAA")
    assert!("SELECT AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA FROM A WHERE A = 3".parse::<SelectStatement>().is_err());
}
