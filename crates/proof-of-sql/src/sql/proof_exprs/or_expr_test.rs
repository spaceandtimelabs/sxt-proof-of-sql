use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, Column, OwnedTableTestAccessor, TestAccessor},
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::{test_utility::*, DynProofExpr, ProofExpr},
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

#[test]
fn we_can_prove_a_simple_or_query() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4]),
        varchar("d", ["ab", "t", "g", "efg"]),
        bigint("b", [0_i64, 1, 0, 2]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a", "d"], &accessor),
        tab(t),
        or(
            equal(column(t, "b", &accessor), const_bigint(1)),
            equal(column(t, "d", &accessor), const_varchar("g")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [2_i64, 3]), varchar("d", ["t", "g"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_simple_or_query_with_variable_integer_types() {
    let data = owned_table([
        int128("a", [1_i128, 2, 3, 4]),
        varchar("d", ["ab", "t", "g", "efg"]),
        smallint("b", [0_i16, 1, 0, 2]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a", "d"], &accessor),
        tab(t),
        or(
            equal(column(t, "b", &accessor), const_bigint(1)),
            equal(column(t, "d", &accessor), const_varchar("g")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([int128("a", [2_i64, 3]), varchar("d", ["t", "g"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_or_query_where_both_lhs_and_rhs_are_true() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4]),
        int128("b", [0_i128, 1, 1, 1]),
        int("c", [0_i32, 2, 2, 0]),
        varchar("d", ["ab", "t", "g", "efg"]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = filter(
        cols_expr_plan(t, &["a", "d"], &accessor),
        tab(t),
        or(
            equal(column(t, "b", &accessor), const_bigint(1)),
            equal(column(t, "d", &accessor), const_varchar("g")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [2_i64, 3, 4]), varchar("d", ["t", "g", "efg"])]);
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
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t,
            data.clone(),
            offset,
            (),
        );
        let ast = filter(
            cols_expr_plan(t, &["a", "d"], &accessor),
            tab(t),
            or(
                equal(
                    column(t, "b", &accessor),
                    const_varchar(filter_val1.as_str()),
                ),
                equal(column(t, "c", &accessor), const_bigint(filter_val2)),
            ),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
        exercise_verification(&verifiable_res, &ast, &accessor, t);
        let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;

        // Calculate/compare expected result
        let (expected_a, expected_d): (Vec<_>, Vec<_>) = multizip((
            data["a"].i64_iter(),
            data["b"].string_iter(),
            data["c"].i64_iter(),
            data["d"].string_iter(),
        ))
        .filter_map(|(a, b, c, d)| {
            if b == &filter_val1 || c == &filter_val2 {
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
fn we_can_query_random_tables_with_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    test_random_tables_with_given_offset(1001);
}

#[test]
fn we_can_compute_the_correct_output_of_an_or_expr_using_result_evaluate() {
    let data = owned_table([
        bigint("a", [1, 2, 3, 4]),
        bigint("b", [0, 1, 0, 1]),
        bigint("c", [0, 2, 2, 0]),
        varchar("d", ["ab", "t", "g", "efg"]),
    ]);
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let and_expr: DynProofExpr = or(
        equal(column(t, "b", &accessor), const_int128(1)),
        equal(column(t, "d", &accessor), const_varchar("g")),
    );
    let alloc = Bump::new();
    let res = and_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = Column::Boolean(&[false, true, true, true]);
    assert_eq!(res, expected_res);
}
