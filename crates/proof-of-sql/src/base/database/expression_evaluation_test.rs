use crate::base::{
    database::{
        owned_table_utility::*, ColumnOperationError, ExpressionEvaluationError, OwnedColumn,
        OwnedTable,
    },
    math::decimal::Precision,
    scalar::Curve25519Scalar,
};
use proof_of_sql_parser::{
    intermediate_ast::Literal,
    intermediate_decimal::IntermediateDecimal,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestamp},
    utility::*,
};

#[test]
fn we_can_evaluate_a_simple_literal() {
    let table: OwnedTable<Curve25519Scalar> =
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
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::Utc,
        vec![actual_timestamp; 5],
    );
    assert_eq!(actual_column, expected_column);

    // A group of people has about 0.67 cats per person
    let expr = lit("0.67".parse::<IntermediateDecimal>().unwrap());
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column = OwnedColumn::Decimal75(Precision::new(2).unwrap(), 2, vec![67.into(); 5]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_a_simple_column() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
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
fn we_can_not_evaluate_a_nonexisting_column() {
    let table: OwnedTable<Curve25519Scalar> =
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
    let table: OwnedTable<Curve25519Scalar> = owned_table([
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
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![false, false, false, true, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared among Slovak, Croatian and Slovenian
    let expr = and(equal(col("sk"), col("hr")), equal(col("hr"), col("sl")));
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![false, false, true, false, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared between Polish and Czech but not Slovenian
    let expr = and(
        equal(col("pl"), col("cz")),
        not(equal(col("pl"), col("sl"))),
    );
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![false, true, false, false, false]);
    assert_eq!(actual_column, expected_column);

    // Proper nouns as well as words shared between Croatian and Slovenian
    let expr = or(
        col("is_proper_noun"),
        and(equal(col("hr"), col("sl")), equal(col("hr"), col("sk"))),
    );
    let actual_column = table.evaluate(&expr).unwrap();
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![true, true, true, false, true]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_an_arithmetic_expression() {
    let table: OwnedTable<Curve25519Scalar> = owned_table([
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
        mul(
            col("decimals"),
            lit("0.75".parse::<IntermediateDecimal>().unwrap()),
        ),
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
        div(
            col("decimals"),
            lit("2.5".parse::<IntermediateDecimal>().unwrap()),
        ),
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
    let table: OwnedTable<Curve25519Scalar> = owned_table([
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
