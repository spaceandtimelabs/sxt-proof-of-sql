use super::SelectPostprocessing;
use core::ops::Add;
use datafusion::logical_expr::Alias;
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
    let postprocessing = SelectPostprocessing::new(vec![Alias::new(df_column("schema.table_name", "a"), None, "a")]);
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
        Alias::new(df_column("schema.table_name", "a"), None, "b"),
        Alias::new(df_column("schema.table_name", "c"), None, "d"),
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
    let table: OwnedTable<DoryScalar> = owned_table([
        bigint("c", [1, 2, 3, 4])
    ]);

    // Create a single postprocessing step that produces
    // (c + c + 1) as a new column named "res".
    let transformed_expr = df_column("schema.table_name", "c").add(df_column("schema.table_name", "c")).add(1);
    let postprocessing = SelectPostprocessing::new(vec![
        Alias::new(transformed_expr, None, "res"),
    ]);

    let expected_table = owned_table([
        bigint("res", [3, 5, 7, 9])
    ]);

    let actual_table = postprocessing.apply(table).unwrap();
    assert_eq!(actual_table, expected_table);
}
