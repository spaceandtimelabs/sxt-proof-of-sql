use crate::{
    base::{
        database::{OwnedTable, owned_table_utility::*},
        scalar::Curve25519Scalar,
    },
    sql::postprocessing::{OwnedTablePostprocessing, apply_postprocessing_steps, test_utility::*},
};
use rand::{Rng, seq::SliceRandom};

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
