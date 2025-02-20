use crate::{
    base::database::{owned_table_utility::*, OwnedTable},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::postprocessing::{apply_postprocessing_steps, test_utility::*, OwnedTablePostprocessing},
};
use proof_of_sql_parser::utility::*;

#[test]
fn we_can_filter_out_owned_table_columns() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("c", [-5_i64, 1, -56, 2]),
        varchar("a", ["d", "a", "f", "b"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] =
        [select_expr(&[aliased_expr(col("a"), "a")])];
    let expected_table = owned_table([varchar("a", ["d", "a", "f", "b"])]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_reorder_and_rename_owned_table_columns() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("c", [-5_i128, 1, -56, 2]),
        varchar("a", ["d", "a", "f", "b"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [select_expr(&[
        aliased_expr(col("a"), "b"),
        aliased_expr(col("c"), "d"),
    ])];
    let expected_table = owned_table([
        varchar("b", ["d", "a", "f", "b"]),
        int128("d", [-5_i128, 1, -56, 2]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_do_computation_on_owned_table_columns() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([bigint("c", [1, 2, 3, 4])]);
    let res_col = add(add(col("c"), col("c")), lit(1));
    let postprocessing: [OwnedTablePostprocessing; 1] =
        [select_expr(&[aliased_expr(res_col, "res")])];
    let expected_table = owned_table([bigint("res", [3, 5, 7, 9])]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}
