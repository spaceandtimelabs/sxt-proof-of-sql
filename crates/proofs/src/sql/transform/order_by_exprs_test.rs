use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::test_utility::{composite_result, orders};
use proofs_sql::intermediate_ast::OrderByDirection::{Asc, Desc};

#[test]
fn we_can_transform_a_result_using_a_single_order_by_in_ascending_direction() {
    let data = record_batch!("c" => [1, -5, 2], "a" => ["a", "d", "b"]);
    let result_expr = composite_result(vec![orders(&["a"], &[Asc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [1, 2, -5], "a" => ["a", "b", "d"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_a_result_using_a_single_order_by_in_descending_direction() {
    let data = record_batch!("c" => [1, -5, 2], "a" => ["a", "d", "b"]);
    let result_expr = composite_result(vec![orders(&["c"], &[Desc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [2, 1, -5], "a" => ["b", "a", "d"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_a_result_ordering_by_the_first_column_then_the_second_column() {
    let data = record_batch!(
        "a" => [123, 342, -234, 777, 123, 34],
        "d" => ["alfa", "beta", "abc", "f", "kl", "f"]
    );
    let result_expr = composite_result(vec![orders(&["a", "d"], &[Desc, Desc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!(
        "a" => [777, 342, 123, 123, 34, -234],
        "d" => ["f", "beta", "kl", "alfa", "f", "abc"]
    );
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_a_result_ordering_by_the_second_column_then_the_first_column() {
    let data = record_batch!(
        "a" => [123, 342, -234, 777, 123, 34],
        "d" => ["alfa", "beta", "abc", "f", "kl", "f"]
    );
    let result_expr = composite_result(vec![orders(&["d", "a"], &[Desc, Asc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!(
        "a" => [123, 34, 777, 342, 123, -234],
        "d" => ["kl", "f", "f", "beta", "alfa", "abc", ]
    );
    assert_eq!(data, expected_data);
}

#[test]
fn order_by_preserve_order_with_equal_elements() {
    let data = record_batch!("c" => [1, -5, 1, 2], "a" => ["f", "d", "a", "b"]);
    let result_expr = composite_result(vec![orders(&["c"], &[Desc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [2, 1, 1, -5], "a" => ["b", "f", "a", "d"]);
    assert_eq!(data, expected_data);
}
