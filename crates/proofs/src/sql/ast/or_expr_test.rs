use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            make_random_test_accessor_data, ColumnType, OwnedTableTestAccessor,
            RandomTestAccessorDescriptor, RecordBatchTestAccessor, TestAccessor,
        },
        scalar::Curve25519Scalar,
    },
    owned_table, record_batch,
    sql::ast::{
        test_expr::TestExprNode,
        test_utility::{equal, or},
        ProvableExpr, ProvableExprPlan,
    },
};
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

fn create_test_or_expr<
    T1: Into<Curve25519Scalar> + Copy + Literal,
    T2: Into<Curve25519Scalar> + Copy + Literal,
>(
    table_ref: &str,
    results: &[&str],
    lhs: (&str, T1),
    rhs: (&str, T2),
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let or_expr = or(
        equal(t, lhs.0, lhs.1, &accessor),
        equal(t, rhs.0, rhs.1, &accessor),
    );
    let df_filter = polars::prelude::col(lhs.0)
        .eq(lit(lhs.1))
        .or(polars::prelude::col(rhs.0).eq(lit(rhs.1)));
    TestExprNode::new(t, results, or_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_a_simple_or_query() {
    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "d" => ["ab", "t", "g", "efg"],
        "b" => [0_i64, 1, 0, 2],
    );
    let test_expr = create_test_or_expr("sxt.t", &["a"], ("b", 1), ("d", "efgh"), data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => [2_i64],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_simple_or_query_with_i128_data() {
    let data = record_batch!(
        "a" => [1_i128, 2, 3, 4],
        "d" => ["ab", "t", "g", "efg"],
        "b" => [0_i128, 1, 0, 2],
    );
    let test_expr = create_test_or_expr("sxt.t", &["a"], ("b", 1), ("d", "efgh"), data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => [2_i128],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_or_query_where_both_lhs_and_rhs_are_true() {
    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [0_i64, 1, 0, 1],
        "c" => [0_i64, 2, 2, 0],
        "d" => ["ab", "t", "g", "efg"],
    );
    let test_expr = create_test_or_expr("sxt.t", &["d"], ("b", 1), ("d", "g"), data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "d" => ["t", "g", "efg"],
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
        ("a", ColumnType::BigInt),
        ("b", ColumnType::VarChar),
        ("c", ColumnType::BigInt),
        ("d", ColumnType::VarChar),
    ];
    for _ in 0..20 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val1 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let filter_val2 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_or_expr(
            "sxt.t",
            &["a", "d"],
            (
                "b",
                ("s".to_owned() + &filter_val1.to_string()[..]).as_str(),
            ),
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

#[test]
fn we_can_compute_the_correct_output_of_an_or_expr_using_result_evaluate() {
    let data = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [0_i64, 1, 0, 1],
        "c" => [0_i64, 2, 2, 0],
        "d" => ["ab", "t", "g", "efg"],
    );
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let and_expr: ProvableExprPlan<RistrettoPoint> =
        or(equal(t, "b", 1, &accessor), equal(t, "d", "g", &accessor));
    let alloc = Bump::new();
    let res = and_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = &[false, true, true, true];
    assert_eq!(res, expected_res);
}
