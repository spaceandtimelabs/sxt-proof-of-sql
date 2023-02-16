use crate::base::database::data_frame_to_record_batch;
use crate::base::database::{
    make_random_test_accessor_data, RandomTestAccessorDescriptor, TestAccessor,
};
use crate::base::scalar::ToScalar;
use crate::sql::ast::test_expr::TestExpr;
use crate::sql::ast::test_utility::equal;

use polars::prelude::Expr;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

fn create_test_expr<T: ToScalar + Copy + Into<Expr>>(
    table_ref: &str,
    results: &[&str],
    filter_col: &str,
    filter_val: T,
    data: DataFrame,
    offset: usize,
) -> TestExpr {
    let mut accessor = TestAccessor::new();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let equals_expr = equal(t, filter_col, filter_val, &accessor);
    let df_filter = polars::prelude::col(filter_col).eq(filter_val);
    TestExpr::new(t, results, equals_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_an_equality_query_with_no_rows() {
    let data = df!(
        "a" => Vec::<i64>::new(),
        "b" => Vec::<i64>::new()
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => Vec::<i64>::new()
        )
        .unwrap(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_selected_row() {
    let data = df!(
        "a" => [123],
        "b" => [0]
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => [123]
        )
        .unwrap(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_non_selected_row() {
    let data = df!(
        "a" => [123],
        "b" => [55]
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => Vec::<i64>::new()
        )
        .unwrap(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_multiple_rows() {
    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 5, 0, 5]
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => [1, 3]
        )
        .unwrap(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_nonzero_comparison() {
    let data = df!(
        "a" => [1, 2, 3, 4, 5],
        "b" => [123, 5, 123, 5, 0],
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], "b", 123_u64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => [1, 3]
        )
        .unwrap(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn verify_fails_if_data_between_prover_and_verifier_differ() {
    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 5, 0, 5],
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], "b", 0_u64, data, 0);

    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 2, 0, 5],
    )
    .unwrap();
    let tampered_test_expr = create_test_expr("sxt.t", &["a"], "b", 0_u64, data, 0);

    let res = test_expr.create_verifiable_result();
    assert!(res
        .verify(&test_expr.ast, &tampered_test_expr.accessor)
        .is_err());
}

fn we_can_query_random_tables_with_multiple_selected_rows_and_given_offset(offset: usize) {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = ["aa", "ab", "b"];
    for _ in 0..10 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_expr("sxt.t", &["aa", "ab"], "ab", filter_val, data, offset);
        let res = test_expr.verify_expr();
        let expected_res = test_expr.query_table();
        assert_eq!(res, expected_res);
    }
}

#[test]
fn we_can_query_random_tables_with_a_zero_offset() {
    we_can_query_random_tables_with_multiple_selected_rows_and_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    we_can_query_random_tables_with_multiple_selected_rows_and_given_offset(121);
}
