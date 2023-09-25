use super::group_by_map_i128_to_utf8;
use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::test_utility::{composite_result, groupby};

use arrow::record_batch::RecordBatch;
use polars::prelude::{col, lit};
use rand::Rng;

#[test]
fn we_can_transform_batch_using_group_by_with_a_varchar_column() {
    let data = record_batch!("a" => ["a", "d", "a", "b"], "b" => [1_i64, -5, 1, 2], "c" => [-1_i128, 0, -1, 3]);
    let by_exprs = vec![col("a")];
    let agg_exprs = vec![col("a").first(), col("b").first(), col("c").first()];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("a" => ["a", "d", "b"], "b" => [1_i64, -5, 2],"c" => [-1_i128, 0, 3]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_group_by_with_a_i64_column() {
    let data = record_batch!("a" => ["a", "d", "a", "b"], "b" => [1_i64, -5, 1, 2], "c" => [-1_i128, 0, -1, 3]);
    let by_exprs = vec![col("b")];
    let agg_exprs = vec![col("a").first(), col("b").first(), col("c").first()];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("a" => ["a", "d", "b"], "b" => [1_i64, -5, 2],"c" => [-1_i128, 0, 3]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_group_by_with_a_i128_column() {
    let data = record_batch!("a" => ["a", "d", "a", "b"], "b" => [1_i64, -5, 1, 2], "c" => [-1_i128, 0, -1, 3]);
    let by_exprs = vec![col("c")];
    let agg_exprs = vec![col("a").first(), col("b").first(), col("c").first()];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("a" => ["a", "d", "b"], "b" => [1_i64, -5, 2],"c" => [-1_i128, 0, 3]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_the_same_group_bys_with_the_same_alias() {
    let data = record_batch!("c" => [1_i64, -5, 7, 7, 2], "a" => ["a", "d", "a", "a", "b"]);
    let by_exprs = vec![col("a"), col("a")];
    let result_expr = composite_result(vec![groupby(by_exprs, vec![col("c").sum()])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [15_i64, -5, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_different_group_bys_with_different_aliases() {
    let data = record_batch!("c" => [1_i64, -5, 7, 7, 2], "a" => ["a", "d", "a", "a", "b"]);
    let by_exprs = vec![col("a"), col("c")];
    let result_expr = composite_result(vec![groupby(
        by_exprs,
        vec![col("a").first(), col("c").first()],
    )]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a" => ["a", "d", "a", "b"], "c" => [1_i64, -5, 7, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_max_aggregation() {
    let data = record_batch!("b" => [1_i64, -5, -3, 7, 2], "c" => [1_i128, -5, -3, 1, -3], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![col("a"), col("c")];
    let agg_exprs = vec![(col("b") + col("c")).max().alias("bc")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("bc" => [8_i128, -10, -1]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_min_aggregation() {
    let data = record_batch!("b" => [1_i64, -5, -3, 7, 2], "c" => [1_i128, -5, -3, 1, -3], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![col("a"), col("c")];
    let agg_exprs = vec![(col("b") * col("c")).min().alias("bc")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("bc" => [1_i128, 25, -6]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_sum_aggregation() {
    let data = record_batch!("b" => [1_i64, -5, -3, 7, 2], "c" => [1_i128, -5, -3, 1, -3], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![col("a"), col("c")];
    let agg_exprs = vec![(col("b") - col("c")).sum().alias("bc")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("bc" => [6_i128, 0, 5]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn sum_aggregation_can_overflow() {
    let data = record_batch!("c" => [i64::MAX, 1], "a" => ["a", "a"]);
    let by_exprs = vec![col("a")];
    let agg_exprs = vec![col("c").sum()];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    result_expr.transform_results(data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_count_aggregation() {
    let data = record_batch!("b" => [1_i64, -5, -3, 7, 2], "c" => [1_i128, -5, -3, 1, -3], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![col("a"), col("c")];
    let agg_exprs = vec![
        col("a").first(),
        (lit(-53) * col("b") - lit(45) * col("c") + lit(103))
            .count()
            .alias("bc"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a" => ["a", "d", "b"], "bc" => [2_i64, 1, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_first_aggregation() {
    let data = record_batch!("a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![col("a")];
    let agg_exprs = vec![
        col("a").first().alias("a_col"),
        col("a").first().alias("a2_col"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_col" => ["a", "d", "b"], "a2_col" => ["a", "d", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_group_by_with_the_same_name_as_the_aggregation_expression() {
    let data =
        record_batch!("c" => [1_i64, -5, -3, 7, 2, 1], "a" => ["a", "d", "b", "a", "b", "f"]);
    let by_exprs = vec![col("c").alias("c")];
    let agg_exprs = vec![col("c").min().alias("c")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [1_i64, -5, -3, 7, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_min_aggregation_with_non_numeric_columns() {
    let data =
        record_batch!("c" => [1_i64, -5, -3, 7, 2, 1], "a" => ["abd", "d", "b", "a", "b", "abc"]);
    let by_exprs = vec![col("c")];
    let agg_exprs = vec![col("c").first(), col("a").min().alias("a_min")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("c" => [1_i64, -5, -3, 7, 2], "a_min" => ["abc", "d", "b", "a", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_max_aggregation_with_non_numeric_columns() {
    let data =
        record_batch!("c" => [1_i64, -5, -3, 7, -5, 1], "a" => ["abd", "a", "b", "a", "aa", "abc"]);
    let by_exprs = vec![col("c")];
    let agg_exprs = vec![col("c").first(), col("a").max().alias("a_max")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("c" => [1_i64, -5, -3, 7], "a_max" => ["abd", "aa", "b", "a"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_count_aggregation_with_non_numeric_columns() {
    let data =
        record_batch!("c" => [1_i64, -5, -3, 7, 2, 1], "a" => ["a", "d", "b", "a", "b", "f"]);
    let by_exprs = vec![col("c")];
    let agg_exprs = vec![col("a").count().alias("a_count")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_count" => [2_i64, 1, 1, 1, 1]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_multiple_aggregations() {
    let data = record_batch!("c" => [1_i128, -5, -3, 7, 2], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![col("a")];
    let agg_exprs = vec![
        col("c").max().alias("c_max"),
        col("a").first(),
        col("c").min().alias("c_min"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c_max" => [7_i128, -5, 2], "a" => ["a", "d", "b"], "c_min" => [1_i128, -5, -3]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_multiple_group_bys_with_multiple_aggregations() {
    let data = record_batch!("c" => [1_i64, -5, -3, 7, -3], "a" => ["a", "d", "b", "a", "b"], "d" => [523_i64, -25, 343, -7, 435]);
    let by_exprs = vec![col("a"), col("c")];
    let agg_exprs = vec![
        col("a").first().alias("a_group"),
        col("d").max().alias("d_max"),
        col("c").count().alias("c_count"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_group" => ["a", "d", "b", "a"], "d_max" => [523_i64, -25, 435, -7], "c_count" => [1_i64, 1, 2, 1]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_different_aliases_associated_with_the_same_group_by_column() {
    let data = record_batch!("a" => ["a", "b"], "d" => [523_i64, -25]);
    let by_exprs = vec![col("a").alias("a1"), col("a").alias("a2")];
    let result_expr = composite_result(vec![groupby(
        by_exprs,
        vec![col("a").alias("a1").first(), col("a").alias("a2").first()],
    )]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a1" => ["a", "b"], "a2" => ["a", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn we_cannot_transform_batch_using_an_empty_group_by_expression() {
    let agg_exprs = vec![col("b").max()];
    composite_result(vec![groupby(vec![], agg_exprs)]);
}

#[test]
#[should_panic]
fn we_cannot_transform_batch_using_an_empty_agg_expression() {
    let group_bys = vec![col("b")];
    composite_result(vec![groupby(group_bys, vec![])]);
}

#[test]
fn we_can_use_decimal_column_and_agg_inside_the_group_by_exprs() {
    let data = record_batch!("d" => [523_i128, -25, 523, 25]);
    let result_expr = composite_result(vec![groupby(
        vec![col("d").max()],
        vec![col("d").sum().alias("d_sum")],
    )]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("d_sum" => [2 * 523_i128]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_arithmetic_expressions_in_the_aggregation() {
    let data = record_batch!(
        "c" => [1_i64, -5, -3, 7, -3],
        "a" => ["a", "d", "b", "a", "b"],
        "d" => [523_i64, -25, 343, -7, 435]
    );
    let by_exprs = vec![col("a").alias("a_group")];
    let agg_exprs = vec![(col("d") * col("c")).sum().alias("cd_sum")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("cd_sum" => [474_i64, 125, -2334]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_arithmetic_outside_the_aggregation_exprs() {
    let data = record_batch!(
        "c" => [1_i128, -5, -3, -5, 7, -3],
        "d" => [-1_i64, -5, 0, -5, 7, 7]
    );
    let by_exprs = vec![col("d").alias("#$d"), col("c").alias("#$cd")];
    let agg_exprs = vec![
        (col("c").first() + col("d").first()).alias("sum_cd1"),
        (col("c") + col("d")).sum().alias("sum_cd2"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!(
        "sum_cd1" => [0_i128, -10, -3, 14, 4],
        "sum_cd2" => [0_i128, -20, -3, 14, 4],
    );
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_arithmetic_expressions_inside_the_group_by_exprs() {
    let data = record_batch!(
        "a" => [1_i64, 8, 13, 7, 16],
        "b" => ["aaaaa", "aaaa", "aaa", "aa", "a"],
        "c" => [9_i128, 2, -3, 4, -5]
    );
    let by_exprs = vec![col("c") + col("a")];
    let agg_exprs = vec![
        (col("c") + col("a")).first().alias("sum_ca"),
        col("b").count(),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("sum_ca" => [10_i128, 11], "b" => [3_i64, 2_i64]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_use_decimal_columns_inside_group_by() {
    let nines: i128 = "9".repeat(38).parse::<i128>().unwrap();
    let data = record_batch!(
        "h" => [-1_i128, 1, -nines, 0, -2, nines, -3, -1, -3, 1, 11],
        "j" => [0_i64, 12, 5, 3, -2, -1, 4, 4, 100, 0, 31],
    );
    let by_exprs = vec![col("h")];
    let agg_exprs = vec![(col("j") + col("h")).sum().alias("h_sum")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!(
        "h_sum" => [2_i128, 14, -nines + 5, 3, -2 - 2, nines - 1, -6 + 100 + 4, 11 + 31],
    );
    assert_eq!(data, expected_data);
}

#[test]
fn transforming_a_batch_of_size_zero_with_min_max_agg_and_decimal_column_is_fine() {
    let data = record_batch!("h" => [-1_i128], "i" => [2_i128], "j" => [2_i128], "k" => [2_i64]);
    let empty_batch = RecordBatch::new_empty(data.schema().clone());
    let by_exprs = vec![col("h")];
    let agg_exprs = vec![
        col("h").max().alias("h"),
        col("i").min().alias("i"),
        col("j").sum().alias("j"),
        col("k").count().alias("k"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(empty_batch.clone());
    let expected_data = empty_batch;
    assert_eq!(data, expected_data);
}

#[test]
fn transforming_a_batch_of_size_one_with_min_max_agg_and_decimal_column_is_fine() {
    let input_data =
        record_batch!("h" => [-1_i128], "i" => [2_i128], "j" => [2_i128], "k" => [2_i128]);
    let by_exprs = vec![col("h")];
    let agg_exprs = vec![
        col("h").max().alias("h"),
        col("i").min().alias("i"),
        col("j").sum().alias("j"),
        col("k").count().alias("k"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(input_data.clone());
    let expected_data =
        record_batch!("h" => [-1_i128], "i" => [2_i128], "j" => [2_i128], "k" => [1_i64]);
    assert_eq!(data, expected_data);
}

fn validate_group_by_map_i128_to_utf8(s: Vec<i128>) {
    let expected_len = s.len();

    // no collision happens
    assert_eq!(
        expected_len,
        s.iter().collect::<std::collections::HashSet<_>>().len()
    );

    assert_eq!(
        expected_len,
        s.into_iter()
            .map(group_by_map_i128_to_utf8)
            .collect::<std::collections::HashSet<_>>()
            .len(),
    );
}

#[test]
fn group_by_with_consecutive_range_doesnt_have_collisions() {
    validate_group_by_map_i128_to_utf8((-300000..300000).collect());
}

#[test]
fn group_by_with_random_data_doesnt_have_collisions() {
    let mut rng = rand::thread_rng();
    let nines = "9".repeat(38).parse::<i128>().unwrap();
    validate_group_by_map_i128_to_utf8(
        (-300000..300000)
            .map(|_| rng.gen_range(-nines, nines + 1))
            .collect(),
    );
}
