use crate::{
    base::{
        database::{owned_table_utility::*, OwnedTable},
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::postprocessing::{apply_postprocessing_steps, test_utility::*, OwnedTablePostprocessing},
};
use rand::{seq::SliceRandom, Rng};
use sqlparser::ast::Ident;

#[test]
fn we_can_transform_a_result_using_a_single_order_by_in_ascending_direction() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("c", [1_i64, -5, i64::MAX]),
        varchar("a", ["a", "d", "b"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[1_usize], &[true])];
    let expected_table: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("c", [1_i64, i64::MAX, -5]),
        varchar("a", ["a", "b", "d"]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_transform_a_result_using_a_single_order_by_in_descending_direction() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("c", [1_i128, i128::MIN, i128::MAX]),
        varchar("a", ["a", "d", "b"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[0_usize], &[false])];
    let expected_table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("c", [i128::MAX, 1, i128::MIN]),
        varchar("a", ["b", "a", "d"]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_transform_a_result_ordering_by_the_first_column_then_the_second_column() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [123_i32, 342, i32::MIN, i32::MAX, 123, 34]),
        varchar("d", ["alfa", "beta", "abc", "f", "kl", "f"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[0_usize, 1], &[false, false])];
    let expected_table: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [i32::MAX, 342, 123, 123, 34, i32::MIN]),
        varchar("d", ["f", "beta", "kl", "alfa", "f", "abc"]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_transform_a_result_ordering_by_the_second_column_then_the_first_column() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        smallint("a", [123_i16, 342, -234, i16::MAX, 123, i16::MIN]),
        varchar("d", ["alfa", "beta", "abc", "f", "kl", "f"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[1_usize, 0], &[false, true])];
    let expected_table: OwnedTable<Curve25519Scalar> = owned_table([
        smallint("a", [123_i16, i16::MIN, i16::MAX, 342, 123, -234]),
        varchar("d", ["kl", "f", "f", "beta", "alfa", "abc"]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_use_int128_columns_inside_order_by_in_desc_order() {
    let s = [
        -1_i128,
        1,
        i128::MIN + 1,
        i128::MAX,
        0,
        -2,
        i128::MIN,
        -3,
        i128::MIN,
        -1,
        -3,
        1,
        -i128::MAX,
        11,
        i128::MAX,
    ];

    let table: OwnedTable<Curve25519Scalar> = owned_table([int128("h", s), int128("j", s)]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[1_usize, 0], &[false, true])];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();

    let mut sorted_s = s;
    sorted_s.sort_unstable();
    let reverse_sorted_s = sorted_s.into_iter().rev().collect::<Vec<_>>();

    let expected_table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("h", reverse_sorted_s.clone()),
        int128("j", reverse_sorted_s),
    ]);
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_use_int128_columns_inside_order_by_in_asc_order() {
    let s = [
        -1_i128,
        1,
        i128::MIN + 1,
        i128::MAX,
        0,
        -2,
        i128::MIN,
        -3,
        i128::MIN,
        -1,
        -3,
        1,
        -i128::MAX,
        11,
        i128::MAX,
    ];

    let table: OwnedTable<Curve25519Scalar> = owned_table([int128("h", s), int128("j", s)]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[1_usize, 0], &[true, false])];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();

    let mut sorted_s = s;
    sorted_s.sort_unstable();

    let expected_table: OwnedTable<Curve25519Scalar> =
        owned_table([int128("h", sorted_s), int128("j", sorted_s)]);
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_do_order_by_with_random_i128_data() {
    let mut rng = rand::thread_rng();
    let range: Vec<i128> = (-300_000..300_000).collect();
    let table: Vec<i128> = range
        .iter()
        .map(|_| rng.gen_range(i128::MIN..i128::MAX))
        .chain(range.clone())
        .collect();

    let (shuffled_data, sorted_data) = {
        let mut shuffled_s = table.clone();
        shuffled_s.shuffle(&mut rng);
        let mut sorted_s = table.clone();
        sorted_s.sort_unstable();
        (shuffled_s, sorted_s)
    };

    let table: OwnedTable<Curve25519Scalar> = owned_table([int128("h", shuffled_data)]);
    let expected_table: OwnedTable<Curve25519Scalar> = owned_table([int128("h", sorted_data)]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[0_usize], &[true])];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_order_with_nulls_in_ascending_order() {
    let mut table: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [1, 2, 3, 4, 5]),
        varchar("b", ["a", "b", "c", "d", "e"]),
    ]);

    let presence = vec![true, false, true, false, true];
    table.set_presence(Ident::new("a"), presence);

    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[0], &[true])];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    let mut expected: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [2, 4, 1, 3, 5]),
        varchar("b", ["b", "d", "a", "c", "e"]),
    ]);

    let expected_presence = vec![false, false, true, true, true];
    expected.set_presence(Ident::new("a"), expected_presence);

    assert_eq!(actual_table, expected);
}

#[test]
fn we_can_order_with_nulls_in_descending_order() {
    let mut table: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [1, 2, 3, 4, 5]),
        varchar("b", ["a", "b", "c", "d", "e"]),
    ]);

    let presence = vec![true, false, true, false, true];
    table.set_presence(Ident::new("a"), presence);

    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[0], &[false])];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    let mut expected: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [5, 3, 1, 2, 4]),
        varchar("b", ["e", "c", "a", "b", "d"]),
    ]);

    let expected_presence = vec![true, true, true, false, false];
    expected.set_presence(Ident::new("a"), expected_presence);

    assert_eq!(actual_table, expected);
}

#[test]
fn we_can_order_with_nulls_in_multiple_columns() {
    let mut table: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [1, 1, 2, 2, 3]),
        varchar("b", ["x", "y", "z", "w", "v"]),
    ]);

    let presence_a = vec![true, true, false, true, false];
    let presence_b = vec![true, false, true, false, true];
    table.set_presence(Ident::new("a"), presence_a);
    table.set_presence(Ident::new("b"), presence_b);

    let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[0, 1], &[true, false])];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    let mut expected: OwnedTable<Curve25519Scalar> = owned_table([
        int("a", [2, 3, 1, 1, 2]),
        varchar("b", ["z", "v", "x", "y", "w"]),
    ]);

    let expected_presence_a = vec![false, false, true, true, true];
    let expected_presence_b = vec![true, true, true, false, false];
    expected.set_presence(Ident::new("a"), expected_presence_a);
    expected.set_presence(Ident::new("b"), expected_presence_b);

    assert_eq!(actual_table, expected);
}

#[test]
fn we_can_order_with_all_column_types() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        boolean("bool_col", [true, false, true]),
        uint8("uint8_col", [5_u8, 2_u8, 8_u8]),
        tinyint("tinyint_col", [3_i8, 1_i8, 4_i8]),
        smallint("smallint_col", [300_i16, 100_i16, 400_i16]),
        int("int_col", [30_000_i32, 10_000_i32, 40_000_i32]),
        bigint("bigint_col", [3_000_000_i64, 1_000_000_i64, 4_000_000_i64]),
        timestamptz(
            "timestamp_col",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [300_000_i64, 100_000_i64, 400_000_i64],
        ),
        int128("int128_col", [3_i128, 1_i128, 4_i128]),
        decimal75("decimal75", 10, 2, [300, 100, 400]),
        varchar("varchar_col", ["c", "a", "d"]),
        varbinary("varbinary_col", [&[3], &[1], &[4]]),
    ]);

    for col_idx in 0..11 {
        let postprocessing: [OwnedTablePostprocessing; 1] = [orders(&[col_idx], &[true])];
        let actual_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        assert_eq!(actual_table.num_rows(), 3);

        match col_idx {
            0 => assert_eq!(actual_table.column_by_index(0).unwrap().len(), 3),
            1 => assert_eq!(actual_table.column_by_index(1).unwrap().len(), 3),
            2 => assert_eq!(actual_table.column_by_index(2).unwrap().len(), 3),
            3 => assert_eq!(actual_table.column_by_index(3).unwrap().len(), 3),
            4 => assert_eq!(actual_table.column_by_index(4).unwrap().len(), 3),
            5 => assert_eq!(actual_table.column_by_index(5).unwrap().len(), 3),
            6 => assert_eq!(actual_table.column_by_index(6).unwrap().len(), 3),
            7 => assert_eq!(actual_table.column_by_index(7).unwrap().len(), 3),
            8 => assert_eq!(actual_table.column_by_index(8).unwrap().len(), 3),
            9 => assert_eq!(actual_table.column_by_index(9).unwrap().len(), 3),
            10 => assert_eq!(actual_table.column_by_index(10).unwrap().len(), 3),
            _ => panic!("Invalid column index"),
        }
    }
}
