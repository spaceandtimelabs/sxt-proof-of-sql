use crate::base::database::{TableRef, TestAccessor};
use crate::record_batch;
use crate::sql::ast::test_utility::{
    and, col_result, cols_result, const_v, equal, filter, not, or, tab,
};
use crate::sql::parse::{Converter, QueryExpr};
use crate::sql::transform::test_utility::result;

use arrow::record_batch::RecordBatch;
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
            "a" => [3]
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
        result(),
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
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_select_star() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5, 6],
            "a" => [3, 2],
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select * from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["b", "a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_more_complex_select_star() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5, 6],
            "a" => [3, 2],
            "c" => [78, 8]
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a, *, b,* from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b", "a", "c", "b", "b", "a", "c"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(),
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
        result(),
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
        result(),
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
        result(),
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
        result(),
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
        result(),
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
        result(),
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
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_min_i64_filter_value() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where a = -9223372036854775808",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", i64::MIN, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_max_i64_filter_value() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where a = 9223372036854775807",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", i64::MAX, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_using_as_rename_keyword() {
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
            vec![col_result(t, "a", "b_rename", &accessor)],
            tab(t),
            equal(t, "b", 4, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_a_nonexistent_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [3],
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
            "a" => [3],
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
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_without_any_filter() {
    let t = "eth.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3],
        ),
        0,
    );
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        result(),
    );
    let queries = ["select * from eth.sxt_tab", "select a from eth.sxt_tab"];
    for query in queries {
        let ast = query_to_provable_ast(t, query, &accessor);
        assert_eq!(ast, expected_ast);
    }
}
