use super::BoolExpr;
use crate::{
    base::{
        database::{
            make_random_test_accessor_data, ColumnType, OwnedTableTestAccessor,
            RandomTestAccessorDescriptor, RecordBatchTestAccessor, TestAccessor,
        },
        scalar::ArkScalar,
    },
    owned_table, record_batch,
    sql::ast::{test_expr::TestExprNode, test_utility::equal},
};
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

fn create_test_equals_expr<T: Into<ArkScalar> + Copy + Literal>(
    table_ref: &str,
    results: &[&str],
    filter_col: &str,
    filter_val: T,
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let equals_expr = equal(t, filter_col, filter_val, &accessor);
    let df_filter = polars::prelude::col(filter_col).eq(lit(filter_val));
    TestExprNode::new(t, results, equals_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_an_equality_query_with_no_rows() {
    let data = record_batch!(
        "a" => Vec::<i64>::new(),
        "b" => Vec::<i64>::new(),
        "d" => Vec::<String>::new(),
    );
    let test_expr = create_test_equals_expr("sxt.t", &["a", "d"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => Vec::<i64>::new(),
        "d" => Vec::<String>::new(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_selected_row() {
    let data = record_batch!(
        "a" => [123_i64],
        "b" => [0_i64],
        "d" => ["abc"]
    );
    let test_expr = create_test_equals_expr("sxt.t", &["d", "a"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "d" => ["abc"],
        "a" => [123_i64],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_non_selected_row() {
    let data = record_batch!(
        "a" => [123_i64],
        "b" => [55_i64],
        "d" => ["abc"]
    );
    let test_expr = create_test_equals_expr("sxt.t", &["a", "d"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => Vec::<i64>::new(),
        "d" => Vec::<String>::new(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_multiple_rows() {
    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "c" => ["t", "ghi", "jj", "f"],
        "b" => [0_i64, 5, 0, 5],
    );
    let test_expr = create_test_equals_expr("sxt.t", &["a", "c"], "b", 0_i64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => [1_i64, 3],
        "c" => ["t", "jj"],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_nonzero_comparison() {
    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4, 5],
        "b" => [123_i64, 5, 123, 5, 0],
        "c" => ["t", "ghi", "jj", "f", "abc"],
    );
    let test_expr = create_test_equals_expr("sxt.t", &["a", "c"], "b", 123_u64, data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => [1_i64, 3],
        "c" => ["t", "jj"],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_string_comparison() {
    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4, 5],
        "b" => [123_i64, 5, 123, 5, 0],
        "c" => ["t", "ghi", "jj", "f", "ghi"],
    );
    let test_expr = create_test_equals_expr("sxt.t", &["a", "b"], "c", "ghi", data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => [2_i64, 5],
        "b" => [5_i64, 0],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn verify_fails_if_data_between_prover_and_verifier_differ() {
    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "c" => ["t", "ghi", "jj", "f"],
        "b" => [0_i64, 5, 0, 5],
    );
    let test_expr = create_test_equals_expr("sxt.t", &["a", "c"], "b", 0_u64, data, 0);

    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "c" => ["t", "ghi", "jj", "f"],
        "b" => [0_i64, 2, 0, 5],
    );
    let tampered_test_expr = create_test_equals_expr("sxt.t", &["a", "c"], "b", 0_u64, data, 0);

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
    let cols = [
        ("aa", ColumnType::BigInt),
        ("ab", ColumnType::VarChar),
        ("b", ColumnType::BigInt),
    ];
    for _ in 0..20 {
        // filtering by string value
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_equals_expr(
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
            create_test_equals_expr("sxt.t", &["aa", "ab", "b"], "b", filter_val, data, offset);
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

#[test]
fn we_can_compute_the_correct_output_of_an_equals_expr_using_result_evaluate() {
    let data = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "c" => ["t", "ghi", "jj", "f"],
        "b" => [0_i64, 5, 0, 5],
    );
    let mut accessor = OwnedTableTestAccessor::new_empty();
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let equals_expr = equal(t, "b", 0, &accessor);
    let alloc = Bump::new();
    let res = equals_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = &[true, false, true, false];
    assert_eq!(res, expected_res);
}
