use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, Column, LiteralValue, OwnedTable, OwnedTableTestAccessor,
            TestAccessor,
        },
        math::decimal::scale_scalar,
        scalar::{Curve25519Scalar, Scalar},
    },
    sql::{
        parse::ConversionError,
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::{test_utility::*, DynProofExpr, ProofExpr},
        proof_plans::{test_utility::*, DynProofPlan},
    },
};
use bumpalo::Bump;
use curve25519_dalek::RistrettoPoint;
use itertools::{multizip, MultiUnzip};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

#[cfg_attr(test, allow(clippy::missing_panics_doc))]
#[test]
fn we_can_compare_columns_with_small_timestamp_values_gte() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([timestamptz(
        "a",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::Utc,
        vec![-1, 0, 1],
    )]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a"], &accessor),
        tab(t),
        gte(
            column(t, "a", &accessor),
            DynProofExpr::new_literal(LiteralValue::TimeStampTZ(
                PoSQLTimeUnit::Nanosecond,
                PoSQLTimeZone::Utc,
                1,
            )),
        ),
    );

    let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([timestamptz(
        "a",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::Utc,
        vec![1],
    )]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_columns_with_small_timestamp_values_lte() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([timestamptz(
        "a",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::Utc,
        vec![-1, 0, 1],
    )]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a"], &accessor),
        tab(t),
        lte(
            column(t, "a", &accessor),
            DynProofExpr::new_literal(LiteralValue::TimeStampTZ(
                PoSQLTimeUnit::Nanosecond,
                PoSQLTimeZone::Utc,
                1,
            )),
        ),
    );

    let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([timestamptz(
        "a",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::Utc,
        vec![-1, 0],
    )]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_constant_column() {
    let data = owned_table([bigint("a", [123_i64, 123, 123]), bigint("b", [1_i64, 2, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(5)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [0; 0])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_varying_column_with_constant_sign() {
    let data = owned_table([bigint("a", [123_i64, 567, 8]), bigint("b", [1_i64, 2, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(5)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [0; 0])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_columns_with_extreme_values() {
    let data = owned_table([
        bigint("bigint_a", [i64::MAX, i64::MIN, i64::MAX]),
        bigint("bigint_b", [i64::MAX, i64::MAX, i64::MIN]),
        int128("int128_a", [i128::MIN, i128::MAX, i128::MAX]),
        int128("int128_b", [i128::MAX, i128::MIN, i128::MAX]),
        boolean("boolean", [true, false, true]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["bigint_b"], &accessor),
        tab(t),
        lte(
            lte(
                lte(
                    column(t, "bigint_a", &accessor),
                    column(t, "bigint_b", &accessor),
                ),
                gte(
                    column(t, "int128_a", &accessor),
                    column(t, "int128_b", &accessor),
                ),
            ),
            column(t, "boolean", &accessor),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("bigint_b", [i64::MAX, i64::MIN])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_columns_with_small_decimal_values_without_scale() {
    let scalar_pos = scale_scalar(Curve25519Scalar::ONE, 38).unwrap() - Curve25519Scalar::ONE;
    let scalar_neg = -scalar_pos;
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123, 25]),
        bigint("b", [55, -53]),
        varchar("d", ["abc", "de"]),
        decimal75("e", 38, 0, [scalar_pos, scalar_neg]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a", "d", "e"], &accessor),
        tab(t),
        lte(column(t, "e", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [25]),
        varchar("d", ["de"]),
        decimal75("e", 38, 0, [scalar_neg]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_columns_with_small_decimal_values_with_scale() {
    let scalar_pos = scale_scalar(Curve25519Scalar::ONE, 38).unwrap() - Curve25519Scalar::ONE;
    let scalar_neg = -scalar_pos;
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123, 25]),
        bigint("b", [55, -53]),
        varchar("d", ["abc", "de"]),
        decimal75("e", 38, 0, [scalar_pos, scalar_neg]),
        decimal75("f", 38, 38, [scalar_neg, scalar_pos]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a", "d", "e", "f"], &accessor),
        tab(t),
        lte(column(t, "f", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [123]),
        varchar("d", ["abc"]),
        decimal75("e", 38, 0, [scalar_pos]),
        decimal75("f", 38, 38, [scalar_neg]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_columns_with_small_decimal_values_with_differing_scale_gte() {
    let scalar_pos = scale_scalar(Curve25519Scalar::ONE, 38).unwrap() - Curve25519Scalar::ONE;
    let scalar_neg = -scalar_pos;
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123, 25]),
        bigint("b", [55, -53]),
        varchar("d", ["abc", "de"]),
        decimal75("e", 38, 0, [scalar_pos, scalar_neg]),
        decimal75("f", 38, 38, [scalar_neg, scalar_pos]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a", "d", "e", "f"], &accessor),
        tab(t),
        gte(column(t, "f", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [25]),
        varchar("d", ["de"]),
        decimal75("e", 38, 0, [scalar_neg]),
        decimal75("f", 38, 38, [scalar_pos]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_columns_returning_extreme_decimal_values() {
    let scalar_min_signed = -Curve25519Scalar::MAX_SIGNED - Curve25519Scalar::ONE;
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123, 25]),
        bigint("b", [55, -53]),
        varchar("d", ["abc", "de"]),
        decimal75(
            "e",
            75,
            0,
            [Curve25519Scalar::MAX_SIGNED, scalar_min_signed],
        ),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a", "d", "e"], &accessor),
        tab(t),
        lte(column(t, "b", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [25]),
        varchar("d", ["de"]),
        decimal75("e", 75, 0, [scalar_min_signed]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_cannot_compare_columns_filtering_on_extreme_decimal_values() {
    let scalar_min_signed = -Curve25519Scalar::MAX_SIGNED - Curve25519Scalar::ONE;
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123, 25]),
        bigint("b", [55, -53]),
        varchar("d", ["abc", "de"]),
        decimal75(
            "e",
            75,
            0,
            [Curve25519Scalar::MAX_SIGNED, scalar_min_signed],
        ),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    assert!(matches!(
        DynProofExpr::try_new_inequality(
            column(t, "e", &accessor),
            const_scalar::<RistrettoPoint, Curve25519Scalar>(Curve25519Scalar::ONE),
            false
        ),
        Err(ConversionError::DataTypeMismatch(_, _))
    ));
}

#[test]
fn we_can_compare_two_columns() {
    let data = owned_table([bigint("a", [1_i64, 5, 8]), bigint("b", [1_i64, 7, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), column(t, "b", &accessor)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64, 7])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_varying_column_with_constant_absolute_value() {
    let data = owned_table([
        bigint("a", [-123_i64, 123, -123]),
        bigint("b", [1_i64, 2, 3]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(0)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64, 3])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_constant_column_of_negative_columns() {
    let data = owned_table([
        bigint("a", [-123_i64, -123, -123]),
        bigint("b", [1_i64, 2, 3]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(5)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64, 2, 3])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_varying_column_with_negative_only_signs() {
    let data = owned_table([
        bigint("a", [-123_i64, -133, -823]),
        bigint("b", [1_i64, 2, 3]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(5)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64, 2, 3])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_column_with_varying_absolute_values_and_signs() {
    let data = owned_table([bigint("a", [-1_i64, 9, 0]), bigint("b", [1_i64, 2, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(1)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64, 3])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_column_with_greater_than_or_equal() {
    let data = owned_table([bigint("a", [-1_i64, 9, 0]), bigint("b", [1_i64, 2, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        gte(column(t, "a", &accessor), const_bigint(1)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [2_i64])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_run_nested_comparison() {
    let data = owned_table([
        bigint("a", [0_i64, 2, 4]),
        bigint("b", [1_i64, 2, 3]),
        boolean("boolean", [false, false, true]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        equal(
            gte(column(t, "a", &accessor), column(t, "b", &accessor)),
            column(t, "boolean", &accessor),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64, 3])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_column_with_varying_absolute_values_and_signs_and_a_constant_bit() {
    let data = owned_table([bigint("a", [-2_i64, 3, 2]), bigint("b", [1_i64, 2, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(0)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_constant_column_of_zeros() {
    let data = owned_table([bigint("a", [0_i64, 0, 0]), bigint("b", [1_i64, 2, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(0)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [1_i64, 2, 3])]);
    assert_eq!(res, expected_res);
}

#[test]
fn the_sign_can_be_0_or_1_for_a_constant_column_of_zeros() {
    let data = owned_table([bigint("a", [0_i64, 0, 0]), bigint("b", [1_i64, 2, 3])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let mut ast = filter(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        lte(column(t, "a", &accessor), const_bigint(0)),
    );
    if let DynProofPlan::Filter(filter) = &mut ast {
        if let DynProofExpr::Inequality(lte) = &mut filter.where_clause {
            lte.treat_column_of_zeros_as_negative = true
        }
    }
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected = owned_table([bigint("b", [1_i64, 2, 3])]);
    assert_eq!(res, expected);
}

fn test_random_tables_with_given_offset(offset: usize) {
    let dist = Uniform::new(-3, 4);
    let mut rng = StdRng::from_seed([0u8; 32]);
    for _ in 0..20 {
        // Generate random table
        let n = Uniform::new(1, 21).sample(&mut rng);
        let data = owned_table([
            bigint("a", dist.sample_iter(&mut rng).take(n)),
            varchar(
                "b",
                dist.sample_iter(&mut rng).take(n).map(|v| format!("s{v}")),
            ),
        ]);

        // Generate random values to filter by
        let filter_val = dist.sample(&mut rng);

        // Create and verify proof
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t,
            data.clone(),
            offset,
            (),
        );
        let ast = filter(
            cols_expr_plan(t, &["a", "b"], &accessor),
            tab(t),
            lte(column(t, "a", &accessor), const_bigint(filter_val)),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
        exercise_verification(&verifiable_res, &ast, &accessor, t);
        let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;

        // Calculate/compare expected result
        let (expected_a, expected_b): (Vec<_>, Vec<_>) =
            multizip((data["a"].i64_iter(), data["b"].string_iter()))
                .filter_map(|(a, b)| {
                    if a <= &filter_val {
                        Some((*a, b.clone()))
                    } else {
                        None
                    }
                })
                .multiunzip();
        let expected_result = owned_table([bigint("a", expected_a), varchar("b", expected_b)]);

        assert_eq!(expected_result, res)
    }
}

#[test]
fn we_can_query_random_tables_using_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_using_a_non_zero_offset() {
    test_random_tables_with_given_offset(5121);
}

#[test]
fn we_can_compute_the_correct_output_of_a_lte_inequality_expr_using_result_evaluate() {
    let data = owned_table([bigint("a", [-1, 9, 1]), bigint("b", [1, 2, 3])]);
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let lhs_expr: DynProofExpr<RistrettoPoint> = column(t, "a", &accessor);
    let rhs_expr = column(t, "b", &accessor);
    let lte_expr = lte(lhs_expr, rhs_expr);
    let alloc = Bump::new();
    let res = lte_expr.result_evaluate(3, &alloc, &accessor);
    let expected_res = Column::Boolean(&[true, false, true]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_output_of_a_gte_inequality_expr_using_result_evaluate() {
    let data = owned_table([bigint("a", [-1, 9, 1]), bigint("b", [1, 2, 3])]);
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let col_expr: DynProofExpr<RistrettoPoint> = column(t, "a", &accessor);
    let lit_expr = const_bigint(1);
    let gte_expr = gte(col_expr, lit_expr);
    let alloc = Bump::new();
    let res = gte_expr.result_evaluate(3, &alloc, &accessor);
    let expected_res = Column::Boolean(&[false, true, true]);
    assert_eq!(res, expected_res);
}
