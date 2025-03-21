use super::{PostprocessingStep, SelectPostprocessing};
use crate::df_util::*;
use core::ops::Add;
use proof_of_sql::{
    base::database::{owned_table_utility::*, OwnedTable},
    proof_primitive::dory::DoryScalar,
};

#[test]
fn we_can_filter_out_owned_table_columns() {
    let table: OwnedTable<DoryScalar> = owned_table([
        bigint("c", [-5_i64, 1, -56, 2]),
        varchar("a", ["d", "a", "f", "b"]),
    ]);
    let postprocessing =
        SelectPostprocessing::new(vec![df_column("schema.table_name", "a").alias("a")]);
    let expected_table = owned_table([varchar("a", ["d", "a", "f", "b"])]);
    let actual_table = postprocessing.apply(table).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_reorder_and_rename_owned_table_columns() {
    let table: OwnedTable<DoryScalar> = owned_table([
        int128("c", [-5_i128, 1, -56, 2]),
        varchar("a", ["d", "a", "f", "b"]),
    ]);

    // Build a single SelectPostprocessing, renaming columns "a" -> "b" and "c" -> "d",
    // in that order.
    let postprocessing = SelectPostprocessing::new(vec![
        df_column("schema.table_name", "a").alias("b"),
        df_column("schema.table_name", "c").alias("d"),
    ]);

    let expected_table = owned_table([
        varchar("b", ["d", "a", "f", "b"]),
        int128("d", [-5_i128, 1, -56, 2]),
    ]);

    let actual_table = postprocessing.apply(table).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_do_computation_on_owned_table_columns() {
    let table: OwnedTable<DoryScalar> = owned_table([bigint("c", [1, 2, 3, 4])]);

    // Create a single postprocessing step that produces
    // c + c as a new column named "res".
    let transformed_expr =
        df_column("schema.table_name", "c").add(df_column("schema.table_name", "c"));
    let postprocessing = SelectPostprocessing::new(vec![
        transformed_expr.clone(),
        transformed_expr.alias("res"),
    ]);

    let expected_table = owned_table([
        bigint("schema.table_name.c + schema.table_name.c", [2, 4, 6, 8]),
        bigint("res", [2, 4, 6, 8]),
    ]);

    let actual_table = postprocessing.apply(table).unwrap();
    assert_eq!(actual_table, expected_table);
}
