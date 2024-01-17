use super::order_by_map_i128_to_utf8;
use crate::{
    base::database::ToArrow,
    record_batch,
    sql::{
        proof::TransformExpr,
        transform::test_utility::{composite_result, orders},
    },
};
use proofs_sql::intermediate_ast::OrderByDirection::{Asc, Desc};
use rand::{distributions::uniform::SampleUniform, seq::SliceRandom, Rng};

#[test]
fn we_can_transform_a_result_using_a_single_order_by_in_ascending_direction() {
    let data = record_batch!("c" => [1_i64, -5, 2], "a" => ["a", "d", "b"]);
    let result_expr = composite_result(vec![orders(&["a"], &[Asc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [1_i64, 2, -5], "a" => ["a", "b", "d"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_a_result_using_a_single_order_by_in_ascending_direction_with_i128_data() {
    let data = record_batch!("c" => [1_i128, -5, 2], "a" => ["a", "d", "b"]);
    let result_expr = composite_result(vec![orders(&["a"], &[Asc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [1_i128, 2, -5], "a" => ["a", "b", "d"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_a_result_using_a_single_order_by_in_descending_direction() {
    let data = record_batch!("c" => [1_i64, -5, 2], "a" => ["a", "d", "b"]);
    let result_expr = composite_result(vec![orders(&["c"], &[Desc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [2_i64, 1, -5], "a" => ["b", "a", "d"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_a_result_ordering_by_the_first_column_then_the_second_column() {
    let data = record_batch!(
        "a" => [123_i64, 342, -234, 777, 123, 34],
        "d" => ["alfa", "beta", "abc", "f", "kl", "f"]
    );
    let result_expr = composite_result(vec![orders(&["a", "d"], &[Desc, Desc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!(
        "a" => [777_i64, 342, 123, 123, 34, -234],
        "d" => ["f", "beta", "kl", "alfa", "f", "abc"]
    );
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_transform_a_result_ordering_by_the_second_column_then_the_first_column() {
    let data = record_batch!(
        "a" => [123_i64, 342, -234, 777, 123, 34],
        "d" => ["alfa", "beta", "abc", "f", "kl", "f"]
    );
    let result_expr = composite_result(vec![orders(&["d", "a"], &[Desc, Asc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!(
        "a" => [123_i64, 34, 777, 342, 123, -234],
        "d" => ["kl", "f", "f", "beta", "alfa", "abc", ]
    );
    assert_eq!(data, expected_data);
}

#[test]
fn order_by_preserve_order_with_equal_elements() {
    let data = record_batch!("c" => [1_i64, -5, 1, 2], "a" => ["f", "d", "a", "b"]);
    let result_expr = composite_result(vec![orders(&["c"], &[Desc])]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [2_i64, 1, 1, -5], "a" => ["b", "f", "a", "d"]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_use_decimal_columns_inside_order_by_in_desc_order() {
    let nines = "9".repeat(38).parse::<i128>().unwrap();
    let s = [
        -1_i128, 1, -nines, -nines, 0, -2, nines, -3, nines, -1, -3, 1, -nines, 11, -nines,
    ];

    let data = record_batch!("h" => s, "j" => s);
    let result_expr = composite_result(vec![orders(&["j", "h"], &[Desc, Asc])]);
    let data = result_expr.transform_results(data);

    let mut sorted_s = s;
    sorted_s.sort_unstable();
    let reverse_sorted_s = sorted_s.into_iter().rev().collect::<Vec<_>>();

    let expected_data: arrow::record_batch::RecordBatch = record_batch!(
        "h" => reverse_sorted_s.clone(),
        "j" => reverse_sorted_s,
    );
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_use_decimal_columns_inside_order_by_in_asc_order() {
    let nines = "9".repeat(38).parse::<i128>().unwrap();
    let s = [
        -1_i128, 1, -nines, -nines, 0, -2, nines, -3, nines, -1, -3, 1, -nines, 11, -nines,
    ];

    let data = record_batch!("h" => s, "j" => s);
    let result_expr = composite_result(vec![orders(&["j", "h"], &[Asc, Desc])]);
    let data = result_expr.transform_results(data);

    let mut sorted_s = s;
    sorted_s.sort_unstable();

    let expected_data: arrow::record_batch::RecordBatch = record_batch!(
        "h" => sorted_s.clone(),
        "j" => sorted_s,
    );
    assert_eq!(data, expected_data);
}

fn validate_integer_columns_with_order_by<T>(low: T, high: T, range: Vec<T>)
where
    T: SampleUniform + Clone + Ord,
    Vec<T>: ToArrow,
{
    let mut rng = rand::thread_rng();
    let data: Vec<T> = range
        .iter()
        .map(|_| rng.gen_range(low.clone()..high.clone()))
        .chain(range.clone())
        .collect();

    let (shuffled_data, sorted_data) = {
        let mut shuffled_s = data.clone();
        shuffled_s.shuffle(&mut rng);
        let mut sorted_s = data.clone();
        sorted_s.sort_unstable();
        (shuffled_s, sorted_s)
    };

    let data = record_batch!("h" => shuffled_data);
    let expected_data = record_batch!("h" => sorted_data);
    let result_expr = composite_result(vec![orders(&["h"], &[Asc])]);
    let data = result_expr.transform_results(data);
    assert_eq!(data, expected_data);
}

#[test]
fn order_by_with_random_i64_data() {
    validate_integer_columns_with_order_by::<i64>(i64::MIN, i64::MAX, (-300000..300000).collect());
}

#[test]
fn order_by_with_random_i128_data() {
    let nines = "9".repeat(38).parse::<i128>().unwrap();
    validate_integer_columns_with_order_by::<i128>(-nines, nines + 1, (-300000..300000).collect());
}

#[test]
fn map_i128_to_utf8_not_equals_is_valid() {
    assert!(
        order_by_map_i128_to_utf8(-99999999999999999999999999999999999999)
            < order_by_map_i128_to_utf8(124)
    );
    assert!(order_by_map_i128_to_utf8(-121) < order_by_map_i128_to_utf8(122));
    assert!(order_by_map_i128_to_utf8(-123) < order_by_map_i128_to_utf8(-122));
    assert!(order_by_map_i128_to_utf8(-123) < order_by_map_i128_to_utf8(124));
    assert!(order_by_map_i128_to_utf8(-123) < order_by_map_i128_to_utf8(0));
    assert!(order_by_map_i128_to_utf8(-1) < order_by_map_i128_to_utf8(0));
    assert!(order_by_map_i128_to_utf8(0) < order_by_map_i128_to_utf8(1));
    assert!(order_by_map_i128_to_utf8(0) < order_by_map_i128_to_utf8(124));
    assert!(
        order_by_map_i128_to_utf8(124)
            < order_by_map_i128_to_utf8(99999999999999999999999999999999999999)
    );
    assert!(
        order_by_map_i128_to_utf8(-99999999999999999999999999999999999999)
            < order_by_map_i128_to_utf8(99999999999999999999999999999999999999)
    );
}

fn validate_order_by_map_i128_to_utf8_with_array(s: Vec<i128>) {
    let mut sorted_s: Vec<_> = s.clone();
    sorted_s.sort_unstable();

    let mut utf8_sorted_s: Vec<_> = s.iter().map(|v| order_by_map_i128_to_utf8(*v)).collect();
    utf8_sorted_s.sort_unstable();

    // ordering is preserved
    assert_eq!(
        sorted_s
            .iter()
            .map(|&v| order_by_map_i128_to_utf8(v))
            .collect::<Vec<_>>(),
        utf8_sorted_s
    );

    // no collision happens
    assert_eq!(
        sorted_s
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        utf8_sorted_s
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
    );
}

#[test]
fn order_by_with_consecutive_range_preserves_ordering() {
    validate_order_by_map_i128_to_utf8_with_array((-300000..300000).collect());
}

#[test]
fn order_by_with_random_data_preserves_ordering() {
    let mut rng = rand::thread_rng();
    let nines = "9".repeat(38).parse::<i128>().unwrap();
    validate_order_by_map_i128_to_utf8_with_array(
        (-300000..300000)
            .map(|_| rng.gen_range(-nines..nines + 1))
            .collect(),
    );
}

#[test]
#[should_panic]
fn order_by_panics_with_min_out_of_range_value() {
    order_by_map_i128_to_utf8(i128::MIN);
}

#[test]
fn order_by_do_not_panic_with_max_out_of_range_value() {
    order_by_map_i128_to_utf8(i128::MAX);
}
