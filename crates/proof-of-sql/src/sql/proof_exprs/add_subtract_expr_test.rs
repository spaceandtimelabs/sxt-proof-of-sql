use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, ColumnType, OwnedTableTestAccessor, TableRef,
            TableTestAccessor,
        },
        math::decimal::Precision,
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
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

// select a, c, b + 4 as res, d from sxt.t where a - b = 3
#[test]
fn we_can_prove_a_typical_add_subtract_query() {
    let data = owned_table([
        smallint("a", [1_i16, 2, 3, 4]),
        int("b", [0_i32, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0_i64, 2, 2, 0]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![
            col_expr_plan(&t, "a", &accessor),
            col_expr_plan(&t, "c", &accessor),
            aliased_plan(add(column(&t, "b", &accessor), const_bigint(4)), "res"),
            col_expr_plan(&t, "d", &accessor),
        ],
        tab(&t),
        equal(
            subtract(column(&t, "a", &accessor), column(&t, "b", &accessor)),
            const_bigint(3),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        smallint("a", [3_i16, 4]),
        bigint("c", [2_i16, 0]),
        decimal75("res", 20, 0, [4_i64, 5]),
        varchar("d", ["efg", "g"]),
    ]);
    assert_eq!(res, expected_res);
}

// select a, a + b + c + 0.4 as c, d from sxt.t where a - b = 0.5
#[test]
fn we_can_prove_a_typical_add_subtract_query_with_decimals() {
    let data = owned_table([
        decimal75("a", 12, 1, [4_i64, 2, 2, 7]),
        decimal75("b", 12, 2, [5_i64, -15, 42, 8]),
        varchar("d", ["ab", "t", "efg", "g"]),
        decimal75("c", 12, 3, [190_i64, 27, 253, 120]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![
            col_expr_plan(&t, "a", &accessor),
            aliased_plan(
                add(
                    scaling_cast(
                        add(
                            scaling_cast(
                                add(
                                    scaling_cast(
                                        column(&t, "a", &accessor),
                                        ColumnType::Decimal75(Precision::new(13).unwrap(), 2),
                                    ),
                                    column(&t, "b", &accessor),
                                ),
                                ColumnType::Decimal75(Precision::new(15).unwrap(), 3),
                            ),
                            column(&t, "c", &accessor),
                        ),
                        ColumnType::Decimal75(Precision::new(17).unwrap(), 4),
                    ),
                    const_decimal75(2, 1, 4),
                ),
                "c",
            ),
            col_expr_plan(&t, "d", &accessor),
        ],
        tab(&t),
        equal(
            subtract(
                scaling_cast(
                    column(&t, "a", &accessor),
                    ColumnType::Decimal75(Precision::new(13).unwrap(), 2),
                ),
                column(&t, "b", &accessor),
            ),
            const_decimal75(12, 2, 35),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        decimal75("a", 12, 1, [4_i64, 2]),
        decimal75("c", 18, 3, [1040_i64, 477]),
        varchar("d", ["ab", "t"]),
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
            vec![
                col_expr_plan(&t, "d", &accessor),
                aliased_plan(
                    subtract(
                        add(column(&t, "a", &accessor), column(&t, "c", &accessor)),
                        const_int128(4),
                    ),
                    "f",
                ),
            ],
            tab(&t),
            and(
                equal(
                    column(&t, "b", &accessor),
                    const_scalar::<Curve25519Scalar, _>(filter_val1.as_str()),
                ),
                equal(
                    column(&t, "c", &accessor),
                    const_scalar::<Curve25519Scalar, _>(filter_val2),
                ),
            ),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
        exercise_verification(&verifiable_res, &ast, &accessor, &t);
        let res = verifiable_res
            .verify(&ast, &accessor, &(), &[])
            .unwrap()
            .table;

        // Calculate/compare expected result
        let (expected_f, expected_d): (Vec<_>, Vec<_>) = multizip((
            data["a"].i64_iter(),
            data["b"].string_iter(),
            data["c"].i64_iter(),
            data["d"].string_iter(),
        ))
        .filter_map(|(a, b, c, d)| {
            if b == &filter_val1 && c == &filter_val2 {
                Some((Curve25519Scalar::from(*a + *c - 4), d.clone()))
            } else {
                None
            }
        })
        .multiunzip();
        let expected_result =
            owned_table([varchar("d", expected_d), decimal75("f", 40, 0, expected_f)]);

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

// b + a - 1
#[test]
fn we_can_compute_the_correct_output_of_an_add_subtract_expr_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_smallint("a", [1_i16, 2, 3, 4], &alloc),
        borrowed_int("b", [0_i32, 1, 0, 1], &alloc),
        borrowed_varchar("d", ["ab", "t", "efg", "g"], &alloc),
        borrowed_bigint("c", [0_i64, 2, 2, 0], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data.clone(), 0, ());
    let add_subtract_expr: DynProofExpr = add(
        column(&t, "b", &accessor),
        subtract(column(&t, "a", &accessor), const_bigint(1)),
    );
    let res = add_subtract_expr
        .first_round_evaluate(&alloc, &data, &[])
        .unwrap();
    let expected_res = borrowed_decimal75("res", 21, 0, [0_i64, 2, 2, 4], &alloc).1;
    assert_eq!(res, expected_res);
}
