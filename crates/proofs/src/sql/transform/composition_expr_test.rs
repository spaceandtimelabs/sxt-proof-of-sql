use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::test_utility::{composite_result, orders, slice};
use crate::sql::transform::CompositionExpr;
use proofs_sql::intermediate_ast::OrderByDirection::Desc;

#[test]
fn we_can_chain_expressions() {
    let limit = 2;
    let offset = 1;
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let mut composition = CompositionExpr::new(orders(&["c"], &[Desc]));
    composition.add(slice(limit, offset));

    let result_expr = composite_result(vec![Box::new(composition)]);
    let data = result_expr.transform_results(data);
    let expected_data = record_batch!("c" => [1_i64, -5], "a" => ["a", "d"]);
    assert_eq!(data, expected_data);
}

#[test]
fn the_order_that_we_chain_expressions_is_relevant() {
    let limit = 2;
    let offset = 1;
    let data = record_batch!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);

    let mut composition1 = CompositionExpr::new(orders(&["c"], &[Desc]));
    composition1.add(slice(limit, offset));
    let result_expr1 = composite_result(vec![Box::new(composition1)]);
    let data1 = result_expr1.transform_results(data.clone());

    let mut composition2 = CompositionExpr::new(slice(limit, offset));
    composition2.add(orders(&["c"], &[Desc]));
    let result_expr2 = composite_result(vec![Box::new(composition2)]);
    let data2 = result_expr2.transform_results(data);

    assert_ne!(data1, data2);

    let expected_data1 = record_batch!("c" => [1_i64, -5], "a" => ["a", "d"]);
    assert_eq!(data1, expected_data1);

    let expected_data2 = record_batch!("c" => [1_i64, -56], "a" => ["a", "f"]);
    assert_eq!(data2, expected_data2);
}
