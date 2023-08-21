use crate::base::database::{
    make_random_test_accessor_data, ColumnType, RandomTestAccessorDescriptor, TestAccessor,
};
use crate::base::scalar::ArkScalar;
use crate::record_batch;
use crate::sql::ast::test_expr::TestExprNode;
use crate::sql::ast::test_utility::{equal, not};
use arrow::record_batch::RecordBatch;

use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

fn create_test_not_expr<T: Into<ArkScalar> + Copy + Literal>(
    table_ref: &str,
    results: &[&str],
    filter_col: &str,
    filter_val: T,
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = TestAccessor::new();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let df_filter = polars::prelude::col(filter_col).neq(lit(filter_val));
    let not_expr = not(equal(t, filter_col, filter_val, &accessor));
    TestExprNode::new(t, results, not_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_a_not_equals_query_with_a_single_selected_row() {
    let data = record_batch!(
        "a" => [123_i64, 456],
        "b" => [0_i64, 1],
        "d" => ["alfa", "gama"]
    );
    let test_expr = create_test_not_expr("sxt.t", &["a", "d"], "b", 1_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => [123_i64],
        "d" => ["alfa"]
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
    let cols = [
        ("aa", ColumnType::BigInt),
        ("ab", ColumnType::VarChar),
        ("b", ColumnType::BigInt),
    ];
    for _ in 0..20 {
        // filtering by string value
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_not_expr(
            "sxt.t",
            &["aa", "ab", "b"],
            "ab",
            ("s".to_owned() + &filter_val.to_string()[..]).as_str(),
            data,
            offset,
        );
        let res = test_expr.verify_expr();
        let expected_res = test_expr.query_table();
        assert_eq!(res, expected_res);

        // filtering by integer value
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr =
            create_test_not_expr("sxt.t", &["aa", "ab", "b"], "b", filter_val, data, offset);
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
