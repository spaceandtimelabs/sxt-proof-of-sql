use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::{test_utility::select, ResultExpr};

#[test]
fn we_can_filter_out_record_batch_columns() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new_with_transformation(select(&[("a", "a2")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a2" => ["d", "a", "f", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_filter_out_record_batch_columns_with_i128_data() {
    let data = record_batch!("c" => [-5_i128, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new_with_transformation(select(&[("a", "a2")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a2" => ["d", "a", "f", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn result_expr_panics_with_batches_containing_duplicate_columns() {
    let data = record_batch!("a" => [-5_i64, 1, -56, 2], "a" => [-5_i64, 1, -56, 2]);
    let result_expr = ResultExpr::new_with_transformation(select(&[("a", "a2"), ("a", "a3")]));
    result_expr.transform_results(data);
}

#[test]
#[should_panic]
fn we_cant_construct_select_expressions_with_duplicate_aliases() {
    select(&[("a", "a2"), ("a", "a2")]);
}

#[test]
fn we_can_reorder_the_record_batch_columns_without_changing_their_names() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new_with_transformation(select(&[("a", "a"), ("c", "c")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a" => ["d", "a", "f", "b"], "c" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_record_batch_columns_to_different_names() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr =
        ResultExpr::new_with_transformation(select(&[("a", "b_test"), ("c", "col_c_test")]));
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("b_test" => ["d", "a", "f", "b"], "col_c_test" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_record_batch_columns_to_new_columns() {
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new_with_transformation(select(&[
        ("c", "abc"),
        ("a", "b_test"),
        ("a", "d2"),
        ("c", "c"),
    ]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("abc" => [-5_i64, 1, -56, 2], "b_test" => ["d", "a", "f", "b"], "d2" => ["d", "a", "f", "b"], "c" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}
