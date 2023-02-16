use crate::base::database::data_frame_to_record_batch;
use crate::base::database::{
    make_random_test_accessor_data, RandomTestAccessorDescriptor, TestAccessor,
};
use crate::base::scalar::ToScalar;
use crate::sql::ast::test_expr::TestExpr;
use crate::sql::ast::test_utility::{equal, or};

use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

fn create_test_expr<T1: ToScalar + Copy + Into<Expr>, T2: ToScalar + Copy + Into<Expr>>(
    table_ref: &str,
    results: &[&str],
    lhs: (&str, T1),
    rhs: (&str, T2),
    data: DataFrame,
    offset: usize,
) -> TestExpr {
    let mut accessor = TestAccessor::new();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let or_expr = or(
        equal(t, lhs.0, lhs.1, &accessor),
        equal(t, rhs.0, rhs.1, &accessor),
    );
    let df_filter = polars::prelude::col(lhs.0)
        .eq(lhs.1)
        .or(polars::prelude::col(rhs.0).eq(rhs.1));
    TestExpr::new(t, results, or_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_a_simple_or_query() {
    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 1, 0, 2],
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], ("b", 1), ("b", 2), data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => [2, 4],
        )
        .unwrap(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_or_query_where_both_lhs_and_rhs_are_true() {
    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 1, 0, 1],
        "c" => [0, 2, 2, 0],
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], ("b", 1), ("c", 2), data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => [2, 3, 4],
        )
        .unwrap(),
    );
    assert_eq!(res, expected_res);
}

fn test_random_tables_with_given_offset(offset: usize) {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = ["a", "b", "c"];
    for _ in 0..10 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val1 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let filter_val2 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_expr(
            "sxt.t",
            &["a"],
            ("b", filter_val1),
            ("c", filter_val2),
            data,
            offset,
        );
        let res = test_expr.verify_expr();
        let expected_res = test_expr.query_table();
        assert_eq!(res, expected_res);
    }
}

#[test]
fn we_can_query_random_tables_with_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    test_random_tables_with_given_offset(1001);
}
