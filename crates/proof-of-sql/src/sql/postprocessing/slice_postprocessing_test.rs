use crate::{
    base::database::{owned_table_utility::*, OwnedTable},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
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

#[expect(clippy::cast_sign_loss)]
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

#[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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

#[expect(
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

#[expect(
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

#[test]
#[allow(clippy::too_many_lines)]
fn we_can_slice_a_table_with_null_values_following_sql_three_valued_logic() {
    use crate::{
        base::database::{
            owned_table_utility::{
                bigint_values, nullable_column_pair, owned_table_with_nulls, varchar_values,
            },
            OwnedColumn,
        },
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
        sql::postprocessing::test_utility::slice,
    };

    // Create a table with multiple columns and rows, with various NULL patterns
    // We'll create 10 rows with different NULL patterns across multiple columns

    // Column A: BigInt with some NULL values
    let a_values = bigint_values::<Curve25519Scalar>([10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    let a_presence = Some(vec![
        true, true, false, true, false, true, true, false, true, true,
    ]);

    // Column B: Int with some NULL values (different pattern)
    let b_values = OwnedColumn::Int(vec![5, 15, 25, 35, 45, 55, 65, 75, 85, 95]);
    let b_presence = Some(vec![
        true, false, true, false, true, true, false, true, true, false,
    ]);

    // Column C: VarChar with some NULL values
    let c_values =
        varchar_values::<Curve25519Scalar>(["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]);
    let c_presence = Some(vec![
        false, true, true, true, false, false, true, true, false, true,
    ]);

    // Get a reference to the integer values from b_values
    let OwnedColumn::Int(b_ints) = &b_values else {
        panic!("Expected Int column")
    };

    // Create the table with nullable columns
    let table = owned_table_with_nulls([
        nullable_column_pair("a", a_values.clone(), a_presence.clone()),
        nullable_column_pair("b", b_values.clone(), b_presence.clone()),
        nullable_column_pair("c", c_values.clone(), c_presence.clone()),
    ]);

    // Print initial table for debugging
    println!("Initial table: {table:?}");

    // Test 1: Apply LIMIT 5
    {
        let limit = 5_usize;
        let postprocessing = [slice(Some(limit as u64), None)];
        let result_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Print result table for debugging
        println!("Result table after LIMIT 5: {result_table:?}");

        // Create expected result - first 5 rows with NULL patterns preserved
        let expected_table = owned_table_with_nulls([
            nullable_column_pair(
                "a",
                bigint_values::<Curve25519Scalar>([10, 20, 30, 40, 50]),
                Some(a_presence.clone().unwrap()[0..5].to_vec()),
            ),
            nullable_column_pair(
                "b",
                OwnedColumn::Int(b_ints[0..5].to_vec()),
                Some(b_presence.clone().unwrap()[0..5].to_vec()),
            ),
            nullable_column_pair(
                "c",
                varchar_values::<Curve25519Scalar>(["A", "B", "C", "D", "E"]),
                Some(c_presence.clone().unwrap()[0..5].to_vec()),
            ),
        ]);

        assert_eq!(result_table, expected_table);
    }

    // Test 2: Apply OFFSET 3
    {
        let offset = 3;
        let postprocessing = [slice(None, Some(offset))];
        let result_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Print result table for debugging
        println!("Result table after OFFSET 3: {result_table:?}");

        // Create expected result - skipping first 3 rows but preserving NULL patterns
        let expected_table = owned_table_with_nulls([
            nullable_column_pair(
                "a",
                bigint_values::<Curve25519Scalar>([40, 50, 60, 70, 80, 90, 100]),
                Some(a_presence.clone().unwrap()[3..].to_vec()),
            ),
            nullable_column_pair(
                "b",
                OwnedColumn::Int(b_ints[3..].to_vec()),
                Some(b_presence.clone().unwrap()[3..].to_vec()),
            ),
            nullable_column_pair(
                "c",
                varchar_values::<Curve25519Scalar>(["D", "E", "F", "G", "H", "I", "J"]),
                Some(c_presence.clone().unwrap()[3..].to_vec()),
            ),
        ]);

        assert_eq!(result_table, expected_table);
    }

    // Test 3: Apply LIMIT 3 OFFSET 2
    {
        let limit = 3_usize;
        let offset = 2;
        let postprocessing = [slice(Some(limit as u64), Some(offset))];
        let result_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Print result table for debugging
        println!("Result table after LIMIT 3 OFFSET 2: {result_table:?}");

        // Create expected result - rows 2-4 with NULL patterns preserved
        let expected_table = owned_table_with_nulls([
            nullable_column_pair(
                "a",
                bigint_values::<Curve25519Scalar>([30, 40, 50]),
                Some(a_presence.clone().unwrap()[2..5].to_vec()),
            ),
            nullable_column_pair(
                "b",
                OwnedColumn::Int(b_ints[2..5].to_vec()),
                Some(b_presence.clone().unwrap()[2..5].to_vec()),
            ),
            nullable_column_pair(
                "c",
                varchar_values::<Curve25519Scalar>(["C", "D", "E"]),
                Some(c_presence.clone().unwrap()[2..5].to_vec()),
            ),
        ]);

        assert_eq!(result_table, expected_table);
    }

    // Test 4: Apply negative offset (from end)
    {
        let offset = -3;
        let postprocessing = [slice(None, Some(offset))];
        let result_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Print result table for debugging
        println!("Result table after OFFSET -3: {result_table:?}");

        // Create expected result - last 3 rows with NULL patterns preserved
        let expected_table = owned_table_with_nulls([
            nullable_column_pair(
                "a",
                bigint_values::<Curve25519Scalar>([80, 90, 100]),
                Some(a_presence.clone().unwrap()[7..].to_vec()),
            ),
            nullable_column_pair(
                "b",
                OwnedColumn::Int(b_ints[7..].to_vec()),
                Some(b_presence.clone().unwrap()[7..].to_vec()),
            ),
            nullable_column_pair(
                "c",
                varchar_values::<Curve25519Scalar>(["H", "I", "J"]),
                Some(c_presence.clone().unwrap()[7..].to_vec()),
            ),
        ]);

        assert_eq!(result_table, expected_table);
    }
}
