use super::{evaluate_expr, ExpressionEvaluationError};
use crate::df_util::*;
use arrow::datatypes::i256;
use core::ops::{Add, Div, Mul, Not, Sub};
use datafusion::{
    common::ScalarValue,
    logical_expr::{expr::Placeholder, BinaryExpr, Expr, Operator},
};
use proof_of_sql::{
    base::{
        database::{owned_table_utility::*, ColumnOperationError, OwnedColumn, OwnedTable},
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    },
    proof_primitive::dory::DoryScalar,
};

#[test]
fn we_can_evaluate_a_literal() {
    let table: OwnedTable<DoryScalar> =
        owned_table([varchar("languages", ["en", "es", "pt", "fr", "ht"])]);

    // "Space and Time" in Hebrew
    let expr = Expr::Literal(ScalarValue::Utf8(Some("מרחב וזמן".to_string())));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::VarChar(vec!["מרחב וזמן".to_string(); 5]);
    assert_eq!(actual_column, expected_column);

    // "Space and Time" in Hebrew as UTF-8 bytes
    let expr = Expr::Literal(ScalarValue::Binary(Some(vec![
        0xd7, 0x9e, 0xd7, 0xa8, 0xd7, 0x97, 0xd7, 0x91, 0x20, 0xd7, 0x95, 0xd7, 0x96, 0xd7, 0x9e,
        0xd7, 0x9f,
    ])));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::VarBinary(vec![
        vec![
            0xd7, 0x9e, 0xd7, 0xa8, 0xd7, 0x97, 0xd7, 0x91, 0x20, 0xd7, 0x95, 0xd7, 0x96, 0xd7,
            0x9e, 0xd7, 0x9f,
        ];
        5
    ]);
    assert_eq!(actual_column, expected_column);

    // 1 as a tinyint
    let expr = Expr::Literal(ScalarValue::Int8(Some(1)));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::TinyInt(vec![1; 5]);
    assert_eq!(actual_column, expected_column);

    // -120 as a smallint
    let expr = Expr::Literal(ScalarValue::Int16(Some(-120)));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::SmallInt(vec![-120; 5]);
    assert_eq!(actual_column, expected_column);

    // i32::MAX as an int
    let expr = Expr::Literal(ScalarValue::Int32(Some(i32::MAX)));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Int(vec![i32::MAX; 5]);
    assert_eq!(actual_column, expected_column);

    // i64::MIN as a bigint
    let expr = Expr::Literal(ScalarValue::Int64(Some(i64::MIN)));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::BigInt(vec![i64::MIN; 5]);
    assert_eq!(actual_column, expected_column);

    // 255 as a uint8
    let expr = Expr::Literal(ScalarValue::UInt8(Some(255_u8)));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Uint8(vec![255_u8; 5]);
    assert_eq!(actual_column, expected_column);

    // Is Proof of SQL in production?
    let expr = Expr::Literal(ScalarValue::Boolean(Some(true)));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![true; 5]);
    assert_eq!(actual_column, expected_column);

    // Early days of Space and Time (2022-03-01T00:00:00Z)
    // Second
    let expr = Expr::Literal(ScalarValue::TimestampSecond(Some(1_646_092_800), None));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::TimestampTZ(
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        vec![1_646_092_800; 5],
    );
    assert_eq!(actual_column, expected_column);

    // Millisecond
    let expr = Expr::Literal(ScalarValue::TimestampMillisecond(
        Some(1_646_092_800_000),
        None,
    ));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::TimestampTZ(
        PoSQLTimeUnit::Millisecond,
        PoSQLTimeZone::utc(),
        vec![1_646_092_800_000; 5],
    );
    assert_eq!(actual_column, expected_column);

    // Microsecond
    let expr = Expr::Literal(ScalarValue::TimestampMicrosecond(
        Some(1_646_092_800_000_000),
        None,
    ));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::TimestampTZ(
        PoSQLTimeUnit::Microsecond,
        PoSQLTimeZone::utc(),
        vec![1_646_092_800_000_000; 5],
    );
    assert_eq!(actual_column, expected_column);

    // Nanosecond
    let expr = Expr::Literal(ScalarValue::TimestampNanosecond(
        Some(1_646_092_800_000_000_000),
        None,
    ));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::TimestampTZ(
        PoSQLTimeUnit::Nanosecond,
        PoSQLTimeZone::utc(),
        vec![1_646_092_800_000_000_000; 5],
    );
    assert_eq!(actual_column, expected_column);

    // A group of people has about 0.57 cats per person
    // Decimal128
    let expr = Expr::Literal(ScalarValue::Decimal128(Some(57.into()), 2, 2));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Decimal75(Precision::new(2).unwrap(), 2, vec![57.into(); 5]);
    assert_eq!(actual_column, expected_column);
    // Decimal256
    let expr = Expr::Literal(ScalarValue::Decimal256(Some(i256::from_i128(57)), 2, 2));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Decimal75(Precision::new(2).unwrap(), 2, vec![57.into(); 5]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_cannot_evaluate_a_literal_if_unsupported() {
    // Overly large i256
    let table: OwnedTable<DoryScalar> =
        owned_table([varchar("languages", ["en", "es", "pt", "fr", "ht"])]);
    let expr = Expr::Literal(ScalarValue::Decimal256(Some(i256::MAX), 2_u8, 2_i8));
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::Unsupported { .. })
    ));

    // Unsupported `ScalarValue`
    let expr = Expr::Literal(ScalarValue::IntervalDayTime(None));
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::Unsupported { .. })
    ));
}

#[test]
fn we_can_evaluate_a_column() {
    let table: OwnedTable<DoryScalar> = owned_table([
        bigint("bigints", [i64::MIN, -1, 0, 1, i64::MAX]),
        varchar("language", ["en", "es", "pt", "fr", "ht"]),
        varchar("john", ["John", "Juan", "João", "Jean", "Jean"]),
    ]);
    let expr = df_column("namespace.table_name", "bigints");
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::BigInt(vec![i64::MIN, -1, 0, 1, i64::MAX]);
    assert_eq!(actual_column, expected_column);

    let expr = df_column("namespace.table_name", "john");
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::VarChar(
        ["John", "Juan", "João", "Jean", "Jean"]
            .iter()
            .map(ToString::to_string)
            .collect(),
    );
    assert_eq!(actual_column, expected_column);

    // Try aliasing
    let alias = expr.alias("juan");
    let actual_column = evaluate_expr(&table, &alias).unwrap();
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_cannot_evaluate_a_nonexisting_column() {
    let table: OwnedTable<DoryScalar> =
        owned_table([varchar("cats", ["Chloe", "Margaret", "Prudence", "Lucy"])]);
    // "not_a_column" is not a column in the table
    let expr = df_column("namespace.table_name", "not_a_column");
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::ColumnNotFound { .. })
    ));
}

// NOT, AND, OR. Also EQ and NEQ.
#[test]
fn we_can_evaluate_a_logical_expression() {
    let table: OwnedTable<DoryScalar> = owned_table([
        varchar("en", ["Elizabeth", "John", "cat", "dog", "Munich"]),
        varchar("pl", ["Elżbieta", "Jan", "kot", "pies", "Monachium"]),
        varchar("cz", ["Alžběta", "Jan", "kočka", "pes", "Mnichov"]),
        varchar("sk", ["Alžbeta", "Ján", "mačka", "pes", "Mníchov"]),
        varchar("hr", ["Elizabeta", "Ivan", "mačka", "pas", "München"]),
        varchar("sl", ["Elizabeta", "Janez", "mačka", "pes", "München"]),
        boolean("is_proper_noun", [true, true, false, false, true]),
    ]);

    // Find words that are not proper nouns
    let expr = df_column("namespace.table_name", "is_proper_noun")
        .not()
        .alias("not_proper_noun");
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![false, false, true, true, false]);
    assert_eq!(actual_column, expected_column);

    // Which Czech and Slovak words agree?
    let expr = df_column("namespace.table_name", "cz").eq(df_column("namespace.table_name", "sk"));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column: OwnedColumn<DoryScalar> =
        OwnedColumn::Boolean(vec![false, false, false, true, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared among Slovak, Croatian and Slovenian
    let expr = (df_column("namespace.table_name", "sk")
        .eq(df_column("namespace.table_name", "hr")))
    .and(df_column("namespace.table_name", "hr").eq(df_column("namespace.table_name", "sl")))
    .alias("shared_sl_hr_sk");
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column: OwnedColumn<DoryScalar> =
        OwnedColumn::Boolean(vec![false, false, true, false, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared between Polish and Czech but not Slovenian
    let expr = (df_column("namespace.table_name", "pl")
        .eq(df_column("namespace.table_name", "cz")))
    .and(df_column("namespace.table_name", "pl").not_eq(df_column("namespace.table_name", "sl")));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column: OwnedColumn<DoryScalar> =
        OwnedColumn::Boolean(vec![false, true, false, false, false]);
    assert_eq!(actual_column, expected_column);

    // Proper nouns as well as words shared between Croatian, Slovenian and Slovak
    let expr = df_column("namespace.table_name", "is_proper_noun").or((df_column(
        "namespace.table_name",
        "hr",
    )
    .eq(df_column("namespace.table_name", "sl")))
    .and(df_column("namespace.table_name", "hr").eq(df_column("namespace.table_name", "sk"))));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column: OwnedColumn<DoryScalar> =
        OwnedColumn::Boolean(vec![true, true, true, false, true]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_a_comparison_expression() {
    let table: OwnedTable<DoryScalar> = owned_table([
        smallint("smallints", [-2_i16, -1, 0, 1, 2]),
        int("ints", [-4_i32, -2, 0, 2, 4]),
        bigint("bigints", [-8_i64, -4, 0, 4, 8]),
        int128("int128s", [-16_i128, -8, 0, 8, 16]),
        decimal75("decimals", 2, 1, [0, 1, 2, 3, 4]),
    ]);

    // Are the smallints less than the ints?
    let expr = df_column("namespace.table_name", "smallints")
        .lt(df_column("namespace.table_name", "ints"));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![false, false, false, true, true]);
    assert_eq!(actual_column, expected_column);

    // Are the ints greater than or equal to the bigints?
    let expr = df_column("namespace.table_name", "ints")
        .gt_eq(df_column("namespace.table_name", "bigints"));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![true, true, true, false, false]);
    assert_eq!(actual_column, expected_column);

    // Are the bigints greater than the int128s?
    let expr = df_column("namespace.table_name", "bigints")
        .gt(df_column("namespace.table_name", "int128s"));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![true, true, false, false, false]);
    assert_eq!(actual_column, expected_column);

    // Are the int128s less than or equal to the decimals?
    let expr = df_column("namespace.table_name", "int128s")
        .lt_eq(df_column("namespace.table_name", "decimals"));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::Boolean(vec![true, true, true, false, false]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_an_arithmetic_expression() {
    let table: OwnedTable<DoryScalar> = owned_table([
        smallint("smallints", [-2_i16, -1, 0, 1, 2]),
        int("ints", [-4_i32, -2, 0, 2, 4]),
        bigint("bigints", [-8_i64, -4, 0, 4, 8]),
        int128("int128s", [-16_i128, -8, 0, 8, 16]),
        decimal75("decimals", 2, 1, [0, 1, 2, 3, 4]),
    ]);

    // Subtract 1 from the bigints
    let expr = df_column("namespace.table_name", "bigints")
        .sub(Expr::Literal(ScalarValue::Int64(Some(1))));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::BigInt(vec![-9, -5, -1, 3, 7]);
    assert_eq!(actual_column, expected_column);

    // Add bigints to the smallints and multiply the sum by the ints
    let expr = (df_column("namespace.table_name", "smallints")
        .add(df_column("namespace.table_name", "bigints")))
    .mul(df_column("namespace.table_name", "ints"));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_column = OwnedColumn::BigInt(vec![40, 10, 0, 10, 40]);
    assert_eq!(actual_column, expected_column);

    // Multiply decimals with 0.75 and add smallints to the product
    let expr =
        df_column("namespace.table_name", "smallints").add(
            df_column("namespace.table_name", "decimals")
                .mul(Expr::Literal(ScalarValue::Decimal128(Some(75_i128), 2, 2))),
        );
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_scalars = [-2000, -925, 150, 1225, 2300]
        .iter()
        .map(|&x| x.into())
        .collect();
    let expected_column = OwnedColumn::Decimal75(Precision::new(9).unwrap(), 3, expected_scalars);
    assert_eq!(actual_column, expected_column);

    // Decimals over 2.5 plus int128s
    let expr = df_column("namespace.table_name", "decimals")
        .div(Expr::Literal(ScalarValue::Decimal128(Some(25_i128), 2, 1)))
        .add(df_column("namespace.table_name", "int128s"));
    let actual_column = evaluate_expr(&table, &expr).unwrap();
    let expected_scalars = [-16_000_000, -7_960_000, 80000, 8_120_000, 16_160_000]
        .iter()
        .map(|&x| x.into())
        .collect();
    let expected_column = OwnedColumn::Decimal75(Precision::new(46).unwrap(), 6, expected_scalars);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_cannot_evaluate_an_expression_if_expr_variant_not_supported() {
    let table: OwnedTable<DoryScalar> = owned_table([bigint("bigints", [1, 2, 3, 4, 5])]);
    // Placeholder
    let expr = Expr::Placeholder(Placeholder::new("$1".to_string(), None));
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::Unsupported { .. })
    ));
}

#[test]
fn we_cannot_evaluate_an_expression_if_binary_operator_not_supported() {
    let table: OwnedTable<DoryScalar> = owned_table([bigint("bigints", [1, 2, 3, 4, 5])]);
    // ArrowAt is not supported
    let expr = Expr::BinaryExpr(BinaryExpr::new(
        Box::new(df_column("namespace.table_name", "bigints")),
        Operator::ArrowAt,
        Box::new(df_column("namespace.table_name", "bigints")),
    ));
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::Unsupported { .. })
    ));
}

#[test]
fn we_cannot_evaluate_expressions_if_column_operation_errors_out() {
    let table: OwnedTable<DoryScalar> = owned_table([
        bigint("bigints", [i64::MIN, -1, 0, 1, i64::MAX]),
        varchar("language", ["en", "es", "pt", "fr", "ht"]),
        varchar("sarah", ["Sarah", "Sara", "Sara", "Sarah", "Sarah"]),
    ]);

    // NOT doesn't work on varchar
    let expr = df_column("namespace.table_name", "sarah").not();
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::UnaryOperationInvalidColumnType { .. }
        })
    ));

    // NOT doesn't work on bigint
    let expr = df_column("namespace.table_name", "bigints").not();
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::UnaryOperationInvalidColumnType { .. }
        })
    ));

    // + doesn't work on varchar
    let expr =
        df_column("namespace.table_name", "sarah").add(df_column("namespace.table_name", "sarah"));
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::BinaryOperationInvalidColumnType { .. }
        })
    ));

    // i64::MIN - 1 overflows
    let expr = df_column("namespace.table_name", "bigints")
        .sub(Expr::Literal(ScalarValue::Int64(Some(1))));
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::IntegerOverflow { .. }
        })
    ));

    // We can't divide by zero
    let expr = df_column("namespace.table_name", "bigints")
        .div(Expr::Literal(ScalarValue::Int64(Some(0))));
    assert!(matches!(
        evaluate_expr(&table, &expr),
        Err(ExpressionEvaluationError::ColumnOperationError {
            source: ColumnOperationError::DivisionByZero
        })
    ));
}
