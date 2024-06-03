use crate::{
    base::database::{RecordBatchTestAccessor, TestAccessor},
    record_batch,
    sql::ast::{test_expr::TestExprNode, test_utility::*},
};
use arrow::record_batch::RecordBatch;

fn create_test_bool_col_expr(
    table_ref: &str,
    results: &[&str],
    filter_col: &str,
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref = table_ref.parse().unwrap();
    accessor.add_table(table_ref, data, offset);
    let col_expr = column(table_ref, filter_col, &accessor);
    let df_filter = polars::prelude::col(filter_col);
    TestExprNode::new(table_ref, results, col_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = record_batch!("a" => [true, false]);
    let test_expr = create_test_bool_col_expr("sxt.t", &["a"], "a", data.clone(), 0);
    let res = test_expr.verify_expr();
    let expected = record_batch!("a" => [true]);
    assert_eq!(res, expected);
}
