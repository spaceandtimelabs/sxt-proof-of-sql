use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            make_random_test_accessor_data, Column, ColumnType, OwnedTableTestAccessor,
            RandomTestAccessorDescriptor, RecordBatchTestAccessor, TestAccessor,
        },
        scalar::Curve25519Scalar,
    },
    owned_table, record_batch,
    sql::ast::{
        test_expr::TestExprNode,
        test_utility::{column, const_int128, const_scalar, equal, not as unot},
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

fn create_test_not_expr<T: Into<Curve25519Scalar> + Copy + Literal>(
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
    let df_filter = polars::prelude::col(filter_col).neq(lit(filter_val));
    let not_expr = unot(equal(
        column(t, filter_col, &accessor),
        const_scalar(filter_val.into()),
    ));
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

#[test]
fn we_can_compute_the_correct_output_of_a_not_expr_using_result_evaluate() {
    let data = owned_table!(
        "a" => [123_i64, 456],
        "b" => [0_i64, 1],
        "d" => ["alfa", "gama"]
    );
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let not_expr: ProvableExprPlan<RistrettoPoint> =
        unot(equal(column(t, "b", &accessor), const_int128(1)));
    let alloc = Bump::new();
    let res = not_expr.result_evaluate(2, &alloc, &accessor);
    let expected_res = Column::Boolean(&[true, false]);
    assert_eq!(res, expected_res);
}
