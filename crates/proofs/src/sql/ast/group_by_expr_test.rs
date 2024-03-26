use super::test_utility::{and, cols_expr, equal, group_by, sums_expr, tab};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{ColumnType, OwnedTableTestAccessor, TestAccessor},
        scalar::Curve25519Scalar,
    },
    owned_table,
    sql::proof::{exercise_verification, VerifiableQueryResult},
};

#[test]
fn we_can_prove_a_simple_group_by_with_bigint_columns() {
    let data = owned_table!(
        "a" => [1_i64, 2, 2, 1, 2],
        "b" => [99_i64, 99, 99, 99, 0],
        "c" => [101_i64, 102, 103, 104, 105],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = group_by(
        cols_expr(t, &["a"], &accessor),
        sums_expr(t, &["c"], &["sum_c"], &[ColumnType::BigInt], &accessor),
        "__count__",
        tab(t),
        equal(t, "b", 99, &accessor),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table!(
        "a" => [1_i64, 2],
        "sum_c" => [101_i64+104, 102+103],
        "__count__" => [2_i64,2],
    );
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_a_complex_group_by_query_with_many_columns() {
    let scalar_filter_data: Vec<Curve25519Scalar> = [
        333, 222, 222, 333, 222, 333, 333, 333, 222, 222, 222, 333, 222, 222, 222, 222, 222, 222,
        333, 333,
    ]
    .iter()
    .map(|i| i.into())
    .collect();
    let scalar_group_data: Vec<Curve25519Scalar> =
        [5, 4, 5, 4, 4, 4, 5, 4, 4, 4, 5, 4, 4, 4, 5, 4, 4, 4, 4, 5]
            .iter()
            .map(|i| i.into())
            .collect();
    let scalar_sum_data: Vec<Curve25519Scalar> = [
        119, 522, 100, 325, 501, 447, 759, 375, 212, 532, 459, 616, 579, 179, 695, 963, 532, 868,
        331, 830,
    ]
    .iter()
    .map(|i| i.into())
    .collect();
    let data = owned_table!(
        "bigint_filter" => [30_i64, 20, 30, 30, 30, 20, 30, 20, 30, 20, 30, 20, 20, 20, 30, 30, 20, 20, 20, 30],
        "bigint_group" => [7_i64, 6, 6, 6, 7, 7, 6, 6, 6, 6, 7, 7, 6, 7, 6, 7, 7, 7, 6, 7],
        "bigint_sum" => [834_i64, 985, 832, 300, 146, 624, 553, 637, 770, 574, 913, 600, 336, 984, 198, 257, 781, 196, 537, 358],
        "int128_filter" => [1030_i128, 1030, 1030, 1020, 1020, 1030, 1020, 1020, 1020, 1030, 1030, 1030, 1020, 1020, 1030, 1020, 1020, 1030, 1020, 1030],
        "int128_group" => [8_i128, 8, 8, 8, 8, 8, 9, 9, 8, 9, 8, 9, 8, 9, 8, 9, 8, 8, 8, 8],
        "int128_sum" => [275_i128, 225, 315, 199, 562, 578, 563, 513, 634, 829, 613, 295, 509, 923, 133, 973, 700, 464, 622, 943],
        "varchar_filter" => ["f2", "f2", "f3", "f2", "f2", "f3", "f3", "f2", "f2", "f3", "f2", "f2", "f2", "f3", "f2", "f3", "f2", "f2", "f3", "f3"],
        "varchar_group" => ["g1", "g2", "g1", "g1", "g1", "g1", "g2", "g1", "g1", "g1", "g2", "g2", "g1", "g1", "g1", "g2", "g1", "g2", "g1", "g1"],
        "scalar_filter" => scalar_filter_data,
        "scalar_group" => scalar_group_data,
        "scalar_sum" => scalar_sum_data,
    );

    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);

    // SELECT scalar_group, int128_group, bigint_group, sum(int128_filter) as sum_int, sum(bigint_filter) as sum_bigint, sum(scalar_filter) as sum_scal, count(*) as __count__
    //  FROM sxt.t WHERE int128_filter = 1020 AND varchar_filter = 'f2'
    //  GROUP BY scalar_group, int128_group, bigint_group
    let expr = group_by(
        cols_expr(
            t,
            &["scalar_group", "int128_group", "bigint_group"],
            &accessor,
        ),
        sums_expr(
            t,
            &["bigint_sum", "int128_sum", "scalar_sum"],
            &["sum_int", "sum_128", "sum_scal"],
            &[ColumnType::BigInt, ColumnType::Int128, ColumnType::Scalar],
            &accessor,
        ),
        "__count__",
        tab(t),
        and(
            equal(t, "int128_filter", 1020, &accessor),
            equal(t, "varchar_filter", "f2", &accessor),
        ),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table!(
        "scalar_group" => [Curve25519Scalar::from(4), Curve25519Scalar::from(4), Curve25519Scalar::from(4)],
        "int128_group" => [8_i128, 8, 9],
        "bigint_group" => [6_i64, 7, 6],
        "sum_int" => [1406_i64, 927, 637],
        "sum_128" => [1342_i128, 1262, 513],
        "sum_scal" => [Curve25519Scalar::from(1116), Curve25519Scalar::from(1033), Curve25519Scalar::from(375)],
        "__count__" => [3_i64, 2, 1],
    );
    assert_eq!(res, expected);

    // SELECT sum(int128_filter) as sum_int, sum(bigint_filter) as sum_bigint, sum(scalar_filter) as sum_scal, count(*) as __count__
    //  FROM sxt.t WHERE int128_filter = 1020 AND varchar_filter = 'f2'
    let expr = group_by(
        vec![],
        sums_expr(
            t,
            &["bigint_sum", "int128_sum", "scalar_sum"],
            &["sum_int", "sum_128", "sum_scal"],
            &[ColumnType::BigInt, ColumnType::Int128, ColumnType::Scalar],
            &accessor,
        ),
        "__count__",
        tab(t),
        and(
            equal(t, "int128_filter", 1020, &accessor),
            equal(t, "varchar_filter", "f2", &accessor),
        ),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table!(
        "sum_int" => [1406_i64 + 927 + 637],
        "sum_128" => [1342_i128 + 1262 + 513],
        "sum_scal" => [Curve25519Scalar::from(1116) + Curve25519Scalar::from(1033) + Curve25519Scalar::from(375)],
        "__count__" => [3_i64 + 2 + 1],
    );
    assert_eq!(res, expected);
}
