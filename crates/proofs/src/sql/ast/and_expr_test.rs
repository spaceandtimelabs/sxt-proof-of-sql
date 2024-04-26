use super::{test_utility::*, FilterExpr, ProvableExpr};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            make_random_test_accessor_data, ColumnType, OwnedTable, OwnedTableTestAccessor,
            RandomTestAccessorDescriptor, TestAccessor,
        },
        scalar::Curve25519Scalar,
    },
    owned_table,
    sql::{
        ast::{
            test_utility::{and, equal},
            ProvableExprPlan,
        },
        proof::{exercise_verification, VerifiableQueryResult},
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;
/// This function creates a TestAccessor, adds a table, and then creates a FilterExpr with the given parameters.
/// It then executes the query, verifies the result, and returns the table.
fn create_and_verify_test_and_expr(
    table_ref: &str,
    results: &[&str],
    lhs: (&str, impl Into<Curve25519Scalar>),
    rhs: (&str, impl Into<Curve25519Scalar>),
    data: OwnedTable<Curve25519Scalar>,
    offset: usize,
) -> OwnedTable<Curve25519Scalar> {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let and_expr = and(
        equal(t, lhs.0, lhs.1, &accessor),
        equal(t, rhs.0, rhs.1, &accessor),
    );
    let ast = FilterExpr::new(cols_result(t, results, &accessor), tab(t), and_expr);
    let res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&res, &ast, &accessor, t);
    res.verify(&ast, &accessor, &()).unwrap().table
}
/// This function filters the given data using polars with the given parameters.
fn filter_test_and_expr(
    results: &[&str],
    lhs: (&str, impl polars::prelude::Literal),
    rhs: (&str, impl polars::prelude::Literal),
    data: OwnedTable<Curve25519Scalar>,
) -> OwnedTable<Curve25519Scalar> {
    let df_filter = polars::prelude::col(lhs.0)
        .eq(lit(lhs.1))
        .and(polars::prelude::col(rhs.0).eq(lit(rhs.1)));
    data.apply_polars_filter(results, df_filter)
}

#[test]
fn we_can_prove_a_simple_and_query() {
    let data = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [0_i64, 1, 0, 1],
        "d" => ["ab", "t", "efg", "g"],
        "c" => [0_i64, 2, 2, 0],
    );
    let res = create_and_verify_test_and_expr("sxt.t", &["a", "d"], ("b", 1), ("d", "t"), data, 0);
    let expected_res = owned_table!(
        "a" => [2_i64],
        "d" => ["t"]
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_simple_and_query_with_128_bits() {
    let data = owned_table!(
        "a" => [1_i128, 2, 3, 4],
        "b" => [0_i128, 1, 0, 1],
        "d" => ["ab", "t", "efg", "g"],
        "c" => [0_i128, 2, 2, 0],
    );
    let res = create_and_verify_test_and_expr("sxt.t", &["a", "d"], ("b", 1), ("d", "t"), data, 0);
    let expected_res = owned_table!(
        "a" => [2_i128],
        "d" => ["t"]
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
        let data = OwnedTable::try_from(data).unwrap();
        let filter_val1 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let filter_val1 = format!("s{filter_val1}");
        let filter_val2 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let results = &["a", "d"];
        let lhs = ("b", filter_val1.as_str());
        let rhs = ("c", filter_val2);
        assert_eq!(
            filter_test_and_expr(results, lhs, rhs, data.clone()),
            create_and_verify_test_and_expr("sxt.t", results, lhs, rhs, data, offset)
        )
    }
}

#[test]
fn we_can_query_random_tables_using_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_using_a_non_zero_offset() {
    test_random_tables_with_given_offset(123);
}

#[test]
fn we_can_compute_the_correct_output_of_an_and_expr_using_result_evaluate() {
    let data = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [0_i64, 1, 0, 1],
        "d" => ["ab", "t", "efg", "g"],
        "c" => [0_i64, 2, 2, 0],
    );
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let and_expr: ProvableExprPlan<RistrettoPoint> =
        and(equal(t, "b", 1, &accessor), equal(t, "d", "t", &accessor));
    let alloc = Bump::new();
    let res = and_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = &[false, true, false, false];
    assert_eq!(res, expected_res);
}
