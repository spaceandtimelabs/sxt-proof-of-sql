use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, Column, OwnedTableTestAccessor},
        scalar::Curve25519Scalar,
    },
    sql::{
        ast::{test_utility::*, ProofPlan, ProvableExpr, ProvableExprPlan},
        parse::ConversionError,
        proof::{exercise_verification, QueryError, VerifiableQueryResult},
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

// select a, c, b + 4 as res, d from sxt.t where a - b = 3
#[test]
fn we_can_prove_a_typical_add_subtract_query() {
    let data = owned_table([
        smallint("a", [1_i16, 2, 3, 4]),
        int("b", [0_i32, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0_i64, 2, 2, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        vec![
            col_expr_plan(t, "a", &accessor),
            col_expr_plan(t, "c", &accessor),
            aliased_plan(add(column(t, "b", &accessor), const_bigint(4)), "res"),
            col_expr_plan(t, "d", &accessor),
        ],
        tab(t),
        equal(
            subtract(column(t, "a", &accessor), column(t, "b", &accessor)),
            const_bigint(3),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        smallint("a", [3_i16, 4]),
        bigint("c", [2_i16, 0]),
        bigint("res", [4_i64, 5]),
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        vec![
            col_expr_plan(t, "a", &accessor),
            aliased_plan(
                add(
                    add(
                        add(column(t, "a", &accessor), column(t, "b", &accessor)),
                        column(t, "c", &accessor),
                    ),
                    const_decimal75(2, 1, 4),
                ),
                "c",
            ),
            col_expr_plan(t, "d", &accessor),
        ],
        tab(t),
        equal(
            subtract(column(t, "a", &accessor), column(t, "b", &accessor)),
            const_decimal75(12, 4, 3500),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        decimal75("a", 12, 1, [4_i64, 2]),
        decimal75("c", 17, 3, [1040_i64, 477]),
        varchar("d", ["ab", "t"]),
    ]);
    assert_eq!(res, expected_res);
}

// Column type issue tests
#[test]
fn decimal_column_type_issues_error_out_when_producing_provable_ast() {
    let data = owned_table([decimal75("a", 75, 2, [1_i16, 2, 3, 4])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    assert!(matches!(
        ProvableExprPlan::try_new_add(column(t, "a", &accessor), const_bigint::<RistrettoPoint>(1)),
        Err(ConversionError::DataTypeMismatch(..))
    ));
}

// Overflow tests
// select a + b as c from sxt.t where b = 1
#[test]
fn result_expr_can_overflow() {
    let data = owned_table([
        smallint("a", [i16::MAX, i16::MIN]),
        smallint("b", [1_i16, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        vec![aliased_plan(
            add(column(t, "a", &accessor), column(t, "b", &accessor)),
            "c",
        )],
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(1)),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    assert!(matches!(
        verifiable_res.verify(&ast, &accessor, &()),
        Err(QueryError::Overflow)
    ));
}

// select a + b as c from sxt.t where b == 0
#[test]
fn overflow_in_nonselected_rows_doesnt_error_out() {
    let data = owned_table([
        smallint("a", [i16::MAX, i16::MIN + 1]),
        smallint("b", [1_i16, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        vec![aliased_plan(
            add(column(t, "a", &accessor), column(t, "b", &accessor)),
            "c",
        )],
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0)),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([smallint("c", [i16::MIN + 1])]);
    assert_eq!(res, expected_res);
}

// select a, b from sxt.t where a + b >= 0
#[test]
fn overflow_in_where_clause_doesnt_error_out() {
    let data = owned_table([bigint("a", [i64::MAX, i64::MIN]), smallint("b", [1_i16, 0])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        cols_expr_plan(t, &["a", "b"], &accessor),
        tab(t),
        gte(
            add(column(t, "a", &accessor), column(t, "b", &accessor)),
            const_bigint(0),
        ),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [i64::MAX]), smallint("b", [1_i16])]);
    assert_eq!(res, expected_res);
}

// select a + b as c, a - b as d from sxt.t
#[test]
fn result_expr_can_overflow_more() {
    let data = owned_table([
        bigint("a", [i64::MAX, i64::MIN, i64::MAX, i64::MIN]),
        bigint("b", [i64::MAX, i64::MAX, i64::MIN, i64::MIN]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        vec![
            aliased_plan(
                add(column(t, "a", &accessor), column(t, "b", &accessor)),
                "c",
            ),
            aliased_plan(
                subtract(column(t, "a", &accessor), column(t, "b", &accessor)),
                "d",
            ),
        ],
        tab(t),
        const_bool(true),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    assert!(matches!(
        verifiable_res.verify(&ast, &accessor, &()),
        Err(QueryError::Overflow)
    ));
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
        let ast = dense_filter(
            vec![
                col_expr_plan(t, "d", &accessor),
                aliased_plan(
                    subtract(
                        add(column(t, "a", &accessor), column(t, "c", &accessor)),
                        const_int128(4),
                    ),
                    "f",
                ),
            ],
            tab(t),
            and(
                equal(
                    column(t, "b", &accessor),
                    const_scalar(filter_val1.as_str()),
                ),
                equal(column(t, "c", &accessor), const_scalar(filter_val2)),
            ),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
        exercise_verification(&verifiable_res, &ast, &accessor, t);
        let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;

        // Calculate/compare expected result
        let (expected_f, expected_d): (Vec<_>, Vec<_>) = multizip((
            data["a"].i64_iter(),
            data["b"].string_iter(),
            data["c"].i64_iter(),
            data["d"].string_iter(),
        ))
        .filter_map(|(a, b, c, d)| {
            if b == &filter_val1 && c == &filter_val2 {
                Some(((*a + *c - 4) as i128, d.clone()))
            } else {
                None
            }
        })
        .multiunzip();
        let expected_result = owned_table([varchar("d", expected_d), int128("f", expected_f)]);

        assert_eq!(expected_result, res)
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
fn we_can_compute_the_correct_output_of_an_add_subtract_expr_using_result_evaluate() {
    let data = owned_table([
        smallint("a", [1_i16, 2, 3, 4]),
        int("b", [0_i32, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0_i64, 2, 2, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let add_subtract_expr: ProvableExprPlan<RistrettoPoint> = add(
        column(t, "b", &accessor),
        subtract(column(t, "a", &accessor), const_bigint(1)),
    );
    let alloc = Bump::new();
    let res = add_subtract_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res_scalar = [0, 2, 2, 4]
        .iter()
        .map(|v| Curve25519Scalar::from(*v))
        .collect::<Vec<_>>();
    let expected_res = Column::Scalar(&expected_res_scalar);
    assert_eq!(res, expected_res);
}
