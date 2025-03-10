use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, Column, ColumnRef, ColumnType,
            OwnedTableTestAccessor, Table, TableRef, TableTestAccessor,
        },
        map::indexmap,
        polynomial::MultilinearExtension,
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::{
        proof::{
            exercise_verification,
            mock_verification_builder::{run_verify_for_each_row, MockVerificationBuilder},
            FinalRoundBuilder, VerifiableQueryResult,
        },
        proof_exprs::{test_utility::*, AndExpr, ColumnExpr, DynProofExpr, ProofExpr},
        proof_plans::test_utility::*,
    },
};
use bumpalo::Bump;
use itertools::{multizip, MultiUnzip};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;
use sqlparser::ast::Ident;
use std::collections::VecDeque;

#[test]
fn we_can_prove_a_simple_and_query() {
    let data = owned_table([
        bigint("a", [1, 2, 3, 4]),
        bigint("b", [0, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0, 2, 2, 0]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        cols_expr_plan(&t, &["a", "d"], &accessor),
        tab(&t),
        and(
            equal(column(&t, "b", &accessor), const_scalar::<TestScalar, _>(1)),
            equal(
                column(&t, "d", &accessor),
                const_scalar::<TestScalar, _>("t"),
            ),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [2]), varchar("d", ["t"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_simple_and_query_with_128_bits() {
    let data = owned_table([
        int128("a", [1, 2, 3, 4]),
        int128("b", [0, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        int128("c", [0, 2, 2, 0]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        cols_expr_plan(&t, &["a", "d"], &accessor),
        tab(&t),
        and(
            equal(column(&t, "b", &accessor), const_scalar::<TestScalar, _>(1)),
            equal(
                column(&t, "d", &accessor),
                const_scalar::<TestScalar, _>("t"),
            ),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([int128("a", [2]), varchar("d", ["t"])]);
    assert_eq!(res, expected_res);
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
            bigint("c", dist.sample_iter(&mut rng).take(n)),
            varchar(
                "d",
                dist.sample_iter(&mut rng).take(n).map(|v| format!("s{v}")),
            ),
        ]);

        // Generate random values to filter by
        let filter_val1 = format!("s{}", dist.sample(&mut rng));
        let filter_val2 = dist.sample(&mut rng);

        // Create and verify proof
        let t = TableRef::new("sxt", "t");
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t.clone(),
            data.clone(),
            offset,
            (),
        );
        let ast = filter(
            cols_expr_plan(&t, &["a", "d"], &accessor),
            tab(&t),
            and(
                equal(
                    column(&t, "b", &accessor),
                    const_varchar(filter_val1.as_str()),
                ),
                equal(column(&t, "c", &accessor), const_bigint(filter_val2)),
            ),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
        exercise_verification(&verifiable_res, &ast, &accessor, &t);
        let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;

        // Calculate/compare expected result
        let (expected_a, expected_d): (Vec<_>, Vec<_>) = multizip((
            data["a"].i64_iter(),
            data["b"].string_iter(),
            data["c"].i64_iter(),
            data["d"].string_iter(),
        ))
        .filter_map(|(a, b, c, d)| {
            if b == &filter_val1 && c == &filter_val2 {
                Some((*a, d.clone()))
            } else {
                None
            }
        })
        .multiunzip();
        let expected_result = owned_table([bigint("a", expected_a), varchar("d", expected_d)]);

        assert_eq!(expected_result, res);
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
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 3, 4], &alloc),
        borrowed_bigint("b", [0, 1, 0, 1], &alloc),
        borrowed_varchar("d", ["ab", "t", "efg", "g"], &alloc),
        borrowed_bigint("c", [0, 2, 2, 0], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data.clone(), 0, ());
    let and_expr: DynProofExpr = and(
        equal(column(&t, "b", &accessor), const_int128(1)),
        equal(column(&t, "d", &accessor), const_varchar("t")),
    );
    let res = and_expr.result_evaluate(&alloc, &data);
    let expected_res = Column::Boolean(&[false, true, false, false]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_verify_a_simple_proof() {
    let alloc = Bump::new();
    let t: TableRef = "sxt.t".parse().unwrap();
    let lhs = &[true, true, false, false];
    let rhs = &[true, false, true, false];
    let table = Table::try_new(indexmap! {
        "a".into() => Column::Boolean::<TestScalar>(lhs),
        "b".into() => Column::Boolean::<TestScalar>(rhs),
    })
    .unwrap();
    let a = ColumnRef::new(t.clone(), Ident::from("a"), ColumnType::Boolean);
    let b = ColumnRef::new(t, Ident::from("b"), ColumnType::Boolean);
    let and_expr = AndExpr::new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
    );

    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(4, VecDeque::new());

    and_expr.prover_evaluate(&mut final_round_builder, &alloc, &table);

    let verification_builder = run_verify_for_each_row(
        4,
        &final_round_builder,
        3,
        |verification_builder, chi_eval, evaluation_point| {
            let accessor = indexmap! {
                a.clone() => lhs.inner_product(evaluation_point),
                b.clone() => rhs.inner_product(evaluation_point)
            };
            and_expr
                .verifier_evaluate(verification_builder, &accessor, chi_eval)
                .unwrap();
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true]; 4]
    );
}

#[test]
fn we_can_reject_a_simple_tampered_proof() {
    let alloc = Bump::new();
    let t: TableRef = "sxt.t".parse().unwrap();
    let lhs = &[true, true, false, false];
    let rhs = &[true, false, true, false];
    let a = ColumnRef::new(t.clone(), Ident::from("a"), ColumnType::Boolean);
    let b = ColumnRef::new(t, Ident::from("b"), ColumnType::Boolean);
    let and_expr = AndExpr::new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
    );

    let evaluation_points = (0..4).map(|i| {
        alloc.alloc_slice_fill_with(4, |j| {
            if i == j {
                TestScalar::ONE
            } else {
                TestScalar::ZERO
            }
        })
    });
    let zero_vec = vec![TestScalar::ZERO];
    // Tampering occurs here. All four rows return false for lhs and rhs
    let final_round_mles: Vec<_> = evaluation_points
        .clone()
        .map(|_| zero_vec.clone())
        .collect();
    let mut verification_builder = MockVerificationBuilder::new(Vec::new(), 3, final_round_mles);

    for evaluation_point in evaluation_points {
        let chi_eval = (&[1, 1, 1, 1]).inner_product(evaluation_point);
        let accessor = indexmap! {
            a.clone() => lhs.inner_product(evaluation_point),
            b.clone() => rhs.inner_product(evaluation_point)
        };
        and_expr
            .verifier_evaluate(&mut verification_builder, &accessor, chi_eval)
            .unwrap();
        verification_builder.increment_row_index();
    }
    assert_eq!(
        verification_builder.identity_subpolynomial_evaluations,
        vec![
            vec![-TestScalar::ONE],
            zero_vec.clone(),
            zero_vec.clone(),
            zero_vec
        ]
    );
}
