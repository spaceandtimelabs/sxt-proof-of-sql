use crate::{
    base::database::{
        make_random_test_accessor_data, ColumnType, RandomTestAccessorDescriptor,
        RecordBatchTestAccessor, TestAccessor,
    },
    record_batch,
    sql::ast::{test_expr::TestExprNode, test_utility::const_v},
};
use arrow::record_batch::RecordBatch;
use polars::prelude::*;
use rand::rngs::StdRng;
use rand_core::SeedableRng;

fn create_test_const_bool_expr(
    table_ref: &str,
    results: &[&str],
    filter_val: bool,
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref = table_ref.parse().unwrap();
    accessor.add_table(table_ref, data, offset);
    let df_filter = lit(filter_val);
    let const_expr = const_v(filter_val);
    TestExprNode::new(table_ref, results, const_expr, df_filter, accessor)
}

fn test_random_tables_with_given_constant(value: bool) {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = [("a", ColumnType::BigInt), ("b", ColumnType::VarChar)];
    for _ in 0..10 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let test_expr = create_test_const_bool_expr("sxt.t", &["a", "b"], value, data, 0);
        let res = test_expr.verify_expr();
        let expected_res = test_expr.query_table();
        assert_eq!(res, expected_res);
    }
}

#[test]
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = record_batch!("a" => [123_i64]);
    let test_expr = create_test_const_bool_expr("sxt.t", &["a"], true, data.clone(), 0);
    let res = test_expr.verify_expr();
    assert_eq!(res, data);
}

#[test]
fn we_can_prove_a_query_with_a_single_non_selected_row() {
    let data = record_batch!("a" => [123_i64]);
    let test_expr = create_test_const_bool_expr("sxt.t", &["a"], false, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!("a" => Vec::<i64>::new());
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_select_from_tables_with_an_always_true_where_clause() {
    test_random_tables_with_given_constant(true);
}

#[test]
fn we_can_select_from_tables_with_an_always_false_where_clause() {
    test_random_tables_with_given_constant(false);
}
