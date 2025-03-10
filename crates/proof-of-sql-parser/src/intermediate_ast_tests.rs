use crate::{
    intermediate_ast::{
        Literal,
        OrderByDirection::{Asc, Desc},
    },
    sql::*,
    utility::*,
    SelectStatement,
};
use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
};
use bigdecimal::BigDecimal;

// String parser tests
#[test]
fn we_can_parse_simple_strings() {
    assert_eq!(
        StringLiteralParser::new().parse("'abc'"),
        Ok("abc".to_string())
    );
}

#[test]
fn we_can_correctly_escape_the_single_quote_character() {
    assert_eq!(
        StringLiteralParser::new().parse("'this isn''t a test'"),
        Ok("this isn't a test".to_string())
    );
}

#[test]
fn we_can_parse_empty_strings() {
    assert_eq!(StringLiteralParser::new().parse("''"), Ok(String::new()));
}

#[test]
fn we_can_parse_strings_with_a_single_character() {
    assert_eq!(StringLiteralParser::new().parse("'a'"), Ok("a".to_string()));
}

#[test]
fn we_can_parse_strings_starting_with_numbers() {
    assert_eq!(
        StringLiteralParser::new().parse("'123a'"),
        Ok("123a".to_string())
    );
}

#[test]
fn we_can_parse_strings_having_multiple_double_quotes() {
    assert_eq!(
        StringLiteralParser::new().parse("'\"123a\"'"),
        Ok("\"123a\"".to_string())
    );
}

#[test]
fn we_cannot_parse_strings_having_more_than_two_quotes() {
    assert!(StringLiteralParser::new().parse("''123a''").is_err());
}

#[test]
fn we_can_parse_strings_containing_spaces() {
    assert_eq!(
        StringLiteralParser::new().parse("'  a12fdf 3a  '"),
        Ok("  a12fdf 3a  ".to_string())
    );
}

#[test]
fn we_can_parse_strings_starting_with_special_characters() {
    assert_eq!(
        StringLiteralParser::new().parse("'$abc'"),
        Ok("$abc".to_string())
    );
}

#[test]
fn we_can_parse_strings_having_unicode_characters() {
    assert_eq!(
        StringLiteralParser::new().parse("'a茶a'"),
        Ok("a茶a".to_string())
    );
}

#[test]
fn we_can_parse_strings_having_whitespace_characters() {
    assert_eq!(
        StringLiteralParser::new().parse("'abc\n12\r3\t'"),
        Ok("abc\n12\r3\t".to_string())
    );
    assert_eq!(
        StringLiteralParser::new().parse(
            "'abc

    ab
123'"
        ),
        Ok("abc\n\n    ab\n123".to_string())
    );
}

#[test]
fn we_can_parse_strings_having_control_characters() {
    assert_eq!(
        StringLiteralParser::new().parse("'\x1F'"),
        Ok("\x1F".to_string())
    );
    assert_eq!(
        StringLiteralParser::new().parse("'abc\x1F'"),
        Ok("abc\x1F".to_string())
    );
}

#[allow(clippy::unicode_not_nfc)]
#[test]
fn unnormalized_strings_should_differ() {
    let lhs = StringLiteralParser::new().parse("'á'").unwrap();
    let rhs = StringLiteralParser::new().parse("'á'").unwrap();
    assert_ne!(lhs, rhs);
}

#[test]
fn we_cannot_parse_strings_having_incorrect_quotes() {
    assert!(StringLiteralParser::new().parse("").is_err());
    assert!(StringLiteralParser::new().parse("'").is_err());
    assert!(StringLiteralParser::new().parse("a").is_err());
    assert!(StringLiteralParser::new().parse("'a").is_err());
    assert!(StringLiteralParser::new().parse("a'").is_err());
    assert!(StringLiteralParser::new().parse("\"a\"").is_err());
}

// Select Query parser Tests
#[test]
fn we_can_parse_a_query_with_constants() {
    let ast =
        "SELECT 3 as bigint, true as boolean, 'proof' as varchar, -2.34 as decimal FROM SXT_TAB;"
            .parse::<SelectStatement>()
            .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                col_res(lit(3), "bigint"),
                col_res(lit(true), "boolean"),
                col_res(lit("proof"), "varchar"),
                col_res(lit("-2.34".parse::<BigDecimal>().unwrap()), "decimal"),
            ],
            tab(None, "sxt_tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_a_column_equals_a_simple_bool() {
    let ast = "SELECT A FROM SXT_TAB WHERE A = false"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("a"), lit(false)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_a_column_equals_a_simple_integer() {
    let ast = "SELECT A FROM SXT_TAB WHERE A = 3;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("a"), lit(3)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_a_column_equals_a_string() {
    let ast = "SELECT A FROM SXT_TAB WHERE A = 'this_is_a_test'"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("a"), lit("this_is_a_test")),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_a_column_equals_a_decimal() {
    let ast = "SELECT A FROM SXT_TAB WHERE A = -0.32;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("a"), lit("-0.32".parse::<BigDecimal>().unwrap())),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_two_result_columns() {
    let ast = "Select a,  b froM sxt_tab where C = D + 1 and E = F and G"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a", "b"]),
            tab(None, "sxt_tab"),
            and(
                and(
                    equal(col("c"), col("d") + lit(1)),
                    equal(col("e"), col("f")),
                ),
                col("g"),
            ),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

//TODO: Add unary negative operator so that it will no longer be necessary to use the minus sign with brackets
//(PROOF-864)
#[test]
fn we_can_parse_a_query_using_select_star() {
    let ast = "SELECT * FROM sxt_Tab WHERE A = -(B);"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            vec![col_res_all()],
            tab(None, "sxt_tab"),
            equal(col("a"), lit(-1_i64) * col("b")),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_using_select_star_and_a_const() {
    let ast = "SELECT *, 4 as bigint FROM sxt_Tab WHERE A = B + 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            vec![col_res_all(), col_res(lit(4), "bigint")],
            tab(None, "sxt_tab"),
            equal(col("a"), col("b") + lit(3)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_using_multiple_select_star_expressions() {
    let ast = "SELECT a, *, b, c, * FROM sxt_Tab WHERE A - B = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            vec![
                col_res(col("a"), "a"),
                col_res_all(),
                col_res(col("b"), "b"),
                col_res(col("c"), "c"),
                col_res_all(),
            ],
            tab(None, "sxt_tab"),
            equal(col("a") - col("b"), lit(3)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_equals_filter_having_a_positive_literal() {
    let ast = "select a, true as boolean from sxt_tab where b = +4;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            vec![col_res(col("a"), "a"), col_res(lit(true), "boolean")],
            tab(None, "sxt_tab"),
            equal(col("b"), lit(4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_equals_filter_having_a_negative_literal() {
    let ast = "select a from sxt_tab where b = -4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("b"), lit(-4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_not_equals_filter_expression() {
    for not_equals_sign in ["!=", "<>"] {
        let ast = ("select a from sxt_tab where b".to_owned() + not_equals_sign + " -4")
            .parse::<SelectStatement>()
            .unwrap();
        let expected_ast = select(
            query(
                cols_res(&["a"]),
                tab(None, "sxt_tab"),
                not(equal(col("b"), lit(-4))),
                vec![],
            ),
            vec![],
            None,
        );
        assert_eq!(ast, expected_ast);
    }
}

#[test]
fn we_can_parse_a_query_with_one_logical_not_filter_expression() {
    let ast = "select a from sxt_tab where not (b = d + 3);"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            not(equal(col("b"), col("d") + lit(3))),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_logical_and_filter_expression() {
    let ast = "select a from sxt_tab where (b = 3) and c"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            and(equal(col("b"), lit(3)), col("c")),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_logical_and_filter_expression_with_both_left_and_right_side_equal_to_string_literals(
) {
    let ast = "select bid_in_usd_over_1e2 from sxt.options_quote where type = 'call' and security = 'eth'".parse::<SelectStatement>().unwrap();
    let expected_ast = select(
        query(
            cols_res(&["bid_in_usd_over_1e2"]),
            tab(Some("sxt"), "options_quote"),
            and(
                equal(col("type"), lit("call")),
                equal(col("security"), lit("eth")),
            ),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_one_logical_or_filter_expression() {
    let ast = "select a from sxt_tab where (b = 3) or (c = -2.34);"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            or(
                equal(col("b"), lit(3)),
                equal(col("c"), lit("-2.34".parse::<BigDecimal>().unwrap())),
            ),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_two_logical_and_not_filter_expressions() {
    let ast = "select a from sxt_tab where (b = 3) and (not (c = d - 2));"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            and(
                equal(col("b"), lit(3)),
                not(equal(col("c"), col("d") - lit(2))),
            ),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_three_logical_not_and_or_filter_expressions() {
    let ast = "select a from sxt_tab where not ((b = 3 * 7) and  ((f = 45) or (c + 2 * d = -2)))"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            not(and(
                equal(col("b"), lit(3) * lit(7)),
                or(
                    equal(col("f"), lit(45)),
                    equal(col("c") + lit(2) * col("d"), lit(-2)),
                ),
            )),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_the_minimum_i128_value_as_the_equal_filter_literal() {
    let ast = ("select a from sxt_tab where b = ".to_owned() + &i128::MIN.to_string())
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("b"), lit(i128::MIN)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);

    let ast = "select a from sxt_tab where b = -170141183460469231731687303715884105728;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("b"), lit(i128::MIN)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_query_with_the_literals_overflowing() {
    // note: see the minus sign in front of the literal, causing the overflow
    assert!(
        ("select a from sxt_tab where b = -".to_owned() + &i128::MIN.to_string())
            .parse::<SelectStatement>()
            .is_err()
    );
}

#[test]
fn we_can_parse_a_query_with_the_maximum_i128_value_as_the_equal_filter_literal() {
    let ast = ("select a from sxt_tab where b = ".to_owned() + &i128::MAX.to_string())
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            equal(col("b"), lit(i128::MAX)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_and_rename_a_result_column_using_the_as_keyword() {
    let ast = "select a as a_rename from sxt_tab where b = 4 + d;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            vec![col_res(col("a"), "a_rename")],
            tab(None, "sxt_tab"),
            equal(col("b"), lit(4) + col("d")),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_and_rename_a_result_column_without_using_the_as_keyword() {
    let parsed_ast = "select a a_rename from sxt_tab where b >= c + 4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            vec![col_res(col("a"), "a_rename")],
            tab(None, "sxt_tab"),
            ge(col("b"), col("c") + lit(4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_not_with_more_precedence_priority_than_logical_and() {
    let parsed_ast = "select a from sxt_tab where a = 3 and not b = 2.53;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = "select a from sxt_tab where (a = 3) and (not b = 2.53)"
        .parse::<SelectStatement>()
        .unwrap();
    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_not_with_less_precedence_priority_than_equal_operator() {
    let parsed_ast = "select a from sxt_tab where not b = d"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "sxt_tab"),
            not(equal(col("b"), col("d"))),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(parsed_ast, expected_ast);
}

#[test]
fn we_can_parse_logical_and_with_more_precedence_priority_than_logical_or() {
    let ast = "select a from sxt_tab where a = -1 or c = -3 and a = 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = "select a from sxt_tab where a = -1 or (c = -3 and a = 3);"
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
fn we_can_parse_a_query_with_one_schema_followed_by_a_table_name() {
    let ast = "select a from eth.sxt_tab where b <= 4;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(Some("eth"), "sxt_tab"),
            le(col("b"), lit(4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_without_a_filter() {
    let ast = "select a from tab".parse::<SelectStatement>().unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);

    let ast = "select * from eth.tab".parse::<SelectStatement>().unwrap();
    let expected_ast = select(
        query_all(vec![col_res_all()], tab(Some("eth"), "tab"), vec![]),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_single_order_by_with_ascending_direction_as_default() {
    let ast = "select a from tab order by x;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        order("x", Asc),
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_single_order_by_with_a_filter() {
    let ast = "select a from tab where y = 3 order by x"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            equal(col("y"), lit(3)),
            vec![],
        ),
        order("x", Asc),
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_single_order_by_in_the_ascending_direction() {
    let ast = "select a from tab order by x asc;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        order("x", Asc),
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_single_order_by_in_the_descending_direction() {
    let ast = "select a from tab order by x desc"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        order("x", Desc),
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_multiple_order_by() {
    let ast = "select * from tab order by x desc, y, z asc;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(vec![col_res_all()], tab(None, "tab"), vec![]),
        orders(&["x", "y", "z"], &[Desc, Asc, Asc]),
        None,
    );
    assert_eq!(ast, expected_ast);
}

// TODO: we should be able to pass this test.
// But due to some lalrpop restriction, we aren't.
// This problem will be addressed in a future PR.
#[allow(clippy::should_panic_without_expect)]
#[test]
#[should_panic]
fn we_cannot_parse_order_by_referencing_reserved_keywords_yet() {
    let ast = "select a as asc from tab order by a asc"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(vec![col_res(col("a"), "asc")], tab(None, "tab"), vec![]),
        orders(&["a"], &[Asc]),
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_queries_with_stray_semicolons() {
    assert!("select a from tab order by ;x"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select a from ; tab".parse::<SelectStatement>().is_err());
    assert!("select a from tab;;".parse::<SelectStatement>().is_err());
    assert!("select a ; from tab;".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_invalid_order_by_expressions() {
    assert!("select a from tab order by x y"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select a from tab order by x, y asc desc"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select a from tab order by x, asc y"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select a from tab order by x asc y"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_two_schemas_followed_by_a_table_name() {
    assert!("select a from schema.Identifier.tab;"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_query_with_a_filter_value_smaller_than_min_i128_as_it_will_overflow() {
    assert!(
        "select a from tab where b = -170141183460469231731687303715884105729"
            .parse::<SelectStatement>()
            .is_err()
    );
}

#[test]
fn we_cannot_parse_a_query_with_a_filter_value_bigger_than_max_i128_as_it_will_overflow() {
    assert!(
        "select a from schema.tab where b = 170141183460469231731687303715884105728"
            .parse::<SelectStatement>()
            .is_err()
    );
}

#[test]
fn we_cannot_parse_a_query_with_select_tablename_followed_by_star() {
    assert!("select tab.* from tab".parse::<SelectStatement>().is_err());
}

#[test]
fn we_cannot_parse_a_query_with_schemas_followed_by_column_and_table_names() {
    assert!("select tab.a from tab".parse::<SelectStatement>().is_err());
    assert!("select tab.a from eth.tab;"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select eth.tab.a from eth.tab"
        .parse::<SelectStatement>()
        .is_err());
    assert!("select a from eth.tab where tab.b = 3;"
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
fn we_can_parse_a_query_having_a_simple_limit_clause() {
    let ast = "select a from tab limit 3"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        vec![],
        slice(3, 0),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_query_having_a_negative_limit_clause() {
    assert!("select a from tab limit -3"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_can_parse_a_query_having_a_simple_offset_clause() {
    let ast = "select a from tab offset 3;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        vec![],
        slice(u64::MAX, 3),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_negative_offset_clause() {
    let ast = "select a from tab offset -3;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        vec![],
        slice(u64::MAX, -3),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_simple_limit_and_offset_clause() {
    let ast = "select a from tab limit 55 offset 3;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        vec![],
        slice(55, 3),
    );
    assert_eq!(ast, expected_ast);

    let ast = "select a from tab offset 3 limit 55;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), vec![]),
        vec![],
        slice(55, 3),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_simple_limit_and_offset_clause_preceded_by_where_expr_and_order_by(
) {
    let ast = "select a from tab where a = 3 order by a limit 55 offset 3;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            equal(col("a"), lit(3)),
            vec![],
        ),
        order("a", Asc),
        slice(55, 3),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_query_having_a_simple_limit_and_offset_clause_preceded_by_where_expr_but_followed_by_order_by(
) {
    assert!(
        "select a from tab where a = 3 limit 55 offset 3 order by a;"
            .parse::<SelectStatement>()
            .is_err()
    );
}

#[test]
fn we_can_parse_a_query_with_filter_ge() {
    let ast = "select a from tab where b >= 4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            ge(col("b"), lit(4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_filter_lt() {
    let ast = "select a from tab where b < 4;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            lt(col("b"), lit(4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_filter_le() {
    let ast = "select a from tab where b <= 4"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            le(col("b"), lit(4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_filter_gt() {
    let ast = "select a from tab where b > 4;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            gt(col("b"), lit(4)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_query_with_inner_join_keyword() {
    assert!(
        "select tab1.a from tab1 join tab2 on tab1.c = tab2.c where tab2.b > 4;"
            .parse::<SelectStatement>()
            .is_err()
    );
}

// Case when
#[test]
fn we_cannot_parse_a_query_with_case_when_keyword() {
    assert!(
        "select case when a == 2 then 3 else 5 from tab where b <= 4;"
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

////////////////////////////////
/// Tests for the `GroupByClause`
////////////////////////////////
#[test]
fn we_can_parse_a_simple_group_by_clause() {
    let ast = "select a from tab group by a;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(cols_res(&["a"]), tab(None, "tab"), group_by(&["a"])),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}
#[test]
fn we_can_parse_a_simple_group_by_clause_with_multiple_columns() {
    let ast = "select a from tab group by a, b, d;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            cols_res(&["a"]),
            tab(None, "tab"),
            group_by(&["a", "b", "d"]),
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_simple_group_by_clause_using_the_wildcard() {
    let ast = "select * from tab group by a"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(vec![col_res_all()], tab(None, "tab"), group_by(&["a"])),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_group_by_clause_containing_multiple_aggregations() {
    let ast = "select min(a), max(a) as max_a, count(a), count(*) count_all from tab group by a, b"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                min_res(col("a"), "__min__"),
                max_res(col("a"), "max_a"),
                count_res(col("a"), "__count__"),
                count_all_res("count_all"),
            ],
            tab(None, "tab"),
            group_by(&["a", "b"]),
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_group_by_clause_containing_multiple_aggregations_where_clause_order_by_and_limit()
{
    let ast = "select min(a), max(a) as max_a, sum(c), count(a), count(*) count_all from tab where d = 3 group by a, b order by b limit 2;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            vec![
                min_res(col("a"), "__min__"),
                max_res(col("a"), "max_a"),
                sum_res(col("c"), "__sum__"),
                count_res(col("a"), "__count__"),
                count_all_res("count_all"),
            ],
            tab(None, "tab"),
            equal(col("d"), lit(3)),
            group_by(&["a", "b"]),
        ),
        order("b", Asc),
        slice(2, 0),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_group_by_clause_after_order_by() {
    assert!("select a from tab order by a group by a"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_cannot_parse_a_group_by_clause_before_where_expr() {
    assert!("select a from tab group by a where a = 3;"
        .parse::<SelectStatement>()
        .is_err());
}

#[test]
fn we_can_parse_a_aggregations_without_group_by_although_it_is_semantically_incorrect() {
    let ast = "select f as f_col, min(a), max(a) as max_a, count(a), count(*) count_all from tab"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                col_res(col("f"), "f_col"),
                min_res(col("a"), "__min__"),
                max_res(col("a"), "max_a"),
                count_res(col("a"), "__count__"),
                count_all_res("count_all"),
            ],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_a_non_count_aggregations_with_wildcard() {
    assert!("select min(*) from tab".parse::<SelectStatement>().is_err());
    assert!("select max(*) from tab".parse::<SelectStatement>().is_err());
    assert!("select sum(*) from tab".parse::<SelectStatement>().is_err());
}

#[test]
fn we_can_parse_a_simple_add_mul_sub_arithmetic_expressions_in_the_result_expr() {
    let ast = "select a + b, 2 * f, -77 - h, sum(a) / sum(b) from tab;"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                col_res(col("a") + col("b"), "__expr__"),
                col_res(lit(2) * col("f"), "__expr__"),
                col_res(lit(-77) - col("h"), "__expr__"),
                col_res(col("a").sum() / col("b").sum(), "__expr__"),
            ],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn mul_and_div_operators_have_the_same_precedence_and_left_expressions_are_always_parsed_first() {
    let ast = "select a * b / c, (a * b) / c, a * (b / c) from tab"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                col_res(col("a") * col("b") / col("c"), "__expr__"),
                col_res((col("a") * col("b")) / col("c"), "__expr__"),
                col_res(col("a") * (col("b") / col("c")), "__expr__"),
            ],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_single_literal_in_the_result_expr() {
    let ast = "select -123 from tab".parse::<SelectStatement>().unwrap();
    let expected_ast = select(
        query_all(
            vec![col_res(lit(-123), "__expr__")],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_single_boolean_literal_in_the_result_expr() {
    let ast = "select false from tab".parse::<SelectStatement>().unwrap();
    let expected_ast = select(
        query_all(
            vec![col_res(lit(false), "__expr__")],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_literals_outside_of_i128_range_in_the_result_expr() {
    assert!("select 170141183460469231731687303715884105727 from tab"
        .parse::<SelectStatement>()
        .is_ok());
    assert_eq!(
        "select 170141183460469231731687303715884105728 from tab".parse::<SelectStatement>(),
        Err(super::error::ParseError::QueryParseError {
            error: "i128 out of range".to_string()
        })
    );
    assert!("select -170141183460469231731687303715884105728 from tab"
        .parse::<SelectStatement>()
        .is_ok());
    assert_eq!(
        "select -170141183460469231731687303715884105729 from tab".parse::<SelectStatement>(),
        Err(super::error::ParseError::QueryParseError {
            error: "i128 out of range".to_string()
        })
    );
}

#[test]
fn we_can_parse_multiple_arithmetic_expression_where_multiplication_has_precedence_in_the_result_expr(
) {
    let ast = "select (2 + f) * (c + g + 2 * h), ((h - g) * 2 + c + g) * (f + 2) as d from tab"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                col_res(
                    (lit(2) + col("f")) * (col("c") + col("g") + lit(2) * col("h")),
                    "__expr__",
                ),
                col_res(
                    ((col("h") - col("g")) * lit(2) + col("c") + col("g")) * (col("f") + lit(2)),
                    "d",
                ),
            ],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_arithmetic_expression_within_aggregations_in_the_result_expr() {
    let ast = "select sum(2 * f + c) as d from tab"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![sum_res(lit(2) * col("f") + col("c"), "d")],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_arithmetic_expression_within_aggregations_and_non_aggregations_in_the_result_expr()
{
    let ast = "select sum(2 * f + c) as d, g - k from tab group by g"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                sum_res(lit(2) * col("f") + col("c"), "d"),
                col_res(col("g") - col("k"), "__expr__"),
            ],
            tab(None, "tab"),
            group_by(&["g"]),
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_use_arithmetic_outside_aggregation_functions() {
    let ast = "select 2 * f - y, 3 * a - sum(f) * max(y) - min(d) + 2 from employees group by f"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![
                col_res(lit(2) * col("f") - col("y"), "__expr__"),
                col_res(
                    lit(3) * col("a") - col("f").sum() * col("y").max() - col("d").min() + lit(2),
                    "__expr__",
                ),
            ],
            tab(None, "employees"),
            group_by(&["f"]),
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_use_aggregation_inside_another_aggregation() {
    let ast = "select sum(max(2 * a)) from tab"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query_all(
            vec![col_res((lit(2) * col("a")).max().sum(), "__sum__")],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_null_literal() {
    let ast = "SELECT NULL FROM tab".parse::<SelectStatement>().unwrap();
    let expected_ast = select(
        query_all(
            vec![col_res(lit(Literal::Null), "__expr__")],
            tab(None, "tab"),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_is_null_expression() {
    let ast = "SELECT a FROM tab WHERE a IS NULL"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            is_null(col("a")),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_is_not_null_expression() {
    let ast = "SELECT a FROM tab WHERE a IS NOT NULL"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            is_not_null(col("a")),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_is_true_expression() {
    let ast = "SELECT a FROM tab WHERE a IS TRUE"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            is_true(col("a")),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_comparing_with_null() {
    let ast = "SELECT a FROM tab WHERE a = NULL"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            equal(col("a"), lit(Literal::Null)),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_with_complex_null_expressions() {
    let ast = "SELECT a FROM tab WHERE (a IS NULL) OR (b IS NOT NULL AND c IS TRUE)"
        .parse::<SelectStatement>()
        .unwrap();
    let expected_ast = select(
        query(
            cols_res(&["a"]),
            tab(None, "tab"),
            or(
                is_null(col("a")),
                and(is_not_null(col("b")), is_true(col("c"))),
            ),
            vec![],
        ),
        vec![],
        None,
    );
    assert_eq!(ast, expected_ast);
}
