use crate::{
    base::{
        database::{ColumnRef, ColumnType, LiteralValue, TestSchemaAccessor},
        map::{indexmap, IndexMap},
        math::decimal::Precision,
    },
    sql::{
        parse::{ConversionError, QueryExpr, WhereExprBuilder},
        proof_exprs::{ColumnExpr, DynProofExpr, LiteralExpr},
    },
};
use curve25519_dalek::RistrettoPoint;
use proof_of_sql_parser::{
    intermediate_decimal::IntermediateDecimal,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestamp},
    utility::*,
    Identifier, SelectStatement,
};
use std::str::FromStr;

fn get_column_mappings_for_testing() -> IndexMap<Identifier, ColumnRef> {
    let tab_ref = "sxt.sxt_tab".parse().unwrap();
    let mut column_mapping = IndexMap::default();
    // Setup column mapping
    column_mapping.insert(
        ident("boolean_column"),
        ColumnRef::new(tab_ref, ident("boolean_column"), ColumnType::Boolean),
    );
    column_mapping.insert(
        ident("decimal_column"),
        ColumnRef::new(
            tab_ref,
            ident("decimal_column"),
            ColumnType::Decimal75(Precision::new(7).unwrap(), 2),
        ),
    );
    column_mapping.insert(
        ident("int128_column"),
        ColumnRef::new(tab_ref, ident("int128_column"), ColumnType::Int128),
    );
    column_mapping.insert(
        ident("bigint_column"),
        ColumnRef::new(tab_ref, ident("bigint_column"), ColumnType::BigInt),
    );

    column_mapping.insert(
        ident("varchar_column"),
        ColumnRef::new(tab_ref, ident("varchar_column"), ColumnType::VarChar),
    );
    column_mapping.insert(
        ident("timestamp_second_column"),
        ColumnRef::new(
            tab_ref,
            ident("timestamp_second_column"),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc),
        ),
    );
    column_mapping.insert(
        ident("timestamp_millisecond_column"),
        ColumnRef::new(
            tab_ref,
            ident("timestamp_millisecond_column"),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::Utc),
        ),
    );
    column_mapping.insert(
        ident("timestamp_microsecond_column"),
        ColumnRef::new(
            tab_ref,
            ident("timestamp_microsecond_column"),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Microsecond, PoSQLTimeZone::Utc),
        ),
    );
    column_mapping.insert(
        ident("timestamp_nanosecond_column"),
        ColumnRef::new(
            tab_ref,
            ident("timestamp_nanosecond_column"),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::Utc),
        ),
    );
    column_mapping
}

#[test]
fn we_can_directly_check_whether_boolean_column_is_true() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_boolean = col("boolean_column");
    assert!(builder.build::<RistrettoPoint>(Some(expr_boolean)).is_ok());
}

#[test]
fn we_can_directly_check_whether_boolean_literal_is_true() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_boolean = lit(false);
    assert!(builder.build::<RistrettoPoint>(Some(expr_boolean)).is_ok());
}

#[test]
fn we_can_directly_check_nested_eq() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_nested = equal(
        col("boolean_column"),
        equal(col("bigint_column"), col("int128_column")),
    );
    assert!(builder.build::<RistrettoPoint>(Some(expr_nested)).is_ok());
}

#[test]
fn we_can_directly_check_whether_boolean_columns_eq_boolean() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_boolean_to_boolean = equal(col("boolean_column"), lit(false));
    assert!(builder
        .build::<RistrettoPoint>(Some(expr_boolean_to_boolean))
        .is_ok());
}

#[test]
fn we_can_directly_check_whether_integer_columns_eq_integer() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_integer_to_integer = equal(col("int128_column"), lit(12345_i128));
    assert!(builder
        .build::<RistrettoPoint>(Some(expr_integer_to_integer))
        .is_ok());
}

#[test]
fn we_can_directly_check_whether_bigint_columns_ge_int128() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_integer_to_integer = ge(col("bigint_column"), lit(-12345_i128));
    let actual = builder
        .build::<RistrettoPoint>(Some(expr_integer_to_integer))
        .unwrap()
        .unwrap();
    let expected = DynProofExpr::try_new_inequality(
        DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
            "sxt.sxt_tab".parse().unwrap(),
            ident("bigint_column"),
            ColumnType::BigInt,
        ))),
        DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Int128(-12345))),
        false,
    )
    .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn we_can_directly_check_whether_bigint_columns_le_int128() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_integer_to_integer = le(col("bigint_column"), lit(-12345_i128));
    let actual = builder
        .build::<RistrettoPoint>(Some(expr_integer_to_integer))
        .unwrap()
        .unwrap();
    let expected = DynProofExpr::try_new_inequality(
        DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
            "sxt.sxt_tab".parse().unwrap(),
            ident("bigint_column"),
            ColumnType::BigInt,
        ))),
        DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Int128(-12345))),
        true,
    )
    .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn we_can_directly_check_whether_varchar_columns_eq_varchar() {
    let column_mapping = get_column_mappings_for_testing();
    // VarChar column with VarChar literal
    let expr = equal(col("varchar_column"), lit("test_string"));
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_check_non_decimal_columns_eq_integer_literals() {
    let column_mapping = get_column_mappings_for_testing();
    // Non-decimal column with integer literal
    let expr = equal(col("bigint_column"), lit(12345_i64));
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_check_scaled_integers_eq_correctly() {
    let column_mapping = get_column_mappings_for_testing();
    // Decimal column with integer literal that can be appropriately scaled
    let expr = equal(col("decimal_column"), lit(12345_i128));
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_check_exact_scale_and_precision_eq() {
    let column_mapping = get_column_mappings_for_testing();
    // Decimal column with matching scale decimal literal
    let expr = equal(
        col("decimal_column"),
        lit(IntermediateDecimal::try_from("123.45").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_check_varying_precision_eq_for_timestamp() {
    let column_mapping = get_column_mappings_for_testing();

    let expr = equal(
        col("timestamp_nanosecond_column"),
        lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123456789Z").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());

    let expr = equal(
        col("timestamp_microsecond_column"),
        lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123456Z").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());

    let expr = equal(
        col("timestamp_millisecond_column"),
        lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123Z").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());

    let expr = equal(
        col("timestamp_second_column"),
        lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00Z").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build::<RistrettoPoint>(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_not_have_missing_column_as_where_clause() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_missing = col("not_a_column");
    let res = builder.build::<RistrettoPoint>(Some(expr_missing));
    assert!(matches!(
        res,
        Result::Err(ConversionError::MissingColumnWithoutTable(_))
    ));
}

#[test]
fn we_can_not_have_non_boolean_column_as_where_clause() {
    let column_mapping = get_column_mappings_for_testing();

    let builder = WhereExprBuilder::new(&column_mapping);

    let expr_non_boolean = col("varchar_column");
    let res = builder.build::<RistrettoPoint>(Some(expr_non_boolean));
    assert!(matches!(
        res,
        Result::Err(ConversionError::NonbooleanWhereClause(_))
    ));
}

#[test]
fn we_can_not_have_non_boolean_literal_as_where_clause() {
    let column_mapping = IndexMap::default();

    let builder = WhereExprBuilder::new(&column_mapping);

    let expr_non_boolean = lit(123_i128);
    let res = builder.build::<RistrettoPoint>(Some(expr_non_boolean));
    assert!(matches!(
        res,
        Result::Err(ConversionError::NonbooleanWhereClause(_))
    ));
}

#[test]
fn we_expect_an_error_while_trying_to_check_varchar_column_eq_decimal() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = TestSchemaAccessor::new(indexmap! {
        t => indexmap! {
            "b".parse().unwrap() => ColumnType::VarChar,
        },
    });

    assert!(matches!(
        QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123").unwrap(),
            t.schema_id(),
            &accessor,
        ),
        Err(ConversionError::DataTypeMismatch(_, _))
    ));
}

#[test]
fn we_expect_an_error_while_trying_to_check_varchar_column_ge_decimal() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = TestSchemaAccessor::new(indexmap! {
        t => indexmap! {
            "b".parse().unwrap() => ColumnType::VarChar,
        },
    });

    assert!(matches!(
        QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b >= 123").unwrap(),
            t.schema_id(),
            &accessor,
        ),
        Err(ConversionError::DataTypeMismatch(_, _))
    ));
}

#[test]
fn we_do_not_expect_an_error_while_trying_to_check_int128_column_eq_decimal_with_zero_scale() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = TestSchemaAccessor::new(indexmap! {
        t => indexmap! {
            "b".parse().unwrap() => ColumnType::Int128,
        },
    });

    assert!(QueryExpr::<RistrettoPoint>::try_new(
        SelectStatement::from_str("select * from sxt_tab where b = 123.000").unwrap(),
        t.schema_id(),
        &accessor,
    )
    .is_ok());
}

#[test]
fn we_do_not_expect_an_error_while_trying_to_check_bigint_column_eq_decimal_with_zero_scale() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = TestSchemaAccessor::new(indexmap! {
        t => indexmap! {
            "b".parse().unwrap() => ColumnType::BigInt,
        },
    });

    assert!(QueryExpr::<RistrettoPoint>::try_new(
        SelectStatement::from_str("select * from sxt_tab where b = 123.000").unwrap(),
        t.schema_id(),
        &accessor,
    )
    .is_ok());
}
