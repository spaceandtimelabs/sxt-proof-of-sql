use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            make_random_test_accessor_data, owned_table_utility::*, Column, ColumnType, OwnedTable,
            OwnedTableTestAccessor, RandomTestAccessorDescriptor, RecordBatchTestAccessor,
            TestAccessor,
        },
        scalar::{Curve25519Scalar, Scalar},
    },
    record_batch,
    sql::ast::{test_expr::TestExprNode, test_utility::*, ProvableExpr, ProvableExprPlan},
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

fn create_test_col_lit_equals_expr<T: Into<Curve25519Scalar> + Copy + Literal>(
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
    let equals_expr = equal(
        column(t, filter_col, &accessor),
        const_scalar(filter_val.into()),
    );
    let df_filter = polars::prelude::col(filter_col).eq(lit(filter_val));
    TestExprNode::new(t, results, equals_expr, df_filter, accessor)
}

fn create_test_col_equals_expr(
    table_ref: &str,
    results: &[&str],
    filter_col_lhs: &str,
    filter_col_rhs: &str,
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let equals_expr = equal(
        column(t, filter_col_lhs, &accessor),
        column(t, filter_col_rhs, &accessor),
    );
    let df_filter = polars::prelude::col(filter_col_lhs).eq(col(filter_col_rhs));
    TestExprNode::new(t, results, equals_expr, df_filter, accessor)
}

// col_bool = (col_lhs = col_rhs)
fn create_test_complex_col_equals_expr(
    table_ref: &str,
    results: &[&str],
    filter_col_bool: &str,
    filter_col_lhs: &str,
    filter_col_rhs: &str,
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let equals_expr = equal(
        column(t, filter_col_bool, &accessor),
        equal(
            column(t, filter_col_lhs, &accessor),
            column(t, filter_col_rhs, &accessor),
        ),
    );
    let df_filter =
        polars::prelude::col(filter_col_bool).eq(col(filter_col_lhs).eq(col(filter_col_rhs)));
    TestExprNode::new(t, results, equals_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_an_equality_query_with_no_rows() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [0; 0]),
        bigint("b", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);

    let test_expr = create_test_col_lit_equals_expr(
        "sxt.t",
        &["a", "d"],
        "b",
        0_i64,
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => Vec::<i64>::new(),
        "d" => Vec::<String>::new(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_another_equality_query_with_no_rows() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [0; 0]),
        bigint("b", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);

    let test_expr =
        create_test_col_equals_expr("sxt.t", &["a", "d"], "a", "b", data.try_into().unwrap(), 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => Vec::<i64>::new(),
        "d" => Vec::<String>::new(),
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_nested_equality_query_with_no_rows() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        boolean("bool", [true; 0]),
        bigint("a", [1; 0]),
        bigint("b", [1; 0]),
        varchar("c", ["t"; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);

    let test_expr = create_test_complex_col_equals_expr(
        "sxt.t",
        &["b", "c", "e"],
        "bool",
        "a",
        "b",
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();

    let expected_res: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("b", [1; 0]),
        varchar("c", ["t"; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);

    assert_eq!(res, expected_res.try_into().unwrap());
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_selected_row() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123]),
        bigint("b", [0]),
        varchar("d", ["abc"]),
        decimal75("e", 75, 0, [0]),
    ]);

    let test_expr = create_test_col_lit_equals_expr(
        "sxt.t",
        &["d", "a"],
        "b",
        0_i64,
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();

    let expected_res = record_batch!(
        "d" => ["abc"],
        "a" => [123_i64],
    );

    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_another_equality_query_with_a_single_selected_row() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123]),
        bigint("b", [123]),
        varchar("d", ["abc"]),
        decimal75("e", 75, 0, [0]),
    ]);

    let test_expr =
        create_test_col_equals_expr("sxt.t", &["d", "a"], "a", "b", data.try_into().unwrap(), 0);
    let res = test_expr.verify_expr();

    let expected_res = record_batch!(
        "d" => ["abc"],
        "a" => [123_i64],
    );

    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_non_selected_row() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123]),
        bigint("b", [55]),
        varchar("d", ["abc"]),
        decimal75("e", 75, 0, [Curve25519Scalar::MAX_SIGNED]),
    ]);

    let test_expr = create_test_col_lit_equals_expr(
        "sxt.t",
        &["a", "d", "e"],
        "b",
        0_i64,
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();

    let expected_res: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);

    assert_eq!(res, expected_res.try_into().unwrap());
}

#[test]
fn we_can_prove_an_equality_query_with_multiple_rows() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 2, 3, 4]),
        bigint("b", [0, 5, 0, 5]),
        varchar("c", ["t", "ghi", "jj", "f"]),
        decimal75(
            "e",
            75,
            0,
            [
                Curve25519Scalar::ZERO,
                Curve25519Scalar::ONE,
                Curve25519Scalar::TWO,
                Curve25519Scalar::MAX_SIGNED,
            ],
        ),
    ]);

    let test_expr = create_test_col_lit_equals_expr(
        "sxt.t",
        &["a", "c", "e"],
        "b",
        0_i64,
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();

    let expected_res: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 3]),
        varchar("c", ["t", "jj"]),
        decimal75("e", 75, 0, [0, 2]),
    ]);

    assert_eq!(res, expected_res.try_into().unwrap());
}

#[test]
fn we_can_prove_a_nested_equality_query_with_multiple_rows() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        boolean("bool", [true, false, true, false]),
        bigint("a", [1, 2, 3, 4]),
        bigint("b", [1, 5, 0, 4]),
        varchar("c", ["t", "ghi", "jj", "f"]),
        decimal75(
            "e",
            75,
            0,
            [
                Curve25519Scalar::ZERO,
                Curve25519Scalar::ONE,
                Curve25519Scalar::TWO,
                Curve25519Scalar::MAX_SIGNED,
            ],
        ),
    ]);

    let test_expr = create_test_complex_col_equals_expr(
        "sxt.t",
        &["a", "c", "e"],
        "bool",
        "a",
        "b",
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();

    let expected_res: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 2]),
        varchar("c", ["t", "ghi"]),
        decimal75("e", 75, 0, [0, 1]),
    ]);

    assert_eq!(res, expected_res.try_into().unwrap());
}

#[test]
fn we_can_prove_an_equality_query_with_a_nonzero_comparison() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 2, 3, 4, 5]),
        bigint("b", [123, 5, 123, 5, 0]),
        varchar("c", ["t", "ghi", "jj", "f", "abc"]),
        decimal75(
            "e",
            42,
            10,
            [
                Curve25519Scalar::ZERO,
                Curve25519Scalar::ONE,
                Curve25519Scalar::TWO,
                Curve25519Scalar::from(3),
                Curve25519Scalar::MAX_SIGNED,
            ],
        ),
    ]);

    let test_expr = create_test_col_lit_equals_expr(
        "sxt.t",
        &["a", "c", "e"],
        "b",
        123_u64,
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();

    let expected_res: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 3]),
        varchar("c", ["t", "jj"]),
        decimal75("e", 42, 10, vec![0, 2]),
    ]);

    assert_eq!(res, expected_res.try_into().unwrap());
}

#[test]
fn we_can_prove_an_equality_query_with_a_string_comparison() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 2, 3, 4, 5, 5]),
        bigint("b", [123, 5, 123, 123, 5, 0]),
        varchar("c", ["t", "ghi", "jj", "f", "abc", "ghi"]),
        decimal75(
            "e",
            42, // precision
            10, // scale
            [
                Curve25519Scalar::ZERO,
                Curve25519Scalar::ONE,
                Curve25519Scalar::TWO,
                Curve25519Scalar::from(3),
                Curve25519Scalar::MAX_SIGNED,
                Curve25519Scalar::from(-1),
            ],
        ),
    ]);

    let test_expr = create_test_col_lit_equals_expr(
        "sxt.t",
        &["a", "b", "e"],
        "c",
        "ghi",
        data.try_into().unwrap(),
        0,
    );
    let res = test_expr.verify_expr();

    let expected_res: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [2, 5]),
        bigint("b", [5, 0]),
        decimal75("e", 42, 10, [1, -1]),
    ]);

    assert_eq!(res, expected_res.try_into().unwrap());
}

#[test]
fn verify_fails_if_data_between_prover_and_verifier_differ() {
    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "c" => ["t", "ghi", "jj", "f"],
        "b" => [0_i64, 5, 0, 5],
    );
    let test_expr = create_test_col_lit_equals_expr("sxt.t", &["a", "c"], "b", 0_u64, data, 0);

    let data = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "c" => ["t", "ghi", "jj", "f"],
        "b" => [0_i64, 2, 0, 5],
    );
    let tampered_test_expr =
        create_test_col_lit_equals_expr("sxt.t", &["a", "c"], "b", 0_u64, data, 0);

    let res = test_expr.create_verifiable_result();
    assert!(res
        .verify(&test_expr.ast, &tampered_test_expr.accessor, &())
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
        let test_expr = create_test_col_lit_equals_expr(
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
        let test_expr = create_test_col_lit_equals_expr(
            "sxt.t",
            &["aa", "ab", "b"],
            "b",
            filter_val,
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
    we_can_query_random_tables_with_multiple_selected_rows_and_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    we_can_query_random_tables_with_multiple_selected_rows_and_given_offset(121);
}

#[test]
fn we_can_compute_the_correct_output_of_an_equals_expr_using_result_evaluate() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 2, 3, 4]),
        bigint("b", [0, 5, 0, 5]),
        varchar("c", ["t", "ghi", "jj", "f"]),
        decimal75(
            "e",
            42,
            10,
            [
                Curve25519Scalar::ZERO,
                Curve25519Scalar::MAX_SIGNED,
                Curve25519Scalar::ZERO,
                Curve25519Scalar::from(-1),
            ],
        ),
    ]);

    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let equals_expr: ProvableExprPlan<RistrettoPoint> = equal(
        column(t, "e", &accessor),
        const_scalar(Curve25519Scalar::ZERO),
    );
    let alloc = Bump::new();
    let res = equals_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = Column::Boolean(&[true, false, true, false]);
    assert_eq!(res, expected_res);
}
