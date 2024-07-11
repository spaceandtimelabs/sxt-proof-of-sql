use crate::{
    base::{database::owned_table_utility::*, scalar::Curve25519Scalar},
    sql::postprocessing::{apply_postprocessing_steps, test_utility::*, OwnedTablePostprocessing},
};
use proof_of_sql_parser::test_utility::*;
use rand::{seq::SliceRandom, Rng};

#[test]
fn we_can_filter_out_owned_table_columns() {
    let data = owned_table([bigint("c", [-5_i64, 1, -56, 2]), varchar("a", ["d", "a", "f", "b"])]);
    let plans = cols_expr_plan(&["a"]);
    let postprocessing: [OwnedTablePostprocessing<Curve25519Scalar>; 1] = [select(&[aliased_col_expr_plan("a")])];
    let expected_table = owned_table([
        varchar("a", ["d", "a", "f", "b"]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_filter_out_owned_table_columns_with_i128_data() {
    let data = owned_table("c" => [-5_i128, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let plans = aliased_cols_expr_plan(&["a"]);
    let postprocessing: [OwnedTablePostprocessing<Curve25519Scalar>; 2] = [select(&[aliased_col_expr_plans("a")])];
    let data = result_expr.transform_results(data).unwrap();
    let expected_data = owned_table!("a2" => ["d", "a", "f", "b"]);
    assert_eq!(data, expected_data);
}

#[test]
#[should_panic]
fn result_expr_panics_with_batches_containing_duplicate_columns() {
    let data = owned_table!("a" => [-5_i64, 1, -56, 2], "a" => [-5_i64, 1, -56, 2]);
    let result_expr = ResultExpr::new(select(&[col("a").alias("a2"), col("a").alias("a3")]));
    result_expr.transform_results(data).unwrap();
}

#[test]
fn we_can_reorder_the_owned_table_columns_without_changing_their_names() {
    let data = owned_table!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[col("a").alias("a"), col("c").alias("c")]));
    let data = result_expr.transform_results(data).unwrap();
    let expected_data = owned_table!("a" => ["d", "a", "f", "b"], "c" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_owned_table_columns_to_different_names() {
    let data = owned_table!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[
        col("a").alias("b_test"),
        col("c").alias("col_c_test"),
    ]));
    let data = result_expr.transform_results(data).unwrap();
    let expected_data =
        owned_table!("b_test" => ["d", "a", "f", "b"], "col_c_test" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}

#[test]
fn we_can_remap_the_owned_table_columns_to_new_columns() {
    let data = owned_table!("c" => [-5_i64, 1, -56, 2], "a" => ["d", "a", "f", "b"]);
    let result_expr = ResultExpr::new(select(&[
        col("c").alias("abc"),
        col("a").alias("b_test"),
        col("a").alias("d2"),
        col("c").alias("c"),
    ]));
    let data = result_expr.transform_results(data).unwrap();
    let expected_data = owned_table!("abc" => [-5_i64, 1, -56, 2], "b_test" => ["d", "a", "f", "b"], "d2" => ["d", "a", "f", "b"], "c" => [-5_i64, 1, -56, 2]);
    assert_eq!(data, expected_data);
}
