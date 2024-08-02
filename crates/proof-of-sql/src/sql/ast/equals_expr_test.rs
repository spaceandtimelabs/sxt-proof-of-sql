use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, Column, OwnedTable, OwnedTableTestAccessor},
        scalar::{Curve25519Scalar, Scalar},
    },
    sql::{
        ast::{test_utility::*, ProvableExpr, ProvableExprPlan},
        proof::{exercise_verification, VerifiableQueryResult},
        utils::{run_timestamp_query_test, TimestampData},
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use itertools::{multizip, MultiUnzip};
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
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
        timestamptz("f", PoSQLTimeUnit::Second, [0; 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "d", "f"], &accessor),
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [0; 0]),
        varchar("d", [""; 0]),
        timestamptz("f", PoSQLTimeUnit::Second, [0; 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_another_equality_query_with_no_rows() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [0; 0]),
        bigint("b", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 75, 0, [0; 0]),
        timestamptz("f", PoSQLTimeUnit::Second, [0; 0]),
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
        timestamptz("f", PoSQLTimeUnit::Second, [1; 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["b", "c", "e", "f"], &accessor),
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
        timestamptz("f", PoSQLTimeUnit::Second, [0; 0]),
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
        timestamptz("f", PoSQLTimeUnit::Second, [123]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["d", "a", "f"], &accessor),
        tab(t),
        equal(column(t, "b", &accessor), const_bigint(0_i64)),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        varchar("d", ["abc"]),
        bigint("a", [123_i64]),
        timestamptz("f", PoSQLTimeUnit::Second, [123]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_another_equality_query_with_a_single_selected_row() {
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123]),
        bigint("b", [123]),
        varchar("d", ["abc"]),
        decimal75("e", 75, 0, [0]),
        timestamptz("f", PoSQLTimeUnit::Second, [123]),
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
        timestamptz("f", PoSQLTimeUnit::Second, [55]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "d", "e", "f"], &accessor),
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
        timestamptz("f", PoSQLTimeUnit::Second, [0; 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_multiple_rows() {
    let timeunit = PoSQLTimeUnit::Second;
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
        timestamptz(
            "f",
            timeunit,
            vec![
                "1970-01-01T00:00:00Z",
                "1969-07-20T20:17:40Z",
                "1993-04-30T00:00:00Z",
                "1927-03-07T00:00:00Z",
            ]
            .to_timestamps(timeunit),
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
    let timeunit = PoSQLTimeUnit::Second;
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
        timestamptz(
            "f",
            timeunit,
            vec![
                "1970-01-01T00:00:00Z",
                "1969-07-20T20:17:40Z",
                "1993-04-30T00:00:00Z",
                "1927-03-07T00:00:00Z",
            ]
            .to_timestamps(timeunit),
        ),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "c", "e", "f"], &accessor),
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
        timestamptz(
            "f",
            timeunit,
            vec!["1970-01-01T00:00:00Z", "1969-07-20T20:17:40Z"].to_timestamps(timeunit),
        ),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_nonzero_comparison() {
    let timeunit = PoSQLTimeUnit::Second;
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
        timestamptz(
            "f",
            timeunit,
            vec![
                "1970-01-01T00:00:00Z",
                "1969-07-20T20:17:40Z",
                "1993-04-30T00:00:00Z",
                "1927-03-07T00:00:00Z",
                "1970-01-01T00:00:01Z",
            ]
            .to_timestamps(timeunit),
        ),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "c", "e", "f"], &accessor),
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
        timestamptz(
            "f",
            timeunit,
            vec!["1970-01-01T00:00:00Z", "1993-04-30T00:00:00Z"].to_timestamps(timeunit),
        ),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_string_comparison() {
    let timeunit = PoSQLTimeUnit::Second;
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
        timestamptz(
            "f",
            timeunit,
            vec![
                "1969-12-31T11:59:59Z",
                "1970-01-01T00:00:00Z",
                "1970-01-01T00:00:01Z",
                "1969-07-20T20:17:40Z",
                "1993-04-30T00:00:00Z",
                "1927-03-07T00:00:00Z",
            ]
            .to_timestamps(timeunit),
        ),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "b", "e", "f"], &accessor),
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
        timestamptz(
            "f",
            timeunit,
            vec!["1970-01-01T00:00:00Z", "1927-03-07T00:00:00Z"].to_timestamps(timeunit),
        ),
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
    let timeunit = PoSQLTimeUnit::Second;
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
        timestamptz(
            "f",
            timeunit,
            vec![
                "1970-01-01T00:00:00Z",
                "1969-07-20T20:17:40Z",
                "1993-04-30T00:00:00Z",
                "1970-01-01T00:00:01Z",
            ]
            .to_timestamps(timeunit),
        ),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let equals_expr: ProvableExprPlan<RistrettoPoint> = equal(
        column(t, "e", &accessor),
        const_scalar(Curve25519Scalar::ZERO),
    );
    let alloc = Bump::new();
    let res = equals_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = Column::Boolean(&[true, false, true, false]);
    assert_eq!(res, expected_res);
}

#[test]
fn test_precision_and_rounding_with_differing_precisions() {
    // Testing timestamps near rounding thresholds in nanoseconds
    let test_timestamps = vec![
        "2009-01-03T18:15:05.999999999Z",
        "2009-01-03T18:15:05.000000001Z",
    ];
    let expected_timestamps = vec!["2009-01-03T18:15:05.000000001Z"];
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.000000001Z';",
        &test_timestamps,
        PoSQLTimeUnit::Nanosecond,
        &expected_timestamps,
        PoSQLTimeUnit::Nanosecond,
    );

    // Testing timestamps near rounding thresholds in microseconds
    let test_timestamps = vec!["2009-01-03T18:15:05.999999Z", "2009-01-03T18:15:05.000000Z"];
    let expected_timestamps = vec!["2009-01-03T18:15:05.000000Z"];
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        &test_timestamps,
        PoSQLTimeUnit::Microsecond,
        &expected_timestamps,
        PoSQLTimeUnit::Microsecond,
    );

    // Testing timestamps near rounding thresholds in milliseconds
    let test_timestamps = vec!["2009-01-03T18:15:05.999Z", "2009-01-03T18:15:05.000Z"];
    let expected_timestamps = vec!["2009-01-03T18:15:05.000Z"];
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        &test_timestamps,
        PoSQLTimeUnit::Millisecond,
        &expected_timestamps,
        PoSQLTimeUnit::Millisecond,
    );

    // Test scaling a query literal to match a variety of timestamp precisions
    let test_timestamps = vec![
        "2009-01-03T18:15:05.0Z",
        "2009-01-03T18:15:05.00Z",
        "2009-01-03T18:15:05.000Z",
        "2009-01-03T18:15:05.0000Z",
        "2009-01-03T18:15:05.00000Z",
        "2009-01-03T18:15:05.000000Z",
        "2009-01-03T18:15:05.0000000Z",
        "2009-01-03T18:15:05.00000000Z",
        "2009-01-03T18:15:05.000000000Z",
        "2009-01-03T18:15:05Z",
        "2009-01-03T18:15:05.1Z",
        "2009-01-03T18:15:05.12Z",
        "2009-01-03T18:15:05.123Z",
        "2009-01-03T18:15:05.1234Z",
        "2009-01-03T18:15:05.12345Z",
        "2009-01-03T18:15:05.123456Z",
        "2009-01-03T18:15:05.1234567Z",
        "2009-01-03T18:15:05.1234568Z",
        "2009-01-03T18:15:05.12345689Z",
    ];

    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        &test_timestamps,
        PoSQLTimeUnit::Millisecond,
        &vec![
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
        ],
        PoSQLTimeUnit::Millisecond,
    );
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.123456Z';",
        &vec![
            "2009-01-03T18:15:05.0Z",
            "2009-01-03T18:15:05.00Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.0000Z",
            "2009-01-03T18:15:05.00000Z",
            "2009-01-03T18:15:05.000000Z",
            "2009-01-03T18:15:05.0000000Z",
            "2009-01-03T18:15:05.00000000Z",
            "2009-01-03T18:15:05.000000000Z",
            "2009-01-03T18:15:05Z",
            "2009-01-03T18:15:05.1Z",
            "2009-01-03T18:15:05.12Z",
            "2009-01-03T18:15:05.123Z",
            "2009-01-03T18:15:05.1234Z",
            "2009-01-03T18:15:05.12345Z",
            "2009-01-03T18:15:05.123456Z",
        ],
        PoSQLTimeUnit::Microsecond,
        &vec!["2009-01-03T18:15:05.123456Z"],
        PoSQLTimeUnit::Microsecond,
    );
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times > timestamp '2009-01-03T18:15:05.123456Z';",
        &test_timestamps,
        PoSQLTimeUnit::Nanosecond,
        &vec![
            "2009-01-03T18:15:05.1234567Z",
            "2009-01-03T18:15:05.1234568Z",
            "2009-01-03T18:15:05.12345689Z",
        ],
        PoSQLTimeUnit::Nanosecond,
    );
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times < timestamp '2009-01-03T18:15:05.123456Z';",
        &test_timestamps,
        PoSQLTimeUnit::Microsecond,
        &vec![
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05.000Z",
            "2009-01-03T18:15:05Z",
            "2009-01-03T18:15:05.1Z",
            "2009-01-03T18:15:05.12Z",
            "2009-01-03T18:15:05.123Z",
            "2009-01-03T18:15:05.1234Z",
            "2009-01-03T18:15:05.12345Z",
        ],
        PoSQLTimeUnit::Microsecond,
    );
}

#[test]
fn test_precision_and_rounding() {
    // Testing timestamps near rounding thresholds in milliseconds
    let test_timestamps = vec!["2009-01-03T18:15:05.999Z"];
    let expected_timestamps = vec!["2009-01-03T18:15:05.999Z"];
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.999Z';",
        &test_timestamps,
        PoSQLTimeUnit::Millisecond,
        &expected_timestamps,
        PoSQLTimeUnit::Millisecond,
    );

    // test microseconds
    let test_timestamps = vec!["2009-01-03T18:15:05.999999Z"];
    let expected_timestamps = vec!["2009-01-03T18:15:05.999999Z"];
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.999999Z';",
        &test_timestamps,
        PoSQLTimeUnit::Microsecond,
        &expected_timestamps,
        PoSQLTimeUnit::Microsecond,
    );

    // test nanoseconds
    let test_timestamps = vec!["2009-01-03T18:15:05.999999999Z"];
    let expected_timestamps = vec!["2009-01-03T18:15:05.999999999Z"];
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.999999999Z';",
        &test_timestamps,
        PoSQLTimeUnit::Nanosecond,
        &expected_timestamps,
        PoSQLTimeUnit::Nanosecond,
    );

    // test nanoseconds
    let test_timestamps = vec!["2009-01-03T18:15:05.999Z", "2009-01-03T18:15:05.000Z"];
    let expected_timestamps = vec!["2009-01-03T18:15:05.000Z"];
    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        &test_timestamps,
        PoSQLTimeUnit::Microsecond,
        &expected_timestamps,
        PoSQLTimeUnit::Microsecond,
    );
}

// This test simulates the following query:
//
// 1. Creating a table:
//    CREATE TABLE test_table(name VARCHAR, mytime TIMESTAMP);
//
// 2. Inserting values into the table:
//    INSERT INTO test_table(name, mytime) VALUES
//    ('a', '2009-01-03T18:15:05+03:00'),
//    ('b', '2009-01-03T18:15:05+04:00'),
//    ('c', '2009-01-03T19:15:05+03:00'),
//    ('d', '2009-01-03T19:15:05+04:00');
//
// 3. Selecting entries where the timestamp matches a specific value:
//    SELECT * FROM test_table WHERE mytime = '2009-01-03T19:15:05+04:00';
//
// This test confirms that timestamp parsing matches that of both postgresql
// and the gateway.
#[test]
fn test_timestamp_queries_match_postgresql_and_gateway() {
    let test_timestamps = vec![
        "2009-01-03T18:15:05+03:00",
        "2009-01-03T18:15:05+04:00",
        "2009-01-03T19:15:05+03:00",
        "2009-01-03T19:15:05+04:00",
    ];
    let expected_timestamps = vec!["2009-01-03T18:15:05+03:00", "2009-01-03T19:15:05+04:00"];

    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T19:15:05+04:00'",
        &test_timestamps,
        PoSQLTimeUnit::Second,
        &expected_timestamps,
        PoSQLTimeUnit::Second,
    );
}

#[test]
fn test_equality_with_variety_of_rfc3339_timestamps() {
    // Testing timestamps near rounding thresholds
    let test_timestamps = vec![
        "2009-01-03T18:15:05Z", // Bitcoin genesis block time
        "1970-01-01T00:00:00Z", // Unix epoch
        "1969-07-20T20:17:40Z", // Apollo 11 moon landing
        "1993-04-30T00:00:00Z", // World Wide Web goes live
        "1927-03-07T00:00:00Z", // Discovery of Penicillin
        "2004-02-04T00:00:00Z", // Founding of Facebook
        "2011-11-26T05:17:57Z", // Curiosity Rover lands on Mars
    ];
    let expected_timestamps = vec!["2009-01-03T18:15:05Z"];

    run_timestamp_query_test(
        "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        &test_timestamps,
        PoSQLTimeUnit::Second,
        &expected_timestamps,
        PoSQLTimeUnit::Second,
    );

    run_timestamp_query_test(
        "SELECT * FROM table WHERE times >= timestamp '1993-04-30T00:00:00Z';",
        &test_timestamps,
        PoSQLTimeUnit::Second,
        &vec![
            "2009-01-03T18:15:05Z",
            "1993-04-30T00:00:00Z",
            "2004-02-04T00:00:00Z",
            "2011-11-26T05:17:57Z",
        ],
        PoSQLTimeUnit::Second,
    );

    run_timestamp_query_test(
        "SELECT * FROM table WHERE times > timestamp '1993-04-30T00:00:00Z';",
        &test_timestamps,
        PoSQLTimeUnit::Second,
        &vec![
            "2009-01-03T18:15:05Z",
            "2004-02-04T00:00:00Z",
            "2011-11-26T05:17:57Z",
        ],
        PoSQLTimeUnit::Second,
    );

    run_timestamp_query_test(
        "SELECT * FROM table WHERE times <= timestamp '1993-04-30T00:00:00Z';",
        &test_timestamps,
        PoSQLTimeUnit::Second,
        &vec![
            "1970-01-01T00:00:00Z",
            "1969-07-20T20:17:40Z",
            "1993-04-30T00:00:00Z",
            "1927-03-07T00:00:00Z",
        ],
        PoSQLTimeUnit::Second,
    );

    run_timestamp_query_test(
        "SELECT * FROM table WHERE times < timestamp '1993-04-30T00:00:00Z';",
        &test_timestamps,
        PoSQLTimeUnit::Second,
        &vec![
            "1970-01-01T00:00:00Z",
            "1969-07-20T20:17:40Z",
            "1927-03-07T00:00:00Z",
        ],
        PoSQLTimeUnit::Second,
    );
}
