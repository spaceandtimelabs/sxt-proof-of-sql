use crate::base::{
    database::{
        owned_column::OwnedNullableColumn, owned_table_utility::*, ColumnOperationError,
        ExpressionEvaluationError, OwnedColumn, OwnedTable,
    },
    math::decimal::Precision,
    scalar::test_scalar::TestScalar,
};
use alloc::vec;
use bigdecimal::BigDecimal;
use proof_of_sql_parser::{
    intermediate_ast::Literal,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestamp},
    utility::*,
};

#[test]
fn we_can_evaluate_a_simple_literal() {
    let table: OwnedTable<TestScalar> =
        owned_table([varchar("languages", ["en", "es", "pt", "fr", "ht"])]);

    // "Space and Time" in Hebrew
    let expr = lit("מרחב וזמן".to_string());
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::VarChar(vec!["מרחב וזמן".to_string(); 5]);
    assert_eq!(actual_column, expected_column);

    // Is Proof of SQL in production?
    let expr = lit(true);
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![true; 5]);
    assert_eq!(actual_column, expected_column);

    // When was Space and Time founded?
    let timestamp = "2022-03-01T00:00:00Z";
    let expr = lit(Literal::Timestamp(
        PoSQLTimestamp::try_from(timestamp).unwrap(),
    ));
    let actual_column = table.evaluate(&expr).unwrap();
    // UNIX timestamp for 2022-03-01T00:00:00Z
    let actual_timestamp = 1_646_092_800;
    let expected_column = OwnedColumn::TimestampTZ(
        PoSQLTimeUnit::Second.into(),
        PoSQLTimeZone::utc().into(),
        vec![actual_timestamp; 5],
    );
    assert_eq!(actual_column, expected_column);

    // A group of people has about 0.67 cats per person
    let expr = lit("0.67".parse::<BigDecimal>().unwrap());
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::Decimal75(Precision::new(2).unwrap(), 2, vec![67.into(); 5]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_a_simple_column() {
    let table: OwnedTable<TestScalar> = owned_table([
        bigint("bigints", [i64::MIN, -1, 0, 1, i64::MAX]),
        varchar("language", ["en", "es", "pt", "fr", "ht"]),
        varchar("john", ["John", "Juan", "João", "Jean", "Jean"]),
    ]);
    let expr = col("bigints");
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::BigInt(vec![i64::MIN, -1, 0, 1, i64::MAX]);
    assert_eq!(actual_column, expected_column);

    let expr = col("john");
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::VarChar(
        ["John", "Juan", "João", "Jean", "Jean"]
            .iter()
            .map(ToString::to_string)
            .collect(),
    );
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_cannot_evaluate_a_nonexisting_column() {
    let table: OwnedTable<TestScalar> =
        owned_table([varchar("cats", ["Chloe", "Margaret", "Prudence", "Lucy"])]);
    // "not_a_column" is not a column in the table
    let expr = col("not_a_column");
    assert!(matches!(
        table.evaluate(&expr),
        Err(ExpressionEvaluationError::ColumnNotFound { .. })
    ));
}

#[test]
fn we_can_evaluate_a_logical_expression() {
    let table: OwnedTable<TestScalar> = owned_table([
        varchar("en", ["Elizabeth", "John", "cat", "dog", "Munich"]),
        varchar("pl", ["Elżbieta", "Jan", "kot", "pies", "Monachium"]),
        varchar("cz", ["Alžběta", "Jan", "kočka", "pes", "Mnichov"]),
        varchar("sk", ["Alžbeta", "Ján", "mačka", "pes", "Mníchov"]),
        varchar("hr", ["Elizabeta", "Ivan", "mačka", "pas", "München"]),
        varchar("sl", ["Elizabeta", "Janez", "mačka", "pes", "München"]),
        boolean("is_proper_noun", [true, true, false, false, true]),
    ]);

    // Find words that are not proper nouns
    let expr = not(col("is_proper_noun"));
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![false, false, true, true, false]);
    assert_eq!(actual_column, expected_column);

    // Which Czech and Slovak words agree?
    let expr = equal(col("cz"), col("sk"));
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column: OwnedColumn<TestScalar> =
        OwnedColumn::Boolean(vec![false, false, false, true, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared among Slovak, Croatian and Slovenian
    let expr = and(equal(col("sk"), col("hr")), equal(col("hr"), col("sl")));
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column: OwnedColumn<TestScalar> =
        OwnedColumn::Boolean(vec![false, false, true, false, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared between Polish and Czech but not Slovenian
    let expr = and(
        equal(col("pl"), col("cz")),
        not(equal(col("pl"), col("sl"))),
    );
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column: OwnedColumn<TestScalar> =
        OwnedColumn::Boolean(vec![false, true, false, false, false]);
    assert_eq!(actual_column, expected_column);

    // Proper nouns as well as words shared between Croatian and Slovenian
    let expr = or(
        col("is_proper_noun"),
        and(equal(col("hr"), col("sl")), equal(col("hr"), col("sk"))),
    );
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column: OwnedColumn<TestScalar> =
        OwnedColumn::Boolean(vec![true, true, true, false, true]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_an_arithmetic_expression() {
    let table: OwnedTable<TestScalar> = owned_table([
        smallint("smallints", [-2_i16, -1, 0, 1, 2]),
        int("ints", [-4_i32, -2, 0, 2, 4]),
        bigint("bigints", [-8_i64, -4, 0, 4, 8]),
        int128("int128s", [-16_i128, -8, 0, 8, 16]),
        decimal75("decimals", 2, 1, [0, 1, 2, 3, 4]),
    ]);

    // Subtract 1 from the bigints
    let expr = sub(col("bigints"), lit(1));
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::BigInt(vec![-9, -5, -1, 3, 7]);
    assert_eq!(actual_column, expected_column);

    // Add bigints to the smallints and multiply the sum by the ints
    let expr = mul(add(col("bigints"), col("smallints")), col("ints"));
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::BigInt(vec![40, 10, 0, 10, 40]);
    assert_eq!(actual_column, expected_column);

    // Multiply decimals with 0.75 and add smallints to the product
    let expr = add(
        col("smallints"),
        mul(col("decimals"), lit("0.75".parse::<BigDecimal>().unwrap())),
    );
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_scalars = [-2000, -925, 150, 1225, 2300]
        .iter()
        .map(|&x| x.into())
        .collect();
    let expected_column = OwnedColumn::Decimal75(Precision::new(9).unwrap(), 3, expected_scalars);
    assert_eq!(actual_column, expected_column);

    // Decimals over 2.5 plus int128s
    let expr = add(
        div(col("decimals"), lit("2.5".parse::<BigDecimal>().unwrap())),
        col("int128s"),
    );
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_scalars = [-16_000_000, -7_960_000, 80000, 8_120_000, 16_160_000]
        .iter()
        .map(|&x| x.into())
        .collect();
    let expected_column = OwnedColumn::Decimal75(Precision::new(46).unwrap(), 6, expected_scalars);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_cannot_evaluate_expressions_if_column_operation_errors_out() {
    let table: OwnedTable<TestScalar> = owned_table([
        bigint("bigints", [i64::MIN, -1, 0, 1, i64::MAX]),
        varchar("language", ["en", "es", "pt", "fr", "ht"]),
        varchar("sarah", ["Sarah", "Sara", "Sara", "Sarah", "Sarah"]),
    ]);

    // NOT doesn't work on varchar
    let expr = not(col("language"));
    assert!(matches!(
        table.evaluate(&expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::UnaryOperationInvalidColumnType { .. }
        })
    ));

    // NOT doesn't work on bigint
    let expr = not(col("bigints"));
    assert!(matches!(
        table.evaluate(&expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::UnaryOperationInvalidColumnType { .. }
        })
    ));

    // + doesn't work on varchar
    let expr = add(col("sarah"), col("bigints"));
    assert!(matches!(
        table.evaluate(&expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::BinaryOperationInvalidColumnType { .. }
        })
    ));

    // i64::MIN - 1 overflows
    let expr = sub(col("bigints"), lit(1));
    assert!(matches!(
        table.evaluate(&expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::IntegerOverflow { .. }
        })
    ));

    // We can't divide by zero
    let expr = div(col("bigints"), lit(0));
    assert!(matches!(
        table.evaluate(&expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::DivisionByZero
        })
    ));
}

#[test]
fn we_can_evaluate_nullable_columns() {
    let table: OwnedTable<TestScalar> = owned_table([
        bigint("ids", [1, 2, 3, 4, 5]),
        varchar("names", ["Alice", "Bob", "Charlie", "Dave", "Eve"]),
        boolean("active", [true, false, true, true, false]),
        int("scores", [10, 20, 30, 40, 50]),
    ]);

    let expr = col("scores");
    let result = table.evaluate_nullable(&expr).unwrap();

    assert_eq!(
        result.values,
        OwnedColumn::<TestScalar>::Int(vec![10, 20, 30, 40, 50])
    );
    assert_eq!(result.presence, None);

    let expr = col("names");
    let result = table.evaluate_nullable(&expr).unwrap();

    assert_eq!(
        result.values,
        OwnedColumn::<TestScalar>::VarChar(vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
            "Dave".to_string(),
            "Eve".to_string()
        ])
    );
    assert_eq!(result.presence, None);
}

#[test]
fn we_can_evaluate_nullable_expressions() {
    let table: OwnedTable<TestScalar> = owned_table([
        bigint("ids", [1, 2, 3, 4, 5]),
        int("values", [10, 20, 30, 40, 50]),
        boolean("flags", [true, false, true, false, true]),
    ]);

    let expr = add(col("values"), col("values"));
    let result = table.evaluate_nullable(&expr).unwrap();

    assert_eq!(
        result.values,
        OwnedColumn::<TestScalar>::Int(vec![20, 40, 60, 80, 100])
    );
    assert_eq!(result.presence, None);

    let expr = and(col("flags"), col("flags"));
    let result = table.evaluate_nullable(&expr).unwrap();

    assert_eq!(
        result.values,
        OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false, true])
    );
    assert_eq!(result.presence, None);
}

#[test]
fn we_can_handle_null_propagation_in_expressions() {
    let a =
        OwnedNullableColumn::<TestScalar>::new(OwnedColumn::<TestScalar>::Int(vec![1, 2, 3, 4, 5]));
    let b = OwnedNullableColumn::<TestScalar>::new(OwnedColumn::<TestScalar>::Int(vec![
        10, 20, 30, 40, 50,
    ]));
    let values_c = OwnedColumn::<TestScalar>::Int(vec![100, 200, 300, 400, 500]);
    let presence_c = Some(vec![true, false, true, false, true]);
    let c = OwnedNullableColumn::<TestScalar>::with_presence(values_c, presence_c.clone()).unwrap();
    let values_d = OwnedColumn::<TestScalar>::Int(vec![1000, 2000, 3000, 4000, 5000]);
    let presence_d = Some(vec![false, true, false, true, false]);
    let d = OwnedNullableColumn::<TestScalar>::with_presence(values_d, presence_d.clone()).unwrap();
    let a_plus_b = a.element_wise_add(&b).unwrap();
    let c_minus_d = c.element_wise_sub(&d).unwrap();
    let result = a_plus_b.element_wise_mul(&c_minus_d).unwrap();
    let expected_values = vec![
        (1 + 10) * (100 - 1000), // -9900
        (2 + 20) * (200 - 2000), // -39600
        (3 + 30) * (300 - 3000), // -89100
        (4 + 40) * (400 - 4000), // -158400
        (5 + 50) * (500 - 5000), // -247500
    ];

    let expected_presence = Some(vec![
        false, // false (d is NULL)
        false, // false (c is NULL)
        false, // false (d is NULL)
        false, // false (c is NULL)
        false, // false (d is NULL)
    ]);

    assert_eq!(
        result.values,
        OwnedColumn::<TestScalar>::Int(expected_values)
    );
    assert_eq!(result.presence, expected_presence);

    let bool_values_c = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false, true]);
    let bool_presence_c = Some(vec![true, false, true, false, true]);
    let bool_c =
        OwnedNullableColumn::<TestScalar>::with_presence(bool_values_c, bool_presence_c).unwrap();
    let all_false =
        OwnedNullableColumn::<TestScalar>::new(OwnedColumn::<TestScalar>::Boolean(vec![
            false, false, false, false, false,
        ]));
    let result = bool_c.element_wise_and(&all_false).unwrap();
    let expected_values =
        OwnedColumn::<TestScalar>::Boolean(vec![false, false, false, false, false]);
    let expected_presence = Some(vec![
        true, // true AND false = false (not NULL)
        true, // NULL AND false = false (not NULL)
        true, // false AND false = false (not NULL)
        true, // NULL AND false = false (not NULL)
        true, // true AND false = false (not NULL)
    ]);

    assert_eq!(result.values, expected_values);
    assert_eq!(result.presence, expected_presence);

    // NULL OR true = true
    let all_true =
        OwnedNullableColumn::<TestScalar>::new(OwnedColumn::<TestScalar>::Boolean(vec![
            true, true, true, true, true,
        ]));
    let result = bool_c.element_wise_or(&all_true).unwrap();
    let expected_values = OwnedColumn::<TestScalar>::Boolean(vec![true, true, true, true, true]);
    let expected_presence = Some(vec![
        true, // true OR true = true (not NULL)
        true, // NULL OR true = true (not NULL)
        true, // false OR true = true (not NULL)
        true, // NULL OR true = true (not NULL)
        true, // true OR true = true (not NULL)
    ]);

    assert_eq!(result.values, expected_values);
    assert_eq!(result.presence, expected_presence);
}

#[test]
fn we_can_convert_nullable_to_non_nullable() {
    let table: OwnedTable<TestScalar> = owned_table([int("a", [1, 2, 3, 4, 5])]);

    let expr = add(col("a"), col("a"));
    let result = table.evaluate(&expr).unwrap();

    assert_eq!(result, OwnedColumn::<TestScalar>::Int(vec![2, 4, 6, 8, 10]));

    let truly_non_null =
        OwnedNullableColumn::<TestScalar>::new(OwnedColumn::<TestScalar>::Int(vec![
            10, 20, 30, 40, 50,
        ]));

    let all_present = OwnedNullableColumn::<TestScalar>::with_presence(
        OwnedColumn::<TestScalar>::Int(vec![10, 20, 30, 40, 50]),
        Some(vec![true, true, true, true, true]),
    )
    .unwrap();

    let with_nulls = OwnedNullableColumn::<TestScalar>::with_presence(
        OwnedColumn::<TestScalar>::Int(vec![100, 200, 300, 400, 500]),
        Some(vec![true, false, true, false, true]),
    )
    .unwrap();

    assert!(!truly_non_null.is_nullable());
    assert!(all_present.is_nullable());
    assert!(with_nulls.is_nullable());
}

#[test]
fn we_can_simulate_sql_where_clause_with_nulls() {
    // Create columns for our test table
    // Column A with some NULL values
    let a_values = OwnedColumn::<TestScalar>::BigInt(vec![1, 1, 0, 0, 2, 2, 0]);
    let a_presence = Some(vec![true, true, false, false, true, true, false]);

    // Column B with some NULL values
    let b_values = OwnedColumn::<TestScalar>::BigInt(vec![1, 0, 1, 0, 2, 0, 2]);
    let b_presence = Some(vec![true, false, true, false, true, false, true]);

    // Column C with no NULL values
    let c_values = OwnedColumn::<TestScalar>::BigInt(vec![101, 102, 103, 104, 105, 106, 107]);

    // Create the table using the new owned_table_with_nulls utility function
    let table = owned_table_with_nulls([
        nullable_column_pair("a", a_values, a_presence.clone()),
        nullable_column_pair("b", b_values, b_presence.clone()),
        nullable_column_pair("c", c_values, None),
    ]);

    // First, evaluate the arithmetic expression: A + B
    let add_expr = add(col("a"), col("b"));
    let sum_result = table.evaluate_nullable(&add_expr).unwrap();

    // The sum result should have NULLs where either A or B is NULL
    assert_eq!(
        sum_result.values,
        OwnedColumn::<TestScalar>::BigInt(vec![2, 1, 1, 0, 4, 2, 2])
    );

    // The presence should be NULL (false) where either A or B is NULL
    let expected_sum_presence = Some(vec![
        true,  // A=1, B=1 -> 1+1=2 (not NULL)
        false, // A=1, B=NULL -> NULL
        false, // A=NULL, B=1 -> NULL
        false, // A=NULL, B=NULL -> NULL
        true,  // A=2, B=2 -> 2+2=4 (not NULL)
        false, // A=2, B=NULL -> NULL
        false, // A=NULL, B=2 -> NULL
    ]);
    assert_eq!(sum_result.presence, expected_sum_presence);

    // Now we evaluate the comparison: (A + B) = 2
    let eq_expr = equal(add(col("a"), col("b")), lit(2));
    let eq_result = table.evaluate_nullable(&eq_expr).unwrap();

    // First verify the presence - this is the critical part that defines NULL behavior
    let expected_eq_presence = Some(vec![
        true,  // A=1, B=1 -> 1+1=2 -> true (not NULL)
        false, // A=1, B=NULL -> NULL
        false, // A=NULL, B=1 -> NULL
        false, // A=NULL, B=NULL -> NULL
        true,  // A=2, B=2 -> 2+2=4 -> false (not NULL)
        false, // A=2, B=NULL -> NULL
        false, // A=NULL, B=2 -> NULL
    ]);
    assert_eq!(eq_result.presence, expected_eq_presence);

    // Then verify only the non-NULL values (where presence is true)
    // For NULL values (presence=false), the actual value is implementation-defined
    match &eq_result.values {
        OwnedColumn::Boolean(values) => {
            let presence = eq_result.presence.as_ref().unwrap();
            for i in 0..values.len() {
                if presence[i] {
                    // Only verify values where presence is true
                    match i {
                        0 => assert!(values[i]),  // 1+1=2 -> true
                        4 => assert!(!values[i]), // 2+2=4 -> false
                        _ => panic!("Unexpected non-NULL value at index {i}"),
                    }
                }
            }
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn we_can_evaluate_null_literal() {
    let table: OwnedTable<TestScalar> = owned_table([
        int("a", [1, 2, 3, 4, 5]),
        varchar("b", ["x", "y", "z", "w", "v"]),
    ]);

    let expr = lit(Literal::Null);
    let result = table.evaluate(&expr).unwrap();

    assert_eq!(result, OwnedColumn::<TestScalar>::Boolean(vec![false; 5]));
}

#[test]
fn we_can_evaluate_nullable_null_literal() {
    let table: OwnedTable<TestScalar> = owned_table([
        int("a", [1, 2, 3, 4, 5]),
        varchar("b", ["x", "y", "z", "w", "v"]),
    ]);

    let expr = lit(Literal::Null);
    let result = table.evaluate_nullable(&expr).unwrap();

    assert_eq!(
        result.values,
        OwnedColumn::<TestScalar>::Boolean(vec![false; 5])
    );
    assert_eq!(result.presence, Some(vec![false; 5]));
}

#[test]
#[should_panic(expected = "Unexpected non-NULL value at index 1")]
fn test_unexpected_non_null_value_index() {
    let a_values = OwnedColumn::<TestScalar>::BigInt(vec![1, 1, 0, 0, 2]);
    let a_presence = Some(vec![true, true, false, false, true]);

    let b_values = OwnedColumn::<TestScalar>::BigInt(vec![1, 1, 1, 0, 2]);
    let b_presence = Some(vec![true, true, true, false, true]);

    let _table = owned_table_with_nulls([
        nullable_column_pair("a", a_values, a_presence),
        nullable_column_pair("b", b_values, b_presence),
    ]);

    let result_values = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false, false]);
    let result_presence = Some(vec![true, true, false, false, true]);

    let eq_result =
        OwnedNullableColumn::<TestScalar>::with_presence(result_values, result_presence).unwrap();

    match &eq_result.values {
        OwnedColumn::Boolean(values) => {
            let presence = eq_result.presence.as_ref().unwrap();
            for i in 0..values.len() {
                if presence[i] {
                    match i {
                        0 => assert!(values[i]),
                        4 => assert!(!values[i]),
                        _ => panic!("Unexpected non-NULL value at index {i}"),
                    }
                }
            }
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
#[should_panic(expected = "Expected boolean column")]
fn test_non_boolean_column_result() {
    let _table: OwnedTable<TestScalar> = owned_table([int("a", [1, 2, 3, 4, 5])]);

    let result_values = OwnedColumn::<TestScalar>::Int(vec![10, 20, 30, 40, 50]);
    let result_presence = Some(vec![true, true, false, false, true]);

    let eq_result =
        OwnedNullableColumn::<TestScalar>::with_presence(result_values, result_presence).unwrap();

    match &eq_result.values {
        OwnedColumn::Boolean(values) => {
            let presence = eq_result.presence.as_ref().unwrap();
            for i in 0..values.len() {
                if presence[i] {
                    match i {
                        0 => assert!(values[i]),
                        4 => assert!(!values[i]),
                        _ => panic!("Unexpected non-NULL value at index {i}"),
                    }
                }
            }
        }
        _ => panic!("Expected boolean column"),
    }
}
