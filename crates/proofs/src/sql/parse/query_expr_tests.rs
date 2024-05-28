use super::ConversionError;
use crate::{
    base::database::{ColumnType, RecordBatchTestAccessor, TableRef, TestAccessor},
    record_batch,
    sql::{
        ast::{test_utility::*, ProofPlan},
        parse::QueryExpr,
        transform::{test_utility::*, LiteralConversion},
    },
};
use arrow::record_batch::RecordBatch;
use curve25519_dalek::RistrettoPoint;
use itertools::Itertools;
use polars::prelude::col as pc;
use proofs_sql::{intermediate_ast::OrderByDirection::*, sql::SelectStatementParser};

fn query_to_provable_ast(
    table: TableRef,
    query: &str,
    accessor: &RecordBatchTestAccessor,
) -> QueryExpr<RistrettoPoint> {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    QueryExpr::try_new(intermediate_ast, table.schema_id(), accessor).unwrap()
}

fn invalid_query_to_provable_ast(table: TableRef, query: &str, accessor: &RecordBatchTestAccessor) {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    assert!(
        QueryExpr::<RistrettoPoint>::try_new(intermediate_ast, table.schema_id(), accessor)
            .is_err()
    );
}

#[cfg(test)]
pub fn record_batch_to_accessor(
    table: TableRef,
    data: RecordBatch,
    offset: usize,
) -> RecordBatchTestAccessor {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(table, data, offset);
    accessor
}

fn get_test_accessor() -> (TableRef, RecordBatchTestAccessor) {
    let table = "sxt.t".parse().unwrap();
    let data = record_batch!(
        "s" => ["abc", ],
        "i" => [1_i64, ],
        "d" => [2_i128, ],

        "s0" => ["abc", ],
        "i0" => [1_i64, ],
        "d0" => [2_i128, ],

        "s1" => ["abc", ],
        "i1" => [1_i64, ],
        "d1" => [2_i128, ],
    );
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(table, data, 0);
    (table, accessor)
}

macro_rules! query {
    (select: $select:expr $(, filter: $filter:expr)? $(, group: $groupby:expr)? $(, order: $orderby:expr)? $(, limit: $limit:expr)? $(, offset: $offset:expr)? $(, should_err: $should_err:tt)? $(,)?) => {{
        let (t, accessor) = get_test_accessor();
        let mut query = String::new();
        query.push_str(&format!("select {} from t", $select.join(", ")));
        macro_rules! filter_str {
            () => {}; ($expr:expr) => { query.push_str(&format!(" where {}", $expr)) };
        }
        filter_str!($($filter)?);
        macro_rules! groupby_str {
            () => {}; ($expr:expr) => { query.push_str(&format!(" group by {}", $expr.clone().join(", "))) };
        }
        groupby_str!($($groupby)?);
        macro_rules! orderby_str {
            () => {}; ($expr:expr) => { query.push_str(&format!(" order by {}", $expr.clone().join(", "))) };
        }
        orderby_str!($($orderby)?);
        macro_rules! limit_str {
            () => {}; ($expr:expr) => { query.push_str(&format!(" limit {}", $expr.to_string())) };
        }
        limit_str!($($limit)?);
        macro_rules! offset_str {
            () => {}; ($expr:expr) => { query.push_str(&format!(" offset {}", $expr.to_string())) };
        }
        offset_str!($($offset)?);

        let intermediate_ast = SelectStatementParser::new().parse(&query).unwrap();
        let query_expr = QueryExpr::<RistrettoPoint>::try_new(intermediate_ast, t.schema_id(), &accessor);
        macro_rules! expect_err_str {
            () => { query_expr.unwrap() };
            (true) => { query_expr.unwrap_err() };
            (false) => { query_expr.unwrap() };
        }
        expect_err_str!($($should_err)?)
    }};
}

macro_rules! expected_query {
    (select: [cols = $result_columns:expr, exprs = $result_exprs:expr] $(, filter: $filter:expr)? $(, group: $group_by:expr)? $(, order: [by = $order_by:expr, dirs = $order_dirs:expr])? $(,)?) => {{
        let (t, accessor) = get_test_accessor();
        let mut result_vec = Vec::new();

        macro_rules! groupby_macro {
            (,$agg:expr) => {
                result_vec.push(select(&$result_exprs.to_vec()));
            };
            ($by:expr, $agg:expr) => {
                result_vec.push(groupby($by, $agg));
                result_vec.push(select(&$result_exprs.into_iter().map(|expr| {
                    match expr {
                        polars::prelude::Expr::Alias(_, alias) => pc(&alias),
                        _ => panic!("Invalid polars agg expression")
                    }
                }).collect::<Vec<_>>()));
            };
        }
        groupby_macro!($($group_by)?, $result_exprs);

        macro_rules! orderby_macro {
            (,) => {};
            ($order:expr, $dirs:expr) => {
                assert_eq!($order.len(), $dirs.len());
                result_vec.push(orders(&$order.to_vec(), &$dirs.to_vec()));
            };
        }
        orderby_macro!($($order_by)?, $($order_dirs)?);

        macro_rules! filter_macro {
            () => {dense_filter(cols_expr(t, &$result_columns, &accessor), tab(t), const_bool(true))};
            ($expr:expr) => { dense_filter(cols_expr(t, &$result_columns, &accessor), tab(t), $expr) };
        }
        let filter = filter_macro!($($filter)?);

        QueryExpr::new(filter, composite_result(result_vec))
    }};
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(3)),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(3_i64)),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_varchar("abc")),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(3)),
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
        dense_filter(
            cols_expr(t, &["a", "b"], &accessor),
            tab(t),
            equal(column(t, "c", &accessor), const_bigint(123)),
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
        dense_filter(
            cols_expr(t, &["a", "b"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(3)),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "b", &accessor), const_bigint(4)),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            not(equal(column(t, "b", &accessor), const_bigint(4))),
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
    let ast = query_to_provable_ast(t, "select a from sxt_tab where b <= -4", &accessor);
    let expected_ast = QueryExpr::new(
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            lte(column(t, "b", &accessor), const_bigint(-4)),
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
        "select a from sxt_tab where (b = 3) and (c <= -2)",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            and(
                equal(column(t, "b", &accessor), const_bigint(3)),
                lte(column(t, "c", &accessor), const_bigint(-2)),
            ),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            or(
                equal(column(t, "b", &accessor), const_bigint(3)),
                equal(column(t, "c", &accessor), const_bigint(-2)),
            ),
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
        "select a from sxt_tab where (b <= 3) or (not (c >= -2))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            or(
                lte(column(t, "b", &accessor), const_bigint(3)),
                not(gte(column(t, "c", &accessor), const_bigint(-2))),
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
        "select a from sxt_tab where not (((f >= 45) or (c <= -2)) and (b = 3))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            not(and(
                or(
                    gte(column(t, "f", &accessor), const_bigint(45)),
                    lte(column(t, "c", &accessor), const_bigint(-2)),
                ),
                equal(column(t, "b", &accessor), const_bigint(3)),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_int128(i128::MIN)),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_int128(i128::MAX)),
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
        "select a as b_rename from sxt_tab where b >= +4",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(
            vec![col_expr(t, "a", &accessor)],
            tab(t),
            gte(column(t, "b", &accessor), const_bigint(4)),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(3)),
        ),
        result(&[("a", "a")]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_without_any_dense_filter() {
    let t = "eth.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3_i64],
        ),
        0,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(cols_expr(t, &["a"], &accessor), tab(t), const_bool(true)),
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
        dense_filter(
            cols_expr(t, &["a", "b"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(3)),
        ),
        composite_result(vec![
            select(&[pc("b").alias("b"), pc("a").alias("a")]),
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
        dense_filter(
            cols_expr(t, &["a", "b"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(3)),
        ),
        composite_result(vec![
            select(&[pc("a").alias("a"), pc("b").alias("b")]),
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
        dense_filter(
            vec![
                col_expr(t, "name", &accessor),
                col_expr(t, "salary", &accessor),
            ],
            tab(t),
            equal(column(t, "salary", &accessor), const_bigint(5)),
        ),
        composite_result(vec![
            select(&[pc("salary").alias("s"), pc("name").alias("salary")]),
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
fn we_cannot_parse_order_by_referencing_invalid_aliased_expressions() {
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
    invalid_query_to_provable_ast(t, "select a as b from sxt_tab order by a desc", &accessor);
    invalid_query_to_provable_ast(t, "select sum(a) from sxt_tab order by a desc", &accessor);
    invalid_query_to_provable_ast(t, "select 2 * a from sxt_tab order by a desc", &accessor);
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
            dense_filter(
                vec![
                    col_expr(t, "name", &accessor),
                    col_expr(t, "salary", &accessor),
                ],
                tab(t),
                const_bool(true),
            ),
            composite_result(vec![
                select(&[
                    pc("salary").alias("s"),
                    pc("name").alias("name"),
                    pc("salary").alias("d"),
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
        dense_filter(cols_expr(t, &["a"], &accessor), tab(t), const_bool(true)),
        composite_result(vec![select(&[pc("a").alias("a")]), slice(3, 0)]),
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
        dense_filter(cols_expr(t, &["a"], &accessor), tab(t), const_bool(true)),
        composite_result(vec![select(&[pc("a").alias("a")])]),
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
        dense_filter(cols_expr(t, &["a"], &accessor), tab(t), const_bool(true)),
        composite_result(vec![select(&[pc("a").alias("a")]), slice(u64::MAX, 7)]),
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
        dense_filter(cols_expr(t, &["a"], &accessor), tab(t), const_bool(true)),
        composite_result(vec![select(&[pc("a").alias("a")]), slice(u64::MAX, -7)]),
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
        dense_filter(cols_expr(t, &["a"], &accessor), tab(t), const_bool(true)),
        composite_result(vec![select(&[pc("a").alias("a")]), slice(55, 3)]),
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
        dense_filter(
            cols_expr(t, &["a"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_bigint(-3)),
        ),
        composite_result(vec![
            select(&[pc("a").alias("a")]),
            orders(&["a"], &[Desc]),
            slice(55, 3),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

///////////////////////////
// Group By Expressions - Prover
///////////////////////////
#[ignore]
#[test]
fn we_can_do_provable_group_by() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64, 7, 2],
            "department" => [5_i64, 5, 2],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "select department, sum(salary) as total_salary, count(*) as num_employee from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(t, &["department"], &accessor),
            sums_expr(
                t,
                &["salary"],
                &["total_salary"],
                &[ColumnType::BigInt],
                &accessor,
            ),
            "num_employee",
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![select(&[
            pc("department").first().alias("department"),
            pc("salary").sum().alias("total_salary"),
            pc("department").count().alias("num_employee"),
        ])]),
    );
    assert_eq!(ast, expected_ast);
}
#[ignore]
#[test]
fn we_can_do_provable_group_by_without_sum() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [4_i64, 7, 2],
            "department" => [5_i64, 5, 2],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "select department, count(*) as num_employee from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(t, &["department"], &accessor),
            vec![],
            "num_employee",
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![select(&[
            pc("department").first().alias("department"),
            pc("department").count().alias("num_employee"),
        ])]),
    );
    assert_eq!(ast, expected_ast);
}
#[ignore]
#[test]
fn we_can_do_provable_group_by_with_two_group_by_columns() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "state" => ["CA", "CA", "NY", "NY", "CA", "CA", "NY"],
            "salary" => [4_i64, 7, 2, 3, 4, 5, 7],
            "department" => [5_i64, 5, 2, 5, 2, 5, 2],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "select state, department, sum(salary) as total_salary, count(*) as num_employee from employees group by state, department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(t, &["state", "department"], &accessor),
            sums_expr(
                t,
                &["salary"],
                &["total_salary"],
                &[ColumnType::BigInt],
                &accessor,
            ),
            "num_employee",
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![select(&[
            pc("state").first().alias("state"),
            pc("department").first().alias("department"),
            pc("salary").sum().alias("total_salary"),
            pc("department").count().alias("num_employee"),
        ])]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_do_provable_group_by_with_two_sums_and_dense_filter() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "tax" => [1_i64, 2, 1, 1, 1, 1, 2],
            "salary" => [4_i64, 7, 2, 3, 4, 5, 7],
            "department" => [5_i64, 5, 2, 5, 2, 5, 2],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "select department, sum(salary) as total_salary, sum(tax) as total_tax, count(*) as num_employee from employees where tax <= 1 group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(t, &["department"], &accessor),
            sums_expr(
                t,
                &["salary", "tax"],
                &["total_salary", "total_tax"],
                &[ColumnType::BigInt, ColumnType::BigInt],
                &accessor,
            ),
            "num_employee",
            tab(t),
            lte(column(t, "tax", &accessor), const_bigint(1)),
        ),
        composite_result(vec![select(&[
            pc("department").first().alias("department"),
            pc("salary").sum().alias("total_salary"),
            pc("tax").sum().alias("total_tax"),
            pc("department").count().alias("num_employee"),
        ])]),
    );
    assert_eq!(ast, expected_ast);
}
///////////////////////////
// Group By Expressions - Polars
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
        dense_filter(
            cols_expr(t, &["department"], &accessor),
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![
            groupby(
                vec![pc("department")],
                vec![pc("department").first().alias("department")],
            ),
            select(&[pc("department")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn group_by_expressions_are_parsed_before_an_order_by_referencing_an_aggregate_alias_result() {
    let query = query!(
        select: ["max(i) max_sal", "i0 d", "count(i0)"],
        group: ["i0", "i1"],
        order: ["max_sal"]
    );
    let expected_query = expected_query!(
        select: [
            cols = ["i", "i0", "i1"],
            exprs = [
                pc("i").max().alias("max_sal"),
                pc("i0").first().alias("d"),
                pc("i0").count().alias("__count__"),
            ]
        ],
        group: [pc("i0"), pc("i1")],
        order: [by = ["max_sal"], dirs = [Asc]]
    );
    assert_eq!(query, expected_query);
}

#[test]
fn we_cannot_parse_non_aggregated_or_non_group_by_columns_in_the_select_clause() {
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
fn alias_references_are_not_allowed_in_the_group_by() {
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
    invalid_query_to_provable_ast(
        t,
        "select salary as min_salary from employees group by min_salary",
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
fn we_can_have_aggregate_functions_without_a_group_by_clause() {
    let ast = query!(
        select: ["count(s)"],
    );
    let expected_ast = expected_query!(
        select: [
            cols = ["s"],
            exprs = [
                pc("s").count().alias("__count__"),
            ]
        ]
    );
    assert_eq!(ast, expected_ast);
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
        dense_filter(
            cols_expr(t, &["bonus", "department"], &accessor),
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![
            groupby(
                vec![pc("department")],
                vec![pc("bonus").count().alias("department")],
            ),
            select(&[pc("department")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
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
        dense_filter(
            cols_expr(t, &["bonus", "department"], &accessor),
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![
            groupby(
                vec![pc("department")],
                vec![
                    pc("department").first().alias("department"),
                    pc("bonus").count().alias("__count__"),
                    pc("department").count().alias("dep"),
                ],
            ),
            select(&[pc("department"), pc("__count__"), pc("dep")]),
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
        dense_filter(
            cols_expr(t, &["department"], &accessor),
            tab(t),
            equal(column(t, "salary", &accessor), const_bigint(4)),
        ),
        composite_result(vec![
            groupby(
                vec![pc("department")],
                vec![pc("department").count().alias("__count__")],
            ),
            select(&[pc("__count__")]),
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
        dense_filter(
            cols_expr(t, &["department"], &accessor),
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![
            groupby(
                vec![pc("department")],
                vec![
                    pc("department").first().alias("d1"),
                    pc("department").first().alias("d2"),
                ],
            ),
            select(&[pc("d1"), pc("d2")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_use_multiple_group_by_clauses_with_multiple_agg_and_non_agg_exprs() {
    let ast = query!(
        select: ["i d1", "max(i1)", "i d2", "sum(i0) sum_bonus", "count(s) count_s"],
        group: ["i", "i0", "i"]
    );
    let expected_ast = expected_query!(
        select: [
            cols = ["i", "i0", "i1", "s"],
            exprs = [
                pc("i").first().alias("d1"),
                pc("i1").max().alias("__max__"),
                pc("i").first().alias("d2"),
                pc("i0").sum().alias("sum_bonus"),
                pc("s").count().alias("count_s"),
            ]
        ],
        group: [pc("i"), pc("i0"), pc("i")]
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_simple_add_mul_sub_div_arithmetic_expressions_in_the_result_expr() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [4_i64],
            "f" => [5_i128],
            "b" => [-7_i64],
            "h" => [123_i128]
        ),
        0,
    );
    // TODO: add `a / b as a_div_b` result expr once polars properly
    // supports decimal division without panicking in production
    let ast = query_to_provable_ast(
        t,
        "select a + b, 2 * f as f2, -77 - h as col, a + f as af from employees",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(
            cols_expr(t, &["a", "b", "f", "h"], &accessor),
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![select(&[
            (pc("a") + pc("b")).alias("__expr__"),
            (lit_i64(2) * pc("f")).alias("f2"),
            ((-77_i64).to_lit() - pc("h")).alias("col"),
            (pc("a") + pc("f")).alias("af"),
            // TODO: add `a / b as a_div_b` result expr once polars properly
            // supports decimal division without panicking in production
            // (pc("a") / pc("b")).alias("a_div_b"),
        ])]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_multiple_arithmetic_expression_where_multiplication_has_precedence_in_the_result_expr(
) {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "c" => [4_i64],
            "f" => [5_i64],
            "g" => [-7_i64],
            "h" => [123_i64]
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select (2 + f) * (c + g + 2 * h), ((h - g) * 2 + c + g) * (f + 2) as d from employees",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(
            cols_expr(t, &["c", "f", "g", "h"], &accessor),
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![select(&[
            ((lit_i64(2) + pc("f")) * (pc("c") + pc("g") + lit_i64(2) * pc("h"))).alias("__expr__"),
            (((pc("h") - pc("g")) * lit_i64(2) + pc("c") + pc("g")) * (pc("f") + lit_i64(2)))
                .alias("d"),
        ])]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_arithmetic_expression_within_aggregations_in_the_result_expr() {
    let t = "sxt.employees".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "c" => [4_i64],
            "f" => [5_i64],
            "g" => [5_i64],
            "k" => [5_i64],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select c, sum(2 * f + c - -7) as d from employees group by c",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        dense_filter(
            cols_expr(t, &["c", "f"], &accessor),
            tab(t),
            const_bool(true),
        ),
        composite_result(vec![
            groupby(
                vec![pc("c")],
                vec![
                    pc("c").first().alias("c"),
                    ((2_i64.to_lit() * pc("f") + pc("c") - (-7_i64).to_lit()).sum()).alias("d"),
                ],
            ),
            select(&[pc("c"), pc("d")]),
        ]),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_need_to_reference_at_least_one_column_in_the_result_expr() {
    assert_eq!(
        query!(select: ["i", "-123 "], should_err: true),
        ConversionError::InvalidExpression(
            "at least one column must be referenced in the result expression".to_string()
        )
    );
    assert_eq!(
        query!(select: ["sum(-123)"], should_err: true),
        ConversionError::InvalidExpression(
            "at least one column must be referenced in the result expression".to_string()
        )
    );
    assert_eq!(
        query!(select: ["i + sum(-123)"], group: ["i"], should_err: true),
        ConversionError::InvalidExpression(
            "at least one column must be referenced in the result expression".to_string()
        )
    );
    assert_eq!(
        query!(select: ["sum(-123) + i"], group: ["i"], should_err: true),
        ConversionError::InvalidExpression(
            "at least one column must be referenced in the result expression".to_string()
        )
    );
}

#[test]
fn we_cannot_use_non_grouped_columns_outside_agg() {
    assert_eq!(
        query!(select: ["i"], group: ["s"], should_err: true),
        ConversionError::InvalidGroupByColumnRef("i".to_string())
    );
    assert_eq!(
        query!(select: ["sum(i)", "i"], group: ["s"], should_err: true),
        ConversionError::InvalidGroupByColumnRef("i".to_string())
    );
    assert_eq!(
        query!(select: ["min(i) + i"], group: ["s"], should_err: true),
        ConversionError::InvalidGroupByColumnRef("i".to_string())
    );
    assert_eq!(
        query!(select: ["2 * i", "min(i)"], group: ["s"], should_err: true),
        ConversionError::InvalidGroupByColumnRef("i".to_string())
    );
    assert_eq!(
        query!(select: ["2 * i", "min(i)"], should_err: true),
        ConversionError::InvalidGroupByColumnRef("i".to_string())
    );
    assert_eq!(
        query!(select: ["sum(i)", "i"], should_err: true),
        ConversionError::InvalidGroupByColumnRef("i".to_string())
    );
    assert_eq!(
        query!(select: ["max(i) + 2 * i"], should_err: true),
        ConversionError::InvalidGroupByColumnRef("i".to_string())
    );
}

#[test]
fn varchar_column_is_not_compatible_with_integer_column() {
    assert_eq!(
        query!(select: ["-123 * s"], should_err: true),
        ConversionError::DataTypeMismatch(
            ColumnType::BigInt.to_string(),
            ColumnType::VarChar.to_string()
        )
    );
    assert_eq!(
        query!(select: ["i - s"], should_err: true),
        ConversionError::DataTypeMismatch(
            ColumnType::BigInt.to_string(),
            ColumnType::VarChar.to_string()
        )
    );
    assert_eq!(
        query!(select: ["s"], filter: "'abc' = i", should_err: true),
        ConversionError::DataTypeMismatch(
            ColumnType::VarChar.to_string(),
            ColumnType::BigInt.to_string(),
        )
    );
    assert_eq!(
        query!(select: ["s"], filter: "'abc' != i", should_err: true),
        ConversionError::DataTypeMismatch(
            ColumnType::VarChar.to_string(),
            ColumnType::BigInt.to_string(),
        )
    );
}

#[test]
fn arithmetic_operations_are_not_allowed_with_varchar_column() {
    assert_eq!(
        query!(select: ["s - s1"], should_err: true),
        ConversionError::DataTypeMismatch(
            ColumnType::VarChar.to_string(),
            ColumnType::VarChar.to_string()
        )
    );
}

#[test]
fn varchar_column_is_not_allowed_within_numeric_aggregations() {
    assert_eq!(
        query!(select: ["sum(s)"], should_err: true),
        ConversionError::non_numeric_expr_in_agg("varchar", "sum")
    );
    assert_eq!(
        query!(select: ["max(s)"], should_err: true),
        ConversionError::non_numeric_expr_in_agg("varchar", "max")
    );
    assert_eq!(
        query!(select: ["min(s)"], should_err: true),
        ConversionError::non_numeric_expr_in_agg("varchar", "min")
    );
}

#[test]
fn group_by_with_bigint_column_is_valid() {
    let query = query!(select: ["i"], group: ["i"]);
    let expected_query = expected_query!(
        select: [cols = ["i"], exprs = [pc("i").first().alias("i")]], group: [pc("i")]
    );
    assert_eq!(query, expected_query);
}

#[test]
fn group_by_with_decimal_column_is_valid() {
    let query = query!(select: ["d"], group: ["d"]);
    let expected_query = expected_query!(
        select: [cols = ["d"], exprs = [pc("d").first().alias("d")]], group: [pc("d")]
    );
    assert_eq!(query, expected_query);
}

#[test]
fn group_by_with_varchar_column_is_valid() {
    let query = query!(select: ["s"], group: ["s"]);
    let expected_query = expected_query!(
        select: [cols = ["s"], exprs = [pc("s").first().alias("s")]], group: [pc("s")]
    );
    assert_eq!(query, expected_query);
}

#[test]
fn we_can_use_arithmetic_outside_agg_expressions_while_using_group_by() {
    let query = query!(
        select: ["2 * i + sum(i) - i1"],
        group: ["i", "i1"]
    );
    let expected_query = expected_query!(
        select: [
            cols = ["i", "i1"],
            exprs = [(lit_i64(2) * (pc("i").first()) + pc("i").sum() - pc("i1").first()).alias("__expr__")]
        ],
        group: [pc("i"), pc("i1")]
    );
    assert_eq!(query, expected_query);
}

#[test]
fn we_can_use_arithmetic_outside_agg_expressions_without_using_group_by() {
    let ast = query!(
        select: ["7 + max(i) as max_i", "min(i + 777 * d) * -5 as min_d"],
    );
    let expected_ast = expected_query!(
        select: [
            cols = ["d", "i"],
            exprs = [
                (lit_i64(7) + pc("i").max()).alias("max_i"),
                ((pc("i") + 777_i64.to_lit() * pc("d")).min() * (-5_i64).to_lit()).alias("min_d"),
            ]
        ]
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn count_aggregation_always_have_integer_type() {
    let ast = query!(
        select: ["7 + count(s) as cs", "count(i) * -5 as ci", "count(d)"]
    );
    let expected_ast = expected_query!(
        select: [
            cols = ["d", "i", "s"],
            exprs = [
                (7_i64.to_lit() + pc("s").count()).alias("cs"),
                (pc("i").count() * (-5_i64).to_lit()).alias("ci"),
                pc("d").count().alias("__count__"),
            ]
        ]
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn select_wildcard_is_valid_with_group_by_exprs() {
    let (t, accessor) = get_test_accessor();
    let columns = accessor.get_column_names(t);
    let ast = query!(
        select: ["*"],
        group: columns.clone()
    );
    let expected_ast = expected_query!(
        select: [
            cols = columns.clone().into_iter().sorted().collect::<Vec<_>>(),
            exprs = columns.iter().map(|c| pc(c).first().alias(c))
        ],
        group: columns.iter().map(|c| pc(c))
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn nested_aggregations_are_not_supported() {
    let supported_agg = ["max", "min", "sum", "count"];
    for perm_aggs in supported_agg.iter().permutations(2) {
        assert_eq!(
            query!(select: [format!("{}({}(i))", perm_aggs[0], perm_aggs[1])], should_err: true),
            ConversionError::InvalidExpression("nested aggregations are not supported".to_string())
        );
    }
}

#[test]
fn select_group_and_order_by_preserve_the_column_order_reference() {
    const N: usize = 4;
    let base_cols: [&str; N] = ["i", "i0", "i1", "s"]; // sorted because of `select: [cols = ... ]`
    let base_ordering = [Asc, Desc, Asc, Desc];
    for (idx, perm_cols) in base_cols.into_iter().permutations(N).unique().enumerate() {
        let group_cols = perm_cols.clone().into_iter().cycle().skip(1).take(N);
        let order_cols = perm_cols.clone().into_iter().cycle().skip(2).take(N);
        let ordering = base_ordering.into_iter().cycle().skip(idx).take(N);
        let query = query!(
            select: perm_cols,
            group: group_cols.clone(),
            order: order_cols.clone().zip(ordering.clone()).map(|(c, o)| format!("{} {}", c, o))
        );
        let expected_query = expected_query!(
            select: [
                cols = base_cols.clone(),
                exprs = perm_cols.iter().map(|c| pc(c).first().alias(c))
            ],
            group: group_cols.map(pc),
            order: [
                by = order_cols.clone().collect::<Vec<_>>(),
                dirs = ordering.clone().collect::<Vec<_>>()
            ]
        );
        assert_eq!(query, expected_query);
    }
}

/// Creates a new QueryExpr, with the given select statement and a sample schema accessor.
fn query_expr_for_test_table(sql_text: &str) -> QueryExpr<RistrettoPoint> {
    let schema_accessor = record_batch_to_accessor(
        "test.table".parse().unwrap(),
        record_batch!(
                "bigint_column" => [5_i64],
                "varchar_column" => ["example"],
                "int128_column" => [10_i128],
        ),
        0,
    );

    let default_schema = "test".parse().unwrap();

    let select_statement = SelectStatementParser::new().parse(sql_text).unwrap();

    QueryExpr::try_new(select_statement, default_schema, &schema_accessor).unwrap()
}

/// Serializes and deserializes QueryExpr with flexbuffers and asserts that it remains the same.
fn assert_query_expr_serializes_to_and_from_flex_buffers(query_expr: QueryExpr<RistrettoPoint>) {
    let serialized = flexbuffers::to_vec(&query_expr).unwrap();
    let deserialized: QueryExpr<RistrettoPoint> =
        flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(deserialized, query_expr);
}

#[test]
fn basic_query_expr_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table("select * from table");
    assert_query_expr_serializes_to_and_from_flex_buffers(query_expr);
}

#[test]
fn query_expr_with_selected_columns_can_serialize_to_and_from_flex_buffers() {
    let query_expr =
        query_expr_for_test_table("select bigint_column, varchar_column, int128_column from table");
    assert_query_expr_serializes_to_and_from_flex_buffers(query_expr);
}

#[test]
fn query_expr_with_aggregation_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table("select count(*) from table group by bigint_column");
    assert_query_expr_serializes_to_and_from_flex_buffers(query_expr);
}

#[test]
fn query_expr_with_filters_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table(
        "select * from table where bigint_column != 5 and varchar_column = 'example' or int128_column = 10",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(query_expr);
}

#[test]
fn query_expr_with_order_and_limits_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table(
        "select * from table order by int128_column desc limit 1 offset 1",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(query_expr);
}

#[test]
fn we_can_serialize_list_of_filters_from_query_expr() {
    let query_expr = query_expr_for_test_table("select * from table");

    let filter_exprs = vec![query_expr.proof_expr()];

    let serialized = flexbuffers::to_vec(&filter_exprs).unwrap();

    let deserialized: Vec<ProofPlan<RistrettoPoint>> =
        flexbuffers::from_slice(serialized.as_slice()).unwrap();
    let deserialized_as_ref: Vec<&ProofPlan<RistrettoPoint>> = deserialized.iter().collect();

    assert_eq!(filter_exprs.len(), deserialized_as_ref.len());
    assert_eq!(filter_exprs[0], deserialized_as_ref[0]);
}
