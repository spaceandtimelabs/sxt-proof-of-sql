use crate::base::database::data_frame_to_record_batch;
use crate::base::database::{
    make_random_test_accessor_data, RandomTestAccessorDescriptor, TestAccessor,
};
use crate::base::scalar::ToScalar;
use crate::sql::ast::test_expr::TestExpr;
use crate::sql::ast::test_utility::{equal, not};

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
    let df_filter = polars::prelude::col(filter_col).neq(filter_val);
    let not_expr = not(equal(t, filter_col, filter_val, &accessor));
    TestExpr::new(t, results, not_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_a_not_equals_query_with_a_single_selected_row() {
    let data = df!(
        "a" => [123, 456],
        "b" => [0, 1],
    )
    .unwrap();
    let test_expr = create_test_expr("sxt.t", &["a"], "b", 1_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = data_frame_to_record_batch(
        &df!(
            "a" => [123]
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
    let cols = ["a", "b"];
    for _ in 0..10 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_expr("sxt.t", &["a"], "b", filter_val, data, offset);
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
    test_random_tables_with_given_offset(75);
}
