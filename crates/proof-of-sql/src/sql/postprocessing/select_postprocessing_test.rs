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

#[test]
fn we_can_select_with_null_values_following_sql_three_valued_logic() {
    use crate::base::database::{
        owned_table_utility::{
            bigint_values, nullable_column_pair, owned_table_with_nulls, varchar_values,
        },
        OwnedColumn,
    };
    use proof_of_sql_parser::utility::{add, col, lit, mul};

    // Create a table with multiple columns and rows, with various NULL patterns
    // We'll create 5 rows with different NULL patterns across multiple columns

    // Column A: BigInt with some NULL values
    let a_values = bigint_values::<Curve25519Scalar>([10, 20, 30, 40, 50]);
    let a_presence = Some(vec![true, true, false, true, false]);

    // Column B: Int with some NULL values (different pattern)
    let b_values = OwnedColumn::Int(vec![5, 15, 25, 35, 45]);
    let b_presence = Some(vec![true, false, true, false, true]);

    // Column C: VarChar with some NULL values
    let c_values = varchar_values::<Curve25519Scalar>(["A", "B", "C", "D", "E"]);
    let c_presence = Some(vec![false, true, true, false, true]);

    // Create the table with nullable columns
    let table = owned_table_with_nulls([
        nullable_column_pair("a", a_values.clone(), a_presence.clone()),
        nullable_column_pair("b", b_values.clone(), b_presence.clone()),
        nullable_column_pair("c", c_values.clone(), c_presence.clone()),
    ]);

    // Test 1: Select specific columns with NULL values
    {
        let postprocessing: [OwnedTablePostprocessing; 1] = [select_expr(&[
            aliased_expr(col("a"), "a"),
            aliased_expr(col("c"), "c"),
        ])];

        let result_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Create expected result - only columns a and c with NULL patterns preserved
        let expected_table = owned_table_with_nulls([
            nullable_column_pair("a", a_values.clone(), a_presence.clone()),
            nullable_column_pair("c", c_values.clone(), c_presence.clone()),
        ]);

        assert_eq!(result_table, expected_table);
    }

    // Test 2: Rename columns with NULL values
    {
        let postprocessing: [OwnedTablePostprocessing; 1] = [select_expr(&[
            aliased_expr(col("a"), "a_renamed"),
            aliased_expr(col("b"), "b_renamed"),
            aliased_expr(col("c"), "c_renamed"),
        ])];

        let result_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Create expected result - all columns renamed but NULL patterns preserved
        let expected_table = owned_table_with_nulls([
            nullable_column_pair("a_renamed", a_values.clone(), a_presence.clone()),
            nullable_column_pair("b_renamed", b_values.clone(), b_presence.clone()),
            nullable_column_pair("c_renamed", c_values.clone(), c_presence.clone()),
        ]);

        assert_eq!(result_table, expected_table);
    }

    // Test 3: Compute expressions on columns with NULL values
    {
        // Create expressions that will result in NULL when input is NULL
        let a_plus_b = add(col("a"), col("b"));
        let a_times_2 = mul(col("a"), lit(2));

        let postprocessing: [OwnedTablePostprocessing; 1] = [select_expr(&[
            aliased_expr(a_plus_b, "a_plus_b"),
            aliased_expr(a_times_2, "a_times_2"),
            aliased_expr(col("c"), "c"),
        ])];

        let result_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Expected values for a_plus_b: NULL propagation in SQL expressions
        // If either operand is NULL, the result is NULL
        // Row 1: 10 + 5 = 15
        // Row 2: 20 + NULL = NULL
        // Row 3: NULL + 25 = NULL
        // Row 4: 40 + NULL = NULL
        // Row 5: NULL + 45 = NULL
        let a_plus_b_values = bigint_values::<Curve25519Scalar>([15, 0, 0, 0, 0]);
        let a_plus_b_presence = Some(vec![true, false, false, false, false]);

        // Expected values for a_times_2
        // Row 1: 10 * 2 = 20
        // Row 2: 20 * 2 = 40
        // Row 3: NULL * 2 = NULL
        // Row 4: 40 * 2 = 80
        // Row 5: NULL * 2 = NULL
        let a_times_2_values = bigint_values::<Curve25519Scalar>([20, 40, 0, 80, 0]);
        let a_times_2_presence = Some(vec![true, true, false, true, false]);

        // Create expected result with computed expressions
        let expected_table = owned_table_with_nulls([
            nullable_column_pair("a_plus_b", a_plus_b_values, a_plus_b_presence),
            nullable_column_pair("a_times_2", a_times_2_values, a_times_2_presence),
            nullable_column_pair("c", c_values.clone(), c_presence.clone()),
        ]);

        assert_eq!(result_table, expected_table);
    }
}
