use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, Column, ColumnType, LiteralValue,
            OwnedTableTestAccessor, Table, TableRef, TableTestAccessor,
        },
        proof::{PlaceholderError, ProofError},
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::{
        proof::{QueryError, VerifiableQueryResult},
        proof_exprs::test_utility::*,
        proof_plans::test_utility::*,
    },
};
use bumpalo::Bump;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

fn test_random_tables_with_given_offset(offset: usize) {
    let dist = Uniform::new(-3, 4);
    let mut rng = StdRng::from_seed([0u8; 32]);
    for _ in 0..20 {
        // Generate random table
        let n = Uniform::new(1, 21).sample(&mut rng);
        let data = owned_table([
            boolean("a", dist.sample_iter(&mut rng).take(n).map(|v| v < 0)),
            varchar(
                "b",
                dist.sample_iter(&mut rng).take(n).map(|v| format!("s{v}")),
            ),
            bigint("c", dist.sample_iter(&mut rng).take(n)),
        ]);

        // Generate random values to filter by
        let random_bigint = dist.sample(&mut rng);
        let random_bigint_literal = LiteralValue::BigInt(random_bigint);
        let random_varchar = format!("s{}", dist.sample(&mut rng));
        let random_varchar_literal = LiteralValue::VarChar(random_varchar.clone());

        // Create and verify proof
        let t = TableRef::new("sxt", "t");
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t.clone(),
            data.clone(),
            offset,
            (),
        );
        let ast = filter(
            vec![
                col_expr_plan(&t, "a", &accessor),
                col_expr_plan(&t, "b", &accessor),
                col_expr_plan(&t, "c", &accessor),
                aliased_placeholder(0, ColumnType::BigInt, "p0"),
                aliased_placeholder(1, ColumnType::VarChar, "p1"),
            ],
            tab(&t),
            const_bool(true),
        );
        let params = vec![random_bigint_literal, random_varchar_literal];
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &(), &params).unwrap();
        let res = verifiable_res
            .verify(&ast, &accessor, &(), &params)
            .unwrap()
            .table;

        // Calculate/compare expected result
        let expected_a: Vec<bool> = data["a"].bool_iter().copied().collect();
        let expected_b: Vec<String> = data["b"].string_iter().cloned().collect();
        let expected_c: Vec<i64> = data["c"].i64_iter().copied().collect();
        let expected_p0: Vec<i64> = vec![random_bigint; n];
        let expected_p1: Vec<String> = vec![random_varchar.clone(); n];
        let expected_result = owned_table([
            boolean("a", expected_a),
            varchar("b", expected_b),
            bigint("c", expected_c),
            bigint("p0", expected_p0),
            varchar("p1", expected_p1),
        ]);

        assert_eq!(expected_result, res);
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
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = owned_table([bigint("a", [123_i64])]);
    let expected_res = owned_table([boolean("p0", [true])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![aliased_placeholder(0, ColumnType::Boolean, "p0")],
        tab(&t),
        const_bool(true),
    );
    let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(
        &ast,
        &accessor,
        &(),
        &[LiteralValue::Boolean(true)],
    )
    .unwrap();
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[LiteralValue::Boolean(true)])
        .unwrap()
        .table;
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_query_with_a_single_non_selected_row() {
    let data = owned_table([bigint("a", [123_i64])]);
    let expected_res = owned_table([boolean("p0", [true; 0])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![aliased_placeholder(0, ColumnType::Boolean, "p0")],
        tab(&t),
        const_bool(false),
    );
    let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(
        &ast,
        &accessor,
        &(),
        &[LiteralValue::Boolean(true)],
    )
    .unwrap();
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[LiteralValue::Boolean(true)])
        .unwrap()
        .table;
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_output_of_a_placeholder_expr_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data: Table<Curve25519Scalar> =
        table([borrowed_bigint("a", [123_i64, 456, 789, 1011], &alloc)]);
    let placeholder_expr: DynProofExpr = DynProofExpr::new_placeholder(0, ColumnType::BigInt);
    let res = placeholder_expr
        .first_round_evaluate(&alloc, &data, &[LiteralValue::BigInt(504_i64)])
        .unwrap();
    let expected_res = Column::BigInt(&[504, 504, 504, 504]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_cannot_prove_placeholder_expr_if_interpolate_fails() {
    let alloc = Bump::new();
    let data: Table<Curve25519Scalar> = table([borrowed_bigint("a", [123_i64], &alloc)]);
    let t = TableRef::new("sxt", "t");
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![aliased_placeholder(0, ColumnType::Boolean, "p0")],
        tab(&t),
        const_bool(true),
    );
    assert!(matches!(
        VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &(), &[],),
        Err(PlaceholderError::InvalidPlaceholderId { .. })
    ));
}

#[test]
fn we_cannot_verify_placeholder_expr_if_interpolate_fails() {
    let alloc = Bump::new();
    let data: Table<Curve25519Scalar> = table([borrowed_bigint("a", [123_i64], &alloc)]);
    let t = TableRef::new("sxt", "t");
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![aliased_placeholder(0, ColumnType::Boolean, "p0")],
        tab(&t),
        const_bool(true),
    );
    let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(
        &ast,
        &accessor,
        &(),
        &[LiteralValue::Boolean(true)],
    )
    .unwrap();
    assert!(matches!(
        verifiable_res.verify(&ast, &accessor, &(), &[]),
        Err(QueryError::ProofError {
            source: ProofError::PlaceholderError { .. }
        })
    ));
}

#[test]
fn we_can_verify_placeholder_expr_if_and_only_if_prover_and_verifier_have_the_same_valid_params() {
    let alloc = Bump::new();
    let data: Table<Curve25519Scalar> = table([borrowed_bigint("a", [123_i64, 456], &alloc)]);
    let t = TableRef::new("sxt", "t");
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![
            col_expr_plan(&t, "a", &accessor),
            aliased_placeholder(0, ColumnType::BigInt, "p0"),
            aliased_placeholder(1, ColumnType::VarChar, "p1"),
        ],
        tab(&t),
        const_bool(true),
    );
    let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(
        &ast,
        &accessor,
        &(),
        &[
            LiteralValue::BigInt(504_i64),
            LiteralValue::VarChar("abc".to_string()),
        ],
    )
    .unwrap();

    // Try some wrong values
    assert!(matches!(
        verifiable_res.clone().verify(
            &ast,
            &accessor,
            &(),
            &[
                LiteralValue::BigInt(503_i64),
                LiteralValue::VarChar("abc".to_string())
            ]
        ),
        Err(QueryError::ProofError { .. })
    ));

    assert!(matches!(
        verifiable_res.clone().verify(
            &ast,
            &accessor,
            &(),
            &[
                LiteralValue::BigInt(504_i64),
                LiteralValue::VarChar("abcd".to_string())
            ]
        ),
        Err(QueryError::ProofError { .. })
    ));

    assert!(matches!(
        verifiable_res.clone().verify(
            &ast,
            &accessor,
            &(),
            &[
                LiteralValue::BigInt(503_i64),
                LiteralValue::VarChar("abcd".to_string())
            ]
        ),
        Err(QueryError::ProofError { .. })
    ));

    // Now try the correct values
    let res = verifiable_res
        .verify(
            &ast,
            &accessor,
            &(),
            &[
                LiteralValue::BigInt(504_i64),
                LiteralValue::VarChar("abc".to_string()),
            ],
        )
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("a", [123_i64, 456]),
        bigint("p0", [504_i64, 504]),
        varchar("p1", ["abc", "abc"]),
    ]);
    assert_eq!(res, expected_res);
}
