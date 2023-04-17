use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::{test_utility::schema, ResultExpr};

#[test]
fn an_empty_result_expr_does_not_change_the_record_batch() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let expected_data = data.clone();
    let result_expr = ResultExpr::default();
    let data = result_expr.transform_results(data);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_filter_out_record_batch_columns() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new_with_result_schema(schema(&[("a", "a2")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a2" => ["d", "a", "f", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_reorder_the_record_batch_columns_without_changing_their_names() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new_with_result_schema(schema(&[("a", "a"), ("c", "c")]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a" => ["d", "a", "f", "b"], "c" => [-5, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_project_the_record_batch_columns_to_different_names() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr =
        ResultExpr::new_with_result_schema(schema(&[("a", "b_test"), ("c", "col_c_test")]));
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("b_test" => ["d", "a", "f", "b"], "col_c_test" => [-5, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_project_the_record_batch_columns_to_new_columns() {
    let data = record_batch!("c" => [-5, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new_with_result_schema(schema(&[
        ("c", "abc"),
        ("a", "b_test"),
        ("a", "d2"),
        ("c", "c"),
    ]));
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("abc" => [-5, 1, -56, 2], "b_test" => ["d", "a", "f", "b"], "d2" => ["d", "a", "f", "b"], "c" => [-5, 1, -56, 2]);
    assert_eq!(data, expected_data);
}
