use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::{test_utility::select, ResultExpr};

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
