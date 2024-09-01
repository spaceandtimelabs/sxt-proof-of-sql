use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, Column, OwnedTableTestAccessor},
        scalar::Curve25519Scalar,
    },
    sql::{
        ast::{test_utility::*, DynProofExpr, ProofPlan, ProvableExpr},
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

// select a * 2 as a, c, b * 4.5 as b, d * 3  + 4.7 as d, e from sxt.t where d * 3.9 = 8.19
#[test]
fn we_can_prove_a_typical_multiply_query() {
    let data = owned_table([
        smallint("a", [1_i16, 2, 3, 4]),
        int("b", [0_i32, 1, 2, 1]),
        varchar("e", ["ab", "t", "efg", "g"]),
        bigint("c", [0_i64, 2, 2, 0]),
        decimal75("d", 2, 1, [21_i64, 4, 21, -7]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        vec![
            aliased_plan(multiply(column(t, "a", &accessor), const_int(2)), "a"),
            col_expr_plan(t, "c", &accessor),
            aliased_plan(
                multiply(column(t, "b", &accessor), const_decimal75(2, 1, 45)),
                "b",
            ),
            aliased_plan(
                add(
                    multiply(column(t, "d", &accessor), const_smallint(3)),
                    const_decimal75(2, 1, 47),
                ),
                "d",
            ),
            col_expr_plan(t, "e", &accessor),
        ],
        tab(t),
        equal(
            multiply(column(t, "d", &accessor), const_decimal75(2, 1, 39)),
            const_decimal75(3, 2, 819),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        int("a", [2_i32, 6]),
        bigint("c", [0_i64, 2]),
        decimal75("b", 13, 1, [0_i64, 90]),
        decimal75("d", 9, 1, [110_i64, 110]),
        varchar("e", ["ab", "efg"]),
    ]);
    assert_eq!(res, expected_res);
}

// Column type issue tests
#[test]
fn decimal_column_type_issues_error_out_when_producing_provable_ast() {
    let data = owned_table([decimal75("a", 57, 2, [1_i16, 2, 3, 4])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    assert!(matches!(
        DynProofExpr::try_new_multiply(
            column(t, "a", &accessor),
            const_bigint::<RistrettoPoint>(1)
        ),
        Err(ConversionError::DataTypeMismatch(..))
    ));
}

// Overflow tests
// select a * b as c from sxt.t where b = 2
#[test]
fn result_expr_can_overflow() {
    let data = owned_table([
        smallint("a", [i16::MAX, i16::MIN]),
        smallint("b", [2_i16, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        vec![aliased_plan(
            multiply(column(t, "a", &accessor), column(t, "b", &accessor)),
            "c",
        )],
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(2)),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    assert!(matches!(
        verifiable_res.verify(&ast, &accessor, &()),
        Err(QueryError::Overflow)
    ));
}

// select a * b as c from sxt.t where b == 0
#[test]
fn overflow_in_nonselected_rows_doesnt_error_out() {
    let data = owned_table([
        smallint("a", [i16::MAX, i16::MIN + 1]),
        smallint("b", [2_i16, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        vec![aliased_plan(
            multiply(column(t, "a", &accessor), column(t, "b", &accessor)),
            "c",
        )],
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0)),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([smallint("c", [0_i16])]);
    assert_eq!(res, expected_res);
}

// select a, b from sxt.t where a * b >= 0
#[test]
fn overflow_in_where_clause_doesnt_error_out() {
    let data = owned_table([
        bigint("a", [i64::MAX, i64::MIN + 1]),
        smallint("b", [2_i16, 1]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        cols_expr_plan(t, &["a", "b"], &accessor),
        tab(t),
        gte(
            multiply(column(t, "a", &accessor), column(t, "b", &accessor)),
            const_bigint(0),
        ),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [i64::MAX]), smallint("b", [2_i16])]);
    assert_eq!(res, expected_res);
}

// select a * b as c from sxt.t
#[test]
fn result_expr_can_overflow_more() {
    let data = owned_table([
        bigint("a", [i64::MAX, i64::MIN, i64::MAX, i64::MIN]),
        bigint("b", [i64::MAX, i64::MAX, i64::MIN, i64::MIN]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        vec![aliased_plan(
            multiply(column(t, "a", &accessor), column(t, "b", &accessor)),
            "c",
        )],
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

// select * from sxt.t where a * b * c * d * e = res
// Only the last row is a valid result
// The other two are due to the fact that scalars are elements of finite fields
// and that hence scalar multiplication inherently wraps around
#[test]
fn where_clause_can_wrap_around() {
    let data = owned_table([
        bigint("a", [2357878470324616199_i64, 2657439699204141, 884]),
        bigint("b", [31194601778911687_i64, 1644425323726039, 884]),
        bigint("c", [500213946116239_i64, 1570568673569987, 884]),
        bigint("d", [211980999383887_i64, 1056107792886999, 884]),
        bigint("e", [927908842441_i64, 998426626609497, 884]),
        bigint("res", [-20_i64, 50, 539835356263424]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast: ProofPlan<RistrettoPoint> = dense_filter(
        cols_expr_plan(t, &["a", "b", "c", "d", "e", "res"], &accessor),
        tab(t),
        equal(
            multiply(
                multiply(
                    multiply(
                        multiply(column(t, "a", &accessor), column(t, "b", &accessor)),
                        column(t, "c", &accessor),
                    ),
                    column(t, "d", &accessor),
                ),
                column(t, "e", &accessor),
            ),
            column(t, "res", &accessor),
        ),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [2357878470324616199_i64, 2657439699204141, 884]),
        bigint("b", [31194601778911687_i64, 1644425323726039, 884]),
        bigint("c", [500213946116239_i64, 1570568673569987, 884]),
        bigint("d", [211980999383887_i64, 1056107792886999, 884]),
        bigint("e", [927908842441_i64, 998426626609497, 884]),
        bigint("res", [-20_i64, 50, 539835356263424]),
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
                    add(
                        multiply(column(t, "a", &accessor), column(t, "c", &accessor)),
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
                Some(((*a * *c + 4) as i128, d.clone()))
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
    test_random_tables_with_given_offset(23);
}

// b * (a - 1.5)
#[test]
fn we_can_compute_the_correct_output_of_a_multiply_expr_using_result_evaluate() {
    let data = owned_table([
        smallint("a", [1_i16, 2, 3, 4]),
        int("b", [0_i32, 1, 5, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0_i64, 2, 2, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let arithmetic_expr: DynProofExpr<RistrettoPoint> = multiply(
        column(t, "b", &accessor),
        subtract(column(t, "a", &accessor), const_decimal75(2, 1, 15)),
    );
    let alloc = Bump::new();
    let res = arithmetic_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res_scalar = [0, 5, 75, 25]
        .iter()
        .map(|v| Curve25519Scalar::from(*v))
        .collect::<Vec<_>>();
    let expected_res = Column::Scalar(&expected_res_scalar);
    assert_eq!(res, expected_res);
}
