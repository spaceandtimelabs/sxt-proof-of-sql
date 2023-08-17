use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::test_utility::*;

#[test]
fn we_can_transform_batch_using_simple_group_by_with_an_alias() {
    let data = record_batch!("c" => [1, -5, 7, 2], "a" => ["a", "d", "a", "b"]);
    let by_exprs = vec![("a", Some("a_col"))];
    let result_expr = composite_result(vec![groupby(by_exprs, vec![])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_col" => ["a", "d", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn we_cannot_transform_batch_using_a_single_group_by_without_an_alias_or_aggregation_columns() {
    composite_result(vec![groupby(vec![("a", None)], vec![])]);
}

#[test]
fn we_can_transform_batch_using_the_same_group_bys_with_different_aliases() {
    let data = record_batch!("c" => [1, -5, 7, 7, 2], "a" => ["a", "d", "a", "a", "b"]);
    let by_exprs = vec![("a", None), ("a", Some("a_col"))];
    let result_expr = composite_result(vec![groupby(by_exprs, vec![])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("#$a" => ["a", "d", "b"], "a_col" => ["a", "d", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_different_group_bys_with_different_aliases() {
    let data = record_batch!("c" => [1, -5, 7, 7, 2], "a" => ["a", "d", "a", "a", "b"]);
    let by_exprs = vec![("a", Some("a_col")), ("c", Some("c_col"))];
    let result_expr = composite_result(vec![groupby(by_exprs, vec![])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_col" => ["a", "d", "a", "b"], "c_col" => [1, -5, 7, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_max_aggregation() {
    let data = record_batch!("c" => [1, -5, -3, 7, 2], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![("a", None)];
    let agg_exprs = vec![agg_expr("max", "c", "c")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("#$a" => ["a", "d", "b"], "c" => [7, -5, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_min_aggregation() {
    let data = record_batch!("c" => [1, -5, -3, 7, 2], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![("a", None)];
    let agg_exprs = vec![agg_expr("min", "c", "c")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("#$a" => ["a", "d", "b"], "c" => [1, -5, -3]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_sum_aggregation() {
    let data = record_batch!("c" => [1, -5, -3, 7, 2], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![("a", None)];
    let agg_exprs = vec![agg_expr("sum", "c", "c")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("#$a" => ["a", "d", "b"], "c" => [8, -5, -1]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn sum_aggregation_can_overflow() {
    let data = record_batch!("c" => [i64::MAX, 1], "a" => ["a", "a"]);
    let by_exprs = vec![("a", None)];
    let agg_exprs = vec![agg_expr("sum", "c", "c")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    result_expr.transform_results(data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_count_aggregation() {
    let data = record_batch!("c" => [1, -5, -3, 7, 2], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![("a", Some("a_col"))];
    let agg_exprs = vec![agg_expr("count", "c", "c_col")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_col" => ["a", "d", "b"], "c_col" => [2, 1, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_group_by_with_the_same_name_as_the_aggregation_expression() {
    let data = record_batch!("c" => [1, -5, -3, 7, 2, 1], "a" => ["a", "d", "b", "a", "b", "f"]);
    let by_exprs = vec![("c", None)];
    let agg_exprs = vec![agg_expr("min", "c", "c")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("#$c" => [1, -5, -3, 7, 2], "c" => [1, -5, -3, 7, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_min_aggregation_with_non_numeric_columns() {
    let data =
        record_batch!("c" => [1, -5, -3, 7, 2, 1], "a" => ["abd", "d", "b", "a", "b", "abc"]);
    let by_exprs = vec![("c", None)];
    let agg_exprs = vec![agg_expr("min", "a", "a_min")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("#$c" => [1, -5, -3, 7, 2], "a_min" => ["abc", "d", "b", "a", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_max_aggregation_with_non_numeric_columns() {
    let data =
        record_batch!("c" => [1, -5, -3, 7, -5, 1], "a" => ["abd", "a", "b", "a", "aa", "abc"]);
    let by_exprs = vec![("c", None)];
    let agg_exprs = vec![agg_expr("max", "a", "a_max")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("#$c" => [1, -5, -3, 7], "a_max" => ["abd", "aa", "b", "a"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_count_aggregation_with_non_numeric_columns() {
    let data = record_batch!("c" => [1, -5, -3, 7, 2, 1], "a" => ["a", "d", "b", "a", "b", "f"]);
    let by_exprs = vec![("c", None)];
    let agg_exprs = vec![agg_expr("count", "a", "a_count")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("#$c" => [1, -5, -3, 7, 2], "a_count" => [2, 1, 1, 1, 1]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_simple_group_by_with_multiple_aggregations() {
    let data = record_batch!("c" => [1, -5, -3, 7, 2], "a" => ["a", "d", "b", "a", "b"]);
    let by_exprs = vec![("a", None)];
    let agg_exprs = vec![agg_expr("max", "c", "c_max"), agg_expr("min", "c", "c_min")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data =
        record_batch!("#$a" => ["a", "d", "b"], "c_max" => [7, -5, 2], "c_min" => [1, -5, -3]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_multiple_group_by_with_single_aggregation() {
    let data = record_batch!("c" => [1, -5, -3, 7, -3], "a" => ["a", "d", "b", "a", "b"], "d" => [523, -25, 343, -7, 435]);
    let by_exprs = vec![("a", Some("a_group")), ("c", None)];
    let agg_exprs = vec![agg_expr("max", "d", "d_max")];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_group" => ["a", "d", "b", "a"], "#$c" => [1, -5, -3, 7], "d_max" => [523, -25, 435, -7]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_batch_using_multiple_group_by_with_multiple_aggregations() {
    let data = record_batch!("c" => [1, -5, -3, 7, -3], "a" => ["a", "d", "b", "a", "b"], "d" => [523, -25, 343, -7, 435]);
    let by_exprs = vec![("a", Some("a_group")), ("c", None)];
    let agg_exprs = vec![
        agg_expr("max", "d", "d_max"),
        agg_expr("count", "c", "c_count"),
    ];
    let result_expr = composite_result(vec![groupby(by_exprs, agg_exprs)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a_group" => ["a", "d", "b", "a"], "#$c" => [1, -5, -3, 7], "d_max" => [523, -25, 435, -7], "c_count" => [1, 1, 2, 1]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn we_cannot_transform_batch_using_the_same_non_aliased_group_by_multiple_times() {
    let by_exprs = vec![("a", None), ("a", None)];
    composite_result(vec![groupby(by_exprs, vec![])]);
}

#[test]
#[should_panic]
fn we_cannot_transform_batch_using_the_same_aliased_group_by_multiple_times() {
    let by_exprs = vec![("a", Some("a2")), ("a", Some("a2"))];
    composite_result(vec![groupby(by_exprs, vec![])]);
}

#[test]
fn we_can_transform_batch_using_different_aliases_associated_with_the_same_group_by_column() {
    let data = record_batch!("a" => ["a", "b"], "d" => [523, -25]);
    let by_exprs = vec![("a", Some("a1")), ("a", Some("a2"))];
    let result_expr = composite_result(vec![groupby(by_exprs, vec![])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("a1" => ["a", "b"], "a2" => ["a", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn we_cannot_transform_batch_using_the_same_aliased_group_by_in_the_aggregation_expression() {
    let by_exprs = vec![("a", Some("a2"))];
    let agg_exprs = vec![agg_expr("max", "b", "a2")];
    composite_result(vec![groupby(by_exprs, agg_exprs)]);
}

#[test]
#[should_panic]
fn we_cannot_transform_batch_using_an_empty_group_by_expression() {
    let agg_exprs = vec![agg_expr("max", "b", "a2")];
    composite_result(vec![groupby(vec![], agg_exprs)]);
}
