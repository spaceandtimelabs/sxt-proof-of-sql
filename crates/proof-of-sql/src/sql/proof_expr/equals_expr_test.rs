use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, Column, OwnedTable, OwnedTableTestAccessor},
        scalar::{Curve25519Scalar, Scalar},
    },
    sql::{
        ast::{test_utility::*, DynProofExpr, ProofExpr},
        proof::{exercise_verification, VerifiableQueryResult},
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use itertools::{multizip, MultiUnzip};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

#[test]
fn we_can_prove_an_equality_query_with_no_rows() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [0; 0]),
        bigint("b", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "d"], &accessor),
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [0; 0]), varchar("d", [""; 0])]);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "d"], &accessor),
        tab(t),
        equal(column(t, "a", &accessor), column(t, "b", &accessor)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [0; 0]), varchar("d", [""; 0])]);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["b", "c", "e"], &accessor),
        tab(t),
        equal(
            column(t, "bool", &accessor),
            equal(column(t, "a", &accessor), column(t, "b", &accessor)),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("b", [1; 0]),
        varchar("c", ["t"; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_selected_row() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123]),
        bigint("b", [0]),
        varchar("d", ["abc"]),
        decimal75("e", 75, 0, [0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["d", "a"], &accessor),
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([varchar("d", ["abc"]), bigint("a", [123_i64])]);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["d", "a"], &accessor),
        tab(t),
        equal(column(t, "a", &accessor), column(t, "b", &accessor)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([varchar("d", ["abc"]), bigint("a", [123_i64])]);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "d", "e"], &accessor),
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);
    assert_eq!(res, expected_res);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "c", "e"], &accessor),
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [1, 3]),
        varchar("c", ["t", "jj"]),
        decimal75("e", 75, 0, [0, 2]),
    ]);
    assert_eq!(res, expected_res);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "c", "e"], &accessor),
        tab(t),
        equal(
            column(t, "bool", &accessor),
            equal(column(t, "a", &accessor), column(t, "b", &accessor)),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [1, 2]),
        varchar("c", ["t", "ghi"]),
        decimal75("e", 75, 0, [0, 1]),
    ]);
    assert_eq!(res, expected_res);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "c", "e"], &accessor),
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(123_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [1, 3]),
        varchar("c", ["t", "jj"]),
        decimal75("e", 42, 10, vec![0, 2]),
    ]);
    assert_eq!(res, expected_res);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "b", "e"], &accessor),
        tab(t),
        equal(column(t, "c", &accessor), const_varchar("ghi")),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [2, 5]),
        bigint("b", [5, 0]),
        decimal75("e", 42, 10, [1, -1]),
    ]);
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
        let filter_val = format!("s{}", dist.sample(&mut rng));

        // Create and verify proof
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t,
            data.clone(),
            offset,
            (),
        );
        let ast = dense_filter(
            cols_expr_plan(t, &["a", "d"], &accessor),
            tab(t),
            equal(
                column(t, "b", &accessor),
                const_varchar(filter_val.as_str()),
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
        .filter_map(|(a, b, _c, d)| {
            if b == &filter_val {
                Some((*a, d.clone()))
            } else {
                None
            }
        })
        .multiunzip();
        let expected_result = owned_table([bigint("a", expected_a), varchar("d", expected_d)]);

        assert_eq!(expected_result, res)
    }
}

#[test]
fn we_can_query_random_tables_using_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_using_a_non_zero_offset() {
    test_random_tables_with_given_offset(121);
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let equals_expr: DynProofExpr<RistrettoPoint> = equal(
        column(t, "e", &accessor),
        const_scalar(Curve25519Scalar::ZERO),
    );
    let alloc = Bump::new();
    let res = equals_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = Column::Boolean(&[true, false, true, false]);
    assert_eq!(res, expected_res);
}
