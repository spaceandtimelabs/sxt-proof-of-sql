use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::{test_utility::select, ResultExpr};

#[test]
fn we_can_filter_out_record_batch_columns() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[("a", "a2")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a2" => ["d", "a", "f", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_reorder_the_record_batch_columns_without_changing_their_names() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[("a", "a"), ("c", "c")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a" => ["d", "a", "f", "b"], "c" => [-5, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_record_batch_columns_to_different_names() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[("a", "b_test"), ("c", "col_c_test")]));
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("b_test" => ["d", "a", "f", "b"], "col_c_test" => [-5, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_record_batch_columns_to_new_columns() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[
        ("c", "abc"),
        ("a", "b_test"),
        ("a", "d2"),
        ("c", "c"),
    ]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("abc" => [-5, 1, -56, 2], "b_test" => ["d", "a", "f", "b"], "d2" => ["d", "a", "f", "b"], "c" => [-5, 1, -56, 2]);
    assert_eq!(data, expected_data);
}
