use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::{test_utility::select, ResultExpr};

use arrow::record_batch::RecordBatch;
use polars::prelude::{col, lit};

#[test]
fn we_can_filter_out_record_batch_columns() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[col("a").alias("a2")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a2" => ["d", "a", "f", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_filter_out_record_batch_columns_with_i128_data() {
    let data = record_batch!("c" => [-5_i128, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[col("a").alias("a2")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a2" => ["d", "a", "f", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn result_expr_panics_with_batches_containing_duplicate_columns() {
    let data = record_batch!("a" => [-5_i64, 1, -56, 2], "a" => [-5_i64, 1, -56, 2]);
    let result_expr = ResultExpr::new(select(&[col("a").alias("a2"), col("a").alias("a3")]));
    result_expr.transform_results(data);
}

#[test]
fn we_can_reorder_the_record_batch_columns_without_changing_their_names() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[col("a").alias("a"), col("c").alias("c")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a" => ["d", "a", "f", "b"], "c" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_record_batch_columns_to_different_names() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[
        col("a").alias("b_test"),
        col("c").alias("col_c_test"),
    ]));
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("b_test" => ["d", "a", "f", "b"], "col_c_test" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_record_batch_columns_to_new_columns() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[
        col("c").alias("abc"),
        col("a").alias("b_test"),
        col("a").alias("d2"),
        col("c").alias("c"),
    ]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("abc" => [-5_i64, 1, -56, 2], "b_test" => ["d", "a", "f", "b"], "d2" => ["d", "a", "f", "b"], "c" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_use_arithmetic_expressions() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[(lit(2) + col("c") * lit(3)).alias("c2")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c2" => [-13_i64, 5, -166, 8]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_use_agg_with_select_expression() {
    let data = record_batch!(
        "c" => [1_i64, -5, -3, 7, -3],
        "a" => [1_i64, 2, 3, 1, 3],
        "d" => [523_i128, -25, 343, -7, 435],
        "h" => [-1_i128, -2, -3, -1, -3],
        "y" => ["a", "b", "c", "d", "e"]
    );
    let result_expr = ResultExpr::new(select(&[
        col("c").sum().alias("c_sum"),
        col("a").max().alias("a_max"),
        col("d").min().alias("d_min"),
        col("h").count().alias("h_count"),
        (col("c").sum() * col("a").max() - col("d").min() + col("h").count() * lit(2) - lit(733))
            .alias("expr"),
    ]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!(
        "c_sum" => [-3_i64],
        "a_max" => [3_i64],
        "d_min" => [-25_i128],
        "h_count" => [5_i64],
        "expr" => [-707_i128],
    );
    assert_eq!(data, expected_data);
}

#[test]
fn using_count_with_an_empty_batch_will_return_zero() {
    let data = record_batch!("i" => [-5_i64], "d" => [3_i128], "s" => ["a"]);
    let empty_data = RecordBatch::new_empty(data.schema());
    let result_expr = ResultExpr::new(select(&[
        col("i").count(),
        col("d").count(),
        col("s").count(),
    ]));
    let data = result_expr.transform_results(empty_data);
    let expected_data = record_batch!("i" => [0_i64], "d" => [0_i64], "s" => [0_i64]);
    assert_eq!(data, expected_data);
}

#[test]
fn using_sum_with_an_empty_batch_will_return_zero() {
    let data = record_batch!("i" => [-5_i64], "d" => [3_i128]);
    let empty_data = RecordBatch::new_empty(data.schema());
    let result_expr = ResultExpr::new(select(&[col("i").sum(), col("d").sum()]));
    let data = result_expr.transform_results(empty_data);
    let expected_data = record_batch!("i" => [0_i64], "d" => [0_i128]);
    assert_eq!(data, expected_data);
}

#[test]
fn using_min_with_an_empty_batch_will_return_empty_even_with_count_or_sum_in_the_result() {
    let data = record_batch!("i" => [-5_i64], "d" => [3_i128], "i1" => [3_i64]);
    let empty_data = RecordBatch::new_empty(data.schema());
    let result_expr = ResultExpr::new(select(&[col("i").count(), col("d").sum(), col("i1").min()]));
    let data = result_expr.transform_results(empty_data.clone());
    let expected_data = empty_data;
    assert_eq!(data, expected_data);
}

#[test]
fn using_max_with_an_empty_batch_will_return_empty_even_with_count_or_sum_in_the_result() {
    let data = record_batch!("i" => [-5_i64], "d" => [3_i128], "i1" => [3_i64]);
    let empty_data = RecordBatch::new_empty(data.schema());
    let result_expr = ResultExpr::new(select(&[col("i").count(), col("d").sum(), col("i1").max()]));
    let data = result_expr.transform_results(empty_data.clone());
    let expected_data = empty_data;
    assert_eq!(data, expected_data);
}
