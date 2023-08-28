use crate::base::database::{TableRef, TestAccessor};
use crate::record_batch;
use crate::sql::ast::test_utility::*;
use crate::sql::parse::{Converter, QueryExpr};
use crate::sql::transform::test_utility::*;
use proofs_sql::intermediate_ast::OrderByDirection::{Asc, Desc};

use arrow::record_batch::RecordBatch;
use polars::prelude::col as pcol;
use proofs_sql::sql::SelectStatementParser;

fn query_to_provable_ast(table: TableRef, query: &str, accessor: &TestAccessor) -> QueryExpr {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    Converter::default()
        .visit_intermediate_ast(&intermediate_ast, accessor, table.schema_id())
        .unwrap()
}

fn invalid_query_to_provable_ast(table: TableRef, query: &str, accessor: &TestAccessor) {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    assert!(Converter::default()
        .visit_intermediate_ast(&intermediate_ast, accessor, table.schema_id())
        .is_err());
}

pub fn record_batch_to_accessor(table: TableRef, data: RecordBatch, offset: usize) -> TestAccessor {
    let mut accessor = TestAccessor::new();
    accessor.add_table(table, data, offset);
    accessor
}

#[test]
fn we_can_convert_an_ast_with_one_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64]
        ),
        0,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_column_and_i128_data() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i128]
        ),
        0,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", 3_i128, &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_column_and_a_filter_by_a_string_literal() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => ["abc"]
        ),
        0,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where a = 'abc'", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", "abc", &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_duplicate_aliases() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64],
            "b" => [4_i64]
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select a as c, b as c from sxt_tab where a = 3",
        &accessor,
    );
    invalid_query_to_provable_ast(t, "select a as b, b from sxt_tab where a = 3", &accessor);
    invalid_query_to_provable_ast(
        t,
        "select a as b, a as b from sxt_tab where a = 3",
        &accessor,
    );
    invalid_query_to_provable_ast(t, "select a, a from sxt_tab where a = 3", &accessor);
}

#[test]
fn we_dont_have_duplicate_filter_result_expressions() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64]
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a as b, a as c from sxt_tab where a = 3",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(&[("a", "b"), ("a", "c")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_two_columns() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a,  b from sxt_tab where c = 123", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b"], &accessor),
            tab(t),
            equal(t, "c", 123, &accessor),
        ),
        result(&[("a", "a"), ("b", "b")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_select_star() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5_i64, 6],
            "a" => [3_i64, 2],
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select * from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(&[("b", "b"), ("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_positive_cond() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where b = +4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "b", 4, &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_not_equals_cond() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where b <> +4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            not(equal(t, "b", 4, &accessor)),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_negative_cond() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where b = -4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "b", -4, &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_and() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where (b = 3) and (c = -2)",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            and(equal(t, "b", 3, &accessor), equal(t, "c", -2, &accessor)),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_or() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where (b = 3) or (c = -2)",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            or(equal(t, "b", 3, &accessor), equal(t, "c", -2, &accessor)),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_or_not() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where (b = 3) or (not (c = -2))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            or(
                equal(t, "b", 3, &accessor),
                not(equal(t, "c", -2, &accessor)),
            ),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_not_and_or() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
            "f" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where not (((f = 45) or (c = -2)) and (b = 3))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            not(and(
                or(equal(t, "f", 45, &accessor), equal(t, "c", -2, &accessor)),
                equal(t, "b", 3, &accessor),
            )),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_min_i128_filter_value() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where a = -170141183460469231731687303715884105728",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", i128::MIN, &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_max_i128_filter_value() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where a = 170141183460469231731687303715884105727",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", i128::MAX, &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_using_an_aliased_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a as b_rename from sxt_tab where b = +4",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![col_result(t, "a", &accessor)],
            tab(t),
            equal(t, "b", 4, &accessor),
        ),
        result(&[("a", "b_rename")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_a_nonexistent_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [3_i64],
        ),
        0,
    );
    invalid_query_to_provable_ast(t, "select * from sxt_tab where a = 3", &accessor);
}

#[test]
fn we_cannot_convert_an_ast_with_a_column_type_different_than_equal_literal() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => ["abc"],
        ),
        0,
    );
    invalid_query_to_provable_ast(t, "select * from sxt_tab where b = 123", &accessor);
}

#[test]
fn we_can_convert_an_ast_with_a_schema() {
    let t = "eth.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64],
        ),
        0,
    );
    let ast = query_to_provable_ast(t, "select a from eth.sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_without_any_filter() {
    let t = "eth.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64],
        ),
        0,
    );
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        result(&[("a", "a")]),
    );
    let queries = ["select * from eth.sxt_tab", "select a from eth.sxt_tab"];
    for query in queries {
        let ast = query_to_provable_ast(t, query, &accessor);
        assert_eq!(ast, expected_ast);
    }
}

/////////////////////////
/// OrderBy
/////////////////////////
#[test]
fn we_can_parse_order_by_with_a_single_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5_i64, 6],
            "a" => [3_i64, 2],
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select * from sxt_tab where a = 3 order by b", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        composite_result(vec![
            select(&[pcol("b").alias("b"), pcol("a").alias("a")]),
            orders(&["b"], &[Asc]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_order_by_with_multiple_columns() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5_i64, 6, -7],
            "a" => [3_i64, 2, 3],
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a, b from sxt_tab where a = 3 order by b desc, a asc",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        composite_result(vec![
            select(&[pcol("a").alias("a"), pcol("b").alias("b")]),
            orders(&["b", "a"], &[Desc, Asc]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_order_by_referencing_an_alias_associated_with_column_b_but_with_name_equals_column_a_also_renamed(
) {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5_i64, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select salary as s, name as salary from sxt_tab where salary = 5 order by salary desc",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_result(t, "name", &accessor),
                col_result(t, "salary", &accessor),
            ],
            tab(t),
            equal(t, "salary", 5, &accessor),
        ),
        composite_result(vec![
            select(&[pcol("salary").alias("s"), pcol("name").alias("salary")]),
            orders(&["salary"], &[Desc]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_order_by_referencing_a_column_name_instead_of_an_alias() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5_i64, 6],
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select salary as s from sxt_tab order by salary",
        &accessor,
    );
}

#[test]
fn we_cannot_parse_order_by_referencing_an_alias_associated_with_column_b_but_with_name_equals_column_a_not_renamed(
) {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5_i64, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select salary, name as salary from sxt_tab where salary = 5 order by salary desc",
        &accessor,
    );
}

#[test]
fn we_cannot_parse_order_by_referencing_an_existing_column_not_appearing_in_the_result_select_list()
{
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5_i64, 6],
            "a" => [3_i64, 2],
        ),
        0,
    );

    // Note: While this operation is acceptable with PostgreSQL, we do not currently support it.
    invalid_query_to_provable_ast(t, "select a from sxt_tab order by b desc", &accessor);
}

#[test]
fn we_cannot_parse_order_by_referencing_an_alias_name_associated_with_two_different_columns() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5_i64, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select salary as s, name as s from sxt_tab order by s desc",
        &accessor,
    );

    invalid_query_to_provable_ast(
        t,
        "select salary as name, name from sxt_tab order by name desc",
        &accessor,
    );

    // Note: While this is not ambiguous with PostgreSQL,
    // it currently is with our code because there is
    // no way to differentiate between the two columns
    // in the record batch since they share the same name.
    invalid_query_to_provable_ast(
        t,
        "select salary as name, name from sxt_tab order by salary desc",
        &accessor,
    );

    // Note: This is not ambiguous with PostgreSQL either,
    // but it is with our code for the reasons mentioned above.
    invalid_query_to_provable_ast(
        t,
        "select salary as s, name as s from sxt_tab order by salary desc",
        &accessor,
    );
}

#[test]
fn we_can_parse_order_by_queries_with_the_same_column_name_appearing_more_than_once_and_with_different_alias_name(
) {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5_i64, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );

    for order_by in ["s", "d"] {
        let ast = query_to_provable_ast(
            t,
            &("select salary as s, name, salary as d from sxt_tab order by ".to_owned() + order_by),
            &accessor,
        );
        let expected_ast = QueryExpr::new(
            filter(
                vec![
                    col_result(t, "name", &accessor),
                    col_result(t, "salary", &accessor),
                ],
                tab(t),
                const_v(true),
            ),
            composite_result(vec![
                select(&[
                    pcol("salary").alias("s"),
                    pcol("name").alias("name"),
                    pcol("salary").alias("d"),
                ]),
                orders(&[order_by], &[Asc]),
            ]),
        );
        assert_eq!(ast, expected_ast);
    }
}

/////////////////////////
// Slice
/////////////////////////

#[test]
fn we_can_parse_a_query_having_a_simple_limit_clause() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(t, "select a from sxt_tab limit 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        composite_result(vec![select(&[pcol("a").alias("a")]), slice(3, 0)]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn no_slice_is_applied_when_limit_is_u64_max_and_offset_is_zero() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(t, "select a from sxt_tab offset 0", &accessor);
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        composite_result(vec![select(&[pcol("a").alias("a")])]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_simple_positive_offset_clause() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(t, "select a from sxt_tab offset 7", &accessor);
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        composite_result(vec![select(&[pcol("a").alias("a")]), slice(u64::MAX, 7)]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_negative_offset_clause() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(t, "select a from sxt_tab offset -7", &accessor);
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        composite_result(vec![select(&[pcol("a").alias("a")]), slice(u64::MAX, -7)]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_simple_limit_and_offset_clause() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(t, "select a from sxt_tab limit 55 offset 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        composite_result(vec![select(&[pcol("a").alias("a")]), slice(55, 3)]),
    );
    assert_eq!(ast, expected_ast);
}

///////////////////////////
// Composition Expressions
///////////////////////////
#[test]
fn we_can_parse_a_query_having_a_simple_limit_and_offset_clause_preceded_by_where_expr_and_order_by(
) {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where a = -3 order by a desc limit 55 offset 3",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", -3, &accessor),
        ),
        composite_result(vec![
            select(&[pcol("a").alias("a")]),
            orders(&["a"], &[Desc]),
            slice(55, 3),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

///////////////////////////
// Group By Expressions
///////////////////////////
#[test]
fn we_can_group_by_without_using_aggregate_functions() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "select department from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["department"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(vec![pcol("department").alias("department")], vec![]),
            select(&[pcol("department")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_group_by_and_then_use_a_single_aggregate_function() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "SELECT max(salary) from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["department", "salary"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(
                vec![pcol("department").alias("#$department")],
                vec![agg_expr("max", "salary", "__max__")],
            ),
            select(&[pcol("__max__")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_have_multiple_group_by_and_then_use_multiple_aggregate_functions() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => [-7_i64]
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "SELECT max(salary) as max_sal, department as d, count(department) from employees group by department, bonus",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["bonus", "department", "salary"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(
                vec![
                    pcol("department").alias("d"),
                    pcol("bonus").alias("#$bonus"),
                ],
                vec![
                    agg_expr("max", "salary", "max_sal"),
                    agg_expr("count", "department", "__count__"),
                ],
            ),
            select(&[pcol("max_sal"), pcol("d"), pcol("__count__")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn group_by_expressions_are_parsed_before_an_order_by_referencing_an_aggregate_alias_result() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => [-7_i64]
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "SELECT max(salary) as max_sal, department as d, count(department) from employees group by department, bonus order by max_sal",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["bonus", "department", "salary"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(
                vec![
                    pcol("department").alias("d"),
                    pcol("bonus").alias("#$bonus"),
                ],
                vec![
                    agg_expr("max", "salary", "max_sal"),
                    agg_expr("count", "department", "__count__"),
                ],
            ),
            select(&[pcol("max_sal"), pcol("d"), pcol("__count__")]),
            orders(&["max_sal"], &[Asc]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_non_aggregated_or_group_by_columns_in_select_clause() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select department, salary from sxt.employees group by department",
        &accessor,
    );
}

#[test]
fn aggregate_functions_are_not_allowed_in_the_group_by() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select department, min(salary) as min_salary from employees group by min_salary",
        &accessor,
    );
}

#[test]
fn order_by_cannot_reference_an_invalid_group_by_column() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
        ),
        0,
    );

    invalid_query_to_provable_ast(
        t,
        "select department as d from sxt.employees group by department order by department",
        &accessor,
    );

    invalid_query_to_provable_ast(
        t,
        "select department, min(salary) from sxt.employees group by department order by salary",
        &accessor,
    );
}

#[test]
fn group_by_column_cannot_be_a_column_result_alias() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
        ),
        0,
    );

    invalid_query_to_provable_ast(
        t,
        "select min(salary) as min_sal from sxt.employees group by min_sal",
        &accessor,
    );
}

#[test]
fn we_cannot_have_aggregate_functions_without_a_group_by_clause() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => ["abc"]
        ),
        0,
    );

    invalid_query_to_provable_ast(t, "select count(bonus) from sxt.employees", &accessor);
}

#[test]
fn we_can_parse_a_query_having_group_by_with_the_same_name_as_the_aggregation_expression() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => ["abc"]
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select count(bonus) department from sxt.employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["bonus", "department"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(
                vec![pcol("department").alias("#$department")],
                vec![agg_expr("count", "bonus", "department")],
            ),
            select(&[pcol("department")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn min_max_sum_aggregate_functions_cannot_be_used_with_non_numeric_columns() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => ["abc"]
        ),
        0,
    );

    invalid_query_to_provable_ast(
        t,
        "select department, max(bonus) from sxt.employees group by department",
        &accessor,
    );

    invalid_query_to_provable_ast(
        t,
        "select department, min(bonus) from sxt.employees group by department",
        &accessor,
    );

    invalid_query_to_provable_ast(
        t,
        "select department, sum(bonus) from sxt.employees group by department",
        &accessor,
    );
}

#[test]
fn count_aggregate_functions_can_be_used_with_non_numeric_columns() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => ["abc"]
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select department, count(bonus), count(department) as dep from sxt.employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["bonus", "department"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(
                vec![pcol("department").alias("department")],
                vec![
                    agg_expr("count", "bonus", "__count__"),
                    agg_expr("count", "department", "dep"),
                ],
            ),
            select(&[pcol("department"), pcol("__count__"), pcol("dep")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn count_all_uses_the_first_group_by_identifier_as_default_result_column() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => ["abc"]
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select count(*) from sxt.employees where salary = 4 group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["department"], &accessor),
            tab(t),
            equal(t, "salary", 4, &accessor),
        ),
        composite_result(vec![
            groupby(
                vec![pcol("department").alias("#$department")],
                vec![agg_expr("count", "department", "__count__")],
            ),
            select(&[pcol("__count__")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn aggregate_result_columns_cannot_reference_invalid_columns() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => ["abc"]
        ),
        0,
    );

    invalid_query_to_provable_ast(
        t,
        "select department, max(non_existent) from sxt.employees group by department",
        &accessor,
    );
}

#[test]
fn we_can_use_the_same_result_columns_with_different_aliases_and_associate_it_with_group_by() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => [-7_i64]
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "SELECT department as d1, department as d2 from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["department"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(
                vec![
                    pcol("department").alias("d1"),
                    pcol("department").alias("d2"),
                ],
                vec![],
            ),
            select(&[pcol("d1"), pcol("d2")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_use_multiple_group_by_clauses() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64],
            "department" => [5_i64],
            "bonus" => [-7_i64]
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select department as d1, max(salary), department as d2, sum(bonus) as sum_bonus from employees group by department, bonus, department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["bonus", "department", "salary"], &accessor),
            tab(t),
            const_v(true),
        ),
        composite_result(vec![
            groupby(
                vec![
                    pcol("department").alias("d1"),
                    pcol("department").alias("d2"),
                    pcol("bonus").alias("#$bonus"),
                ],
                vec![
                    agg_expr("max", "salary", "__max__"),
                    agg_expr("sum", "bonus", "sum_bonus"),
                ],
            ),
            select(&[pcol("d1"), pcol("__max__"), pcol("d2"), pcol("sum_bonus")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}
