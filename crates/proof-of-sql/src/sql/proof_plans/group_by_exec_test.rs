use super::test_utility::*;
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, OwnedTableTestAccessor, TableRef, TestAccessor},
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::test_utility::*,
    },
};

/// `select sum(c) as sum_c, count(*) as __count__ from sxt.t where b = 99`
#[test]
fn we_can_prove_aggregation_without_group_by() {
    let data = owned_table([
        bigint("a", [1, 2, 2, 1, 2]),
        bigint("b", [99, 99, 99, 99, 0]),
        bigint("c", [101, 102, 103, 104, 105]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = group_by(
        vec![],
        vec![sum_expr(column(&t, "c", &accessor), "sum_c")],
        "__count__",
        tab(&t),
        equal(column(&t, "b", &accessor), const_int128(99)),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("sum_c", [101 + 104 + 102 + 103]),
        bigint("__count__", [4]),
    ]);
    assert_eq!(res, expected);
}

/// `select a, sum(c) as sum_c, count(*) as __count__ from sxt.t where b = 99 group by a`
#[test]
fn we_can_prove_a_simple_group_by_with_bigint_columns() {
    let data = owned_table([
        bigint("a", [1, 2, 2, 1, 2]),
        bigint("b", [99, 99, 99, 99, 0]),
        bigint("c", [101, 102, 103, 104, 105]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = group_by(
        cols_expr(&t, &["a"], &accessor),
        vec![sum_expr(column(&t, "c", &accessor), "sum_c")],
        "__count__",
        tab(&t),
        equal(column(&t, "b", &accessor), const_int128(99)),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("a", [1, 2]),
        bigint("sum_c", [101 + 104, 102 + 103]),
        bigint("__count__", [2, 2]),
    ]);
    assert_eq!(res, expected);
}

/// `select a, sum(c * 2 + 1) as sum_c, count(*) as __count__ from sxt.t where b = 99 group by a`
#[test]
fn we_can_prove_a_group_by_with_bigint_columns() {
    let data = owned_table([
        bigint("a", [1, 2, 2, 1, 2]),
        bigint("b", [99, 99, 99, 99, 0]),
        bigint("c", [101, 102, 103, 104, 105]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = group_by(
        cols_expr(&t, &["a"], &accessor),
        vec![sum_expr(
            add(
                multiply(column(&t, "c", &accessor), const_bigint(2)),
                const_bigint(1),
            ),
            "sum_c",
        )],
        "__count__",
        tab(&t),
        equal(column(&t, "b", &accessor), const_int128(99)),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("a", [1, 2]),
        bigint("sum_c", [(101 + 104) * 2 + 2, (102 + 103) * 2 + 2]),
        bigint("__count__", [2, 2]),
    ]);
    assert_eq!(res, expected);
}

#[expect(clippy::too_many_lines)]
#[test]
fn we_can_prove_a_complex_group_by_query_with_many_columns() {
    let scalar_filter_data: Vec<Curve25519Scalar> = [
        333, 222, 222, 333, 222, 333, 333, 333, 222, 222, 222, 333, 222, 222, 222, 222, 222, 222,
        333, 333,
    ]
    .iter()
    .map(core::convert::Into::into)
    .collect();
    let scalar_group_data: Vec<Curve25519Scalar> =
        [5, 4, 5, 4, 4, 4, 5, 4, 4, 4, 5, 4, 4, 4, 5, 4, 4, 4, 4, 5]
            .iter()
            .map(core::convert::Into::into)
            .collect();
    let scalar_sum_data: Vec<Curve25519Scalar> = [
        119, 522, 100, 325, 501, 447, 759, 375, 212, 532, 459, 616, 579, 179, 695, 963, 532, 868,
        331, 830,
    ]
    .iter()
    .map(core::convert::Into::into)
    .collect();
    let data = owned_table([
        bigint(
            "bigint_filter",
            [
                30, 20, 30, 30, 30, 20, 30, 20, 30, 20, 30, 20, 20, 20, 30, 30, 20, 20, 20, 30,
            ],
        ),
        bigint(
            "bigint_group",
            [7, 6, 6, 6, 7, 7, 6, 6, 6, 6, 7, 7, 6, 7, 6, 7, 7, 7, 6, 7],
        ),
        bigint(
            "bigint_sum",
            [
                834, 985, 832, 300, 146, 624, 553, 637, 770, 574, 913, 600, 336, 984, 198, 257,
                781, 196, 537, 358,
            ],
        ),
        int128(
            "int128_filter",
            [
                1030, 1030, 1030, 1020, 1020, 1030, 1020, 1020, 1020, 1030, 1030, 1030, 1020, 1020,
                1030, 1020, 1020, 1030, 1020, 1030,
            ],
        ),
        int128(
            "int128_group",
            [8, 8, 8, 8, 8, 8, 9, 9, 8, 9, 8, 9, 8, 9, 8, 9, 8, 8, 8, 8],
        ),
        int128(
            "int128_sum",
            [
                275, 225, 315, 199, 562, 578, 563, 513, 634, 829, 613, 295, 509, 923, 133, 973,
                700, 464, 622, 943,
            ],
        ),
        varchar(
            "varchar_filter",
            [
                "f2", "f2", "f3", "f2", "f2", "f3", "f3", "f2", "f2", "f3", "f2", "f2", "f2", "f3",
                "f2", "f3", "f2", "f2", "f3", "f3",
            ],
        ),
        varchar(
            "varchar_group",
            [
                "g1", "g2", "g1", "g1", "g1", "g1", "g2", "g1", "g1", "g1", "g2", "g2", "g1", "g1",
                "g1", "g2", "g1", "g2", "g1", "g1",
            ],
        ),
        scalar("scalar_filter", scalar_filter_data),
        scalar("scalar_group", scalar_group_data),
        scalar("scalar_sum", scalar_sum_data),
    ]);

    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    // SELECT scalar_group, int128_group, bigint_group, sum(bigint_sum + 1) as sum_int, sum(bigint_sum - int128_sum) as sum_bigint, sum(scalar_filter) as sum_scal, count(*) as __count__
    //  FROM sxt.t WHERE int128_filter = 1020 AND varchar_filter = 'f2'
    //  GROUP BY scalar_group, int128_group, bigint_group
    let expr = group_by(
        cols_expr(
            &t,
            &["scalar_group", "int128_group", "bigint_group"],
            &accessor,
        ),
        vec![
            sum_expr(
                add(column(&t, "bigint_sum", &accessor), const_bigint(1)),
                "sum_int",
            ),
            sum_expr(
                subtract(
                    column(&t, "bigint_sum", &accessor),
                    column(&t, "int128_sum", &accessor),
                ),
                "sum_128",
            ),
            sum_expr(column(&t, "scalar_sum", &accessor), "sum_scal"),
        ],
        "__count__",
        tab(&t),
        and(
            equal(column(&t, "int128_filter", &accessor), const_int128(1020)),
            equal(column(&t, "varchar_filter", &accessor), const_varchar("f2")),
        ),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        scalar("scalar_group", [4, 4, 4]),
        int128("int128_group", [8, 8, 9]),
        bigint("bigint_group", [6, 7, 6]),
        bigint("sum_int", [1409, 929, 638]),
        int128("sum_128", [64, -335, 124]),
        scalar("sum_scal", [1116, 1033, 375]),
        bigint("__count__", [3, 2, 1]),
    ]);
    assert_eq!(res, expected);

    // SELECT sum(bigint_sum) as sum_int, sum(int128_sum * 4) as sum_128, sum(scalar_sum) as sum_scal, count(*) as __count__
    //  FROM sxt.t WHERE int128_filter = 1020 AND varchar_filter = 'f2'
    let expr = group_by(
        vec![],
        vec![
            sum_expr(column(&t, "bigint_sum", &accessor), "sum_int"),
            sum_expr(
                multiply(column(&t, "int128_sum", &accessor), const_bigint(4)),
                "sum_128",
            ),
            sum_expr(column(&t, "scalar_sum", &accessor), "sum_scal"),
        ],
        "__count__",
        tab(&t),
        and(
            equal(column(&t, "int128_filter", &accessor), const_int128(1020)),
            equal(column(&t, "varchar_filter", &accessor), const_varchar("f2")),
        ),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("sum_int", [1406 + 927 + 637]),
        int128("sum_128", [(1342 + 1262 + 513) * 4]),
        scalar("sum_scal", [1116 + 1033 + 375]),
        bigint("__count__", [3 + 2 + 1]),
    ]);
    assert_eq!(res, expected);
}
