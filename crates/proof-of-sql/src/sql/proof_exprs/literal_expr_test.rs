use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, Column, OwnedTableTestAccessor},
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::test_utility::*,
        proof_plans::test_utility::*,
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
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
        let lit = dist.sample(&mut rng) < 0;

        // Create and verify proof
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t,
            data.clone(),
            offset,
            (),
        );
        let ast = filter(
            cols_expr_plan(t, &["a", "b", "c"], &accessor),
            tab(t),
            const_bool(lit),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
        exercise_verification(&verifiable_res, &ast, &accessor, t);
        let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;

        // Calculate/compare expected result
        let (expected_a, expected_b, expected_c): (Vec<bool>, Vec<String>, Vec<i64>) = if lit {
            (
                data["a"].bool_iter().copied().collect(),
                data["b"].string_iter().cloned().collect(),
                data["c"].i64_iter().copied().collect(),
            )
        } else {
            (vec![], vec![], vec![])
        };
        let expected_result = owned_table([
            boolean("a", expected_a),
            varchar("b", expected_b),
            bigint("c", expected_c),
        ]);

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
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = owned_table([bigint("a", [123_i64])]);
    let expected_res = data.clone();
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a"], &accessor),
        tab(t),
        const_bool(true),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_query_with_a_single_non_selected_row() {
    let data = owned_table([bigint("a", [123_i64])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a"], &accessor),
        tab(t),
        const_bool(false),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [1_i64; 0])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_output_of_a_literal_expr_using_result_evaluate() {
    let data = owned_table([bigint("a", [123_i64, 456, 789, 1011])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let literal_expr: DynProofExpr<RistrettoPoint> = const_bool(true);
    let alloc = Bump::new();
    let res = literal_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = Column::Boolean(&[true, true, true, true]);
    assert_eq!(res, expected_res);
}
