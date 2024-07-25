use crate::{
    base::{
        database::{owned_table_utility::*, OwnedColumn},
        math::decimal::Precision,
        scalar::Curve25519Scalar,
    },
    sql::postprocessing::{PostprocessingError, PostprocessingEvaluator},
};
use proof_of_sql_parser::{
    intermediate_ast::{BinaryOperator, Expression, Literal, UnaryOperator},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestamp},
};

#[test]
fn we_can_evaluate_a_simple_literal() {
    let table = owned_table([varchar("languages", ["en", "es", "pt", "fr", "ht"])]);

    // "Space and Time" in Hebrew
    let expr = Expression::Literal(Literal::VarChar("מרחב וזמן".to_string()));
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::VarChar(vec!["מרחב וזמן".to_string(); 5]);
    assert_eq!(actual_column, expected_column);

    // Is Proof of SQL in production?
    let expr = Expression::Literal(Literal::Boolean(true));
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::Boolean(vec![true; 5]);
    assert_eq!(actual_column, expected_column);

    // When was Space and Time founded?
    let timestamp = "2022-03-01T00:00:00Z";
    let expr = Expression::Literal(Literal::Timestamp(
        PoSQLTimestamp::try_from(timestamp).unwrap(),
    ));
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    // UNIX timestamp for 2022-03-01T00:00:00Z
    let actual_timestamp = 1646092800;
    let expected_column = OwnedColumn::TimestampTZ(
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::Utc,
        vec![actual_timestamp; 5],
    );
    assert_eq!(actual_column, expected_column);

    // A group of people has about 0.67 cats per person
    let expr = Expression::Literal(Literal::Decimal("0.67".parse().unwrap()));
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::Decimal75(Precision::new(2).unwrap(), 2, vec![67.into(); 5]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_a_simple_column() {
    let table = owned_table([
        bigint("bigints", [i64::MIN, -1, 0, 1, i64::MAX]),
        varchar("language", ["en", "es", "pt", "fr", "ht"]),
        varchar("john", ["John", "Juan", "João", "Jean", "Jean"]),
    ]);
    let expr = Expression::Column("bigints".parse().unwrap());
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::BigInt(vec![i64::MIN, -1, 0, 1, i64::MAX]);
    assert_eq!(actual_column, expected_column);

    let expr = Expression::Column("john".parse().unwrap());
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::VarChar(
        ["John", "Juan", "João", "Jean", "Jean"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_not_evaluate_a_nonexisting_column() {
    let table = owned_table([varchar("cats", ["Chloe", "Margaret", "Prudence", "Lucy"])]);
    // "not_a_column" is not a column in the table
    let expr = Expression::Column("not_a_column".parse().unwrap());
    assert!(matches!(
        PostprocessingEvaluator::<Curve25519Scalar>::new(&table).evaluate(&expr),
        Err(PostprocessingError::ColumnNotFound(_))
    ));
}

#[test]
fn we_can_evaluate_a_logical_expression() {
    let table = owned_table([
        varchar("en", ["Elizabeth", "John", "cat", "dog", "Munich"]),
        varchar("pl", ["Elżbieta", "Jan", "kot", "pies", "Monachium"]),
        varchar("cz", ["Alžběta", "Jan", "kočka", "pes", "Mnichov"]),
        varchar("sk", ["Alžbeta", "Ján", "mačka", "pes", "Mníchov"]),
        varchar("hr", ["Elizabeta", "Ivan", "mačka", "pas", "München"]),
        varchar("sl", ["Elizabeta", "Janez", "mačka", "pes", "München"]),
        boolean("is_proper_noun", [true, true, false, false, true]),
    ]);

    // Find words that are not proper nouns
    let expr = Expression::Unary {
        op: UnaryOperator::Not,
        expr: Box::new(Expression::Column("is_proper_noun".parse().unwrap())),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::Boolean(vec![false, false, true, true, false]);
    assert_eq!(actual_column, expected_column);

    // Which Czech and Slovak words agree?
    let expr = Expression::Binary {
        op: BinaryOperator::Equal,
        left: Box::new(Expression::Column("cz".parse().unwrap())),
        right: Box::new(Expression::Column("sk".parse().unwrap())),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![false, false, false, true, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared among Slovak, Croatian and Slovenian
    let expr = Expression::Binary {
        op: BinaryOperator::And,
        left: Box::new(Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column("sk".parse().unwrap())),
            right: Box::new(Expression::Column("hr".parse().unwrap())),
        }),
        right: Box::new(Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column("hr".parse().unwrap())),
            right: Box::new(Expression::Column("sl".parse().unwrap())),
        }),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![false, false, true, false, false]);
    assert_eq!(actual_column, expected_column);

    // Find words shared between Polish and Czech but not Slovenian
    let expr = Expression::Binary {
        op: BinaryOperator::And,
        left: Box::new(Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column("pl".parse().unwrap())),
            right: Box::new(Expression::Column("cz".parse().unwrap())),
        }),
        right: Box::new(Expression::Unary {
            op: UnaryOperator::Not,
            expr: Box::new(Expression::Binary {
                op: BinaryOperator::Equal,
                left: Box::new(Expression::Column("pl".parse().unwrap())),
                right: Box::new(Expression::Column("sl".parse().unwrap())),
            }),
        }),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![false, true, false, false, false]);
    assert_eq!(actual_column, expected_column);

    // Proper nouns as well as words shared between Croatian and Slovenian
    let expr = Expression::Binary {
        op: BinaryOperator::Or,
        left: Box::new(Expression::Column("is_proper_noun".parse().unwrap())),
        right: Box::new(Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column("hr".parse().unwrap())),
            right: Box::new(Expression::Column("sl".parse().unwrap())),
        }),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column: OwnedColumn<Curve25519Scalar> =
        OwnedColumn::Boolean(vec![true, true, true, false, true]);
    assert_eq!(actual_column, expected_column);
}

#[test]
fn we_can_evaluate_an_arithmetic_expression() {
    let table = owned_table([
        smallint("smallints", [-2_i16, -1, 0, 1, 2]),
        int("ints", [-4_i32, -2, 0, 2, 4]),
        bigint("bigints", [-8_i64, -4, 0, 4, 8]),
        decimal75("decimals", 2, 1, [0, 1, 2, 3, 4]),
    ]);

    // Subtract 1 from the bigints
    let expr = Expression::Binary {
        op: BinaryOperator::Subtract,
        left: Box::new(Expression::Column("bigints".parse().unwrap())),
        right: Box::new(Expression::Literal(Literal::BigInt(1))),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::BigInt(vec![-9, -5, -1, 3, 7]);
    assert_eq!(actual_column, expected_column);

    // Add bigints to the smallints and multiply the sum by the ints
    let expr = Expression::Binary {
        op: BinaryOperator::Multiply,
        left: Box::new(Expression::Binary {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Column("smallints".parse().unwrap())),
            right: Box::new(Expression::Column("bigints".parse().unwrap())),
        }),
        right: Box::new(Expression::Column("ints".parse().unwrap())),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_column = OwnedColumn::BigInt(vec![40, 10, 0, 10, 40]);
    assert_eq!(actual_column, expected_column);

    // Multiply decimals with 0.75 and add smallints to the product
    let expr = Expression::Binary {
        op: BinaryOperator::Add,
        left: Box::new(Expression::Column("smallints".parse().unwrap())),
        right: Box::new(Expression::Binary {
            op: BinaryOperator::Multiply,
            left: Box::new(Expression::Column("decimals".parse().unwrap())),
            right: Box::new(Expression::Literal(Literal::Decimal(
                "0.75".parse().unwrap(),
            ))),
        }),
    };
    let actual_column = PostprocessingEvaluator::<Curve25519Scalar>::new(&table)
        .evaluate(&expr)
        .unwrap();
    let expected_scalars = [-2000, -925, 150, 1225, 2300]
        .iter()
        .map(|&x| x.into())
        .collect();
    let expected_column = OwnedColumn::Decimal75(Precision::new(9).unwrap(), 3, expected_scalars);
    assert_eq!(actual_column, expected_column);
}
