use crate::{
    base::{
        database::{owned_table_utility::*, OwnedTable},
        scalar::Curve25519Scalar,
    },
    sql::postprocessing::{apply_postprocessing_steps, test_utility::*},
};

#[test]
fn we_can_slice_an_owned_table_using_only_a_positive_limit_value() {
    let limit = 3_usize;
    let data_a = [123_i64, 342, -234, 777, 123, 34];
    let data_d = ["alfa", "beta", "abc", "f", "kl", "f"];
    let table: OwnedTable<Curve25519Scalar> =
        owned_table([bigint("a", data_a.to_vec()), varchar("d", data_d.to_vec())]);
    let expected_table = owned_table([
        bigint("a", data_a[0..limit].to_vec()),
        varchar("d", data_d[0..limit].to_vec()),
    ]);
    let postprocessing = [slice(Some(limit as u64), None)];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#[test]
fn we_can_slice_an_owned_table_using_only_a_zero_limit_value() {
    let limit = 0;
    let data_a = [123_i64, 342, -234, 777, 123, 34];
    let data_d = ["alfa", "beta", "abc", "f", "kl", "f"];
    let table: OwnedTable<Curve25519Scalar> =
        owned_table([bigint("a", data_a.to_vec()), varchar("d", data_d.to_vec())]);
    let expected_table = owned_table([
        bigint("a", Vec::<i64>::new()),
        varchar("d", Vec::<String>::new()),
    ]);
    let postprocessing = [slice(Some(limit as u64), None)];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#[test]
fn we_can_slice_an_owned_table_using_only_a_positive_offset_value() {
    let offset = 3;
    let data_a = [123_i64, 342, -234, 777, 123, 34];
    let data_d = ["alfa", "beta", "abc", "f", "kl", "f"];
    let table: OwnedTable<Curve25519Scalar> =
        owned_table([bigint("a", data_a.to_vec()), varchar("d", data_d.to_vec())]);
    let expected_table = owned_table([
        bigint("a", data_a[(offset as usize)..].to_vec()),
        varchar("d", data_d[(offset as usize)..].to_vec()),
    ]);
    let postprocessing = [slice(None, Some(offset))];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
#[test]
fn we_can_slice_an_owned_table_using_only_a_negative_offset_value() {
    let offset = -2;
    let data_a = [123_i64, 342, -234, 777, 123, 34];
    let data_d = ["alfa", "beta", "abc", "f", "kl", "f"];
    let table: OwnedTable<Curve25519Scalar> =
        owned_table([bigint("a", data_a.to_vec()), varchar("d", data_d.to_vec())]);
    let expected_table = owned_table([
        bigint(
            "a",
            data_a[(data_a.len() as i64 + offset) as usize..].to_vec(),
        ),
        varchar(
            "d",
            data_d[(data_a.len() as i64 + offset) as usize..].to_vec(),
        ),
    ]);
    let postprocessing = [slice(None, Some(offset))];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
#[test]
fn we_can_slice_an_owned_table_using_both_limit_and_offset_values() {
    let offset = -2;
    let limit = 1_usize;
    let data_a = [123_i64, 342, -234, 777, 123, 34];
    let data_d = ["alfa", "beta", "abc", "f", "kl", "f"];
    let table: OwnedTable<Curve25519Scalar> =
        owned_table([bigint("a", data_a.to_vec()), varchar("d", data_d.to_vec())]);
    let beg_expected_index = (data_a.len() as i64 + offset) as usize;
    let expected_table = owned_table([
        bigint(
            "a",
            data_a[beg_expected_index..(beg_expected_index + limit)].to_vec(),
        ),
        varchar(
            "d",
            data_d[beg_expected_index..(beg_expected_index + limit)].to_vec(),
        ),
    ]);
    let postprocessing = [slice(Some(limit as u64), Some(offset))];
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}
