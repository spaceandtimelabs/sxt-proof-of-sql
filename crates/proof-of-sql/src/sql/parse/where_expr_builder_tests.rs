use crate::{
    base::{
        database::{ColumnRef, ColumnType, LiteralValue, TableRef, TestSchemaAccessor},
        map::{indexmap, IndexMap},
        math::decimal::Precision,
    },
    sql::{
        parse::{ConversionError, QueryExpr, WhereExprBuilder},
        proof_exprs::{ColumnExpr, DynProofExpr, LiteralExpr},
    },
};
use bigdecimal::BigDecimal;
use core::str::FromStr;
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestamp},
    utility::*,
    SelectStatement,
};
use sqlparser::ast::Ident;

/// # Panics
///
/// Will panic if:
/// - The parsing of the table reference `"sxt.sxt_tab"` fails, which would occur if the input
///   string does not adhere to the expected format for identifiers. This is because `parse()`
///   is called on the identifier string and `unwrap()` is used to handle the result.
/// - The precision used for creating the `Decimal75` column type fails. The `Precision::new(7)`
///   call is expected to succeed; however, if it encounters an invalid precision value, it will
///   cause a panic when `unwrap()` is called.
fn get_column_mappings_for_testing() -> IndexMap<Ident, ColumnRef> {
    let tab_ref = TableRef::new("sxt", "sxt_tab");
    let mut column_mapping = IndexMap::default();
    // Setup column mapping
    column_mapping.insert(
        "boolean_column".into(),
        ColumnRef::new(
            tab_ref.clone(),
            "boolean_column".into(),
            ColumnType::Boolean,
        ),
    );
    column_mapping.insert(
        "decimal_column".into(),
        ColumnRef::new(
            tab_ref.clone(),
            "decimal_column".into(),
            ColumnType::Decimal75(Precision::new(7).unwrap(), 2),
        ),
    );
    column_mapping.insert(
        "int128_column".into(),
        ColumnRef::new(tab_ref.clone(), "int128_column".into(), ColumnType::Int128),
    );
    column_mapping.insert(
        "bigint_column".into(),
        ColumnRef::new(tab_ref.clone(), "bigint_column".into(), ColumnType::BigInt),
    );

    column_mapping.insert(
        "varchar_column".into(),
        ColumnRef::new(
            tab_ref.clone(),
            "varchar_column".into(),
            ColumnType::VarChar,
        ),
    );
    column_mapping.insert(
        "timestamp_second_column".into(),
        ColumnRef::new(
            tab_ref.clone(),
            "timestamp_second_column".into(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
        ),
    );
    column_mapping.insert(
        "timestamp_millisecond_column".into(),
        ColumnRef::new(
            tab_ref.clone(),
            "timestamp_millisecond_column".into(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc()),
        ),
    );
    column_mapping.insert(
        "timestamp_microsecond_column".into(),
        ColumnRef::new(
            tab_ref.clone(),
            "timestamp_microsecond_column".into(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Microsecond, PoSQLTimeZone::utc()),
        ),
    );
    column_mapping.insert(
        "timestamp_nanosecond_column".into(),
        ColumnRef::new(
            tab_ref.clone(),
            "timestamp_nanosecond_column".into(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc()),
        ),
    );
    column_mapping
}

#[test]
fn we_can_directly_check_whether_boolean_column_is_true() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_boolean = col("boolean_column");
    assert!(builder.build(Some(expr_boolean)).is_ok());
}

#[test]
fn we_can_directly_check_whether_boolean_literal_is_true() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_boolean = lit(false);
    assert!(builder.build(Some(expr_boolean)).is_ok());
}

#[test]
fn we_can_directly_check_nested_eq() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_nested = equal(
        col("boolean_column"),
        equal(col("bigint_column"), col("int128_column")),
    );
    assert!(builder.build(Some(expr_nested)).is_ok());
}

#[test]
fn we_can_directly_check_whether_boolean_columns_eq_boolean() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_boolean_to_boolean = equal(col("boolean_column"), lit(false));
    assert!(builder.build(Some(expr_boolean_to_boolean)).is_ok());
}

#[test]
fn we_can_directly_check_whether_integer_columns_eq_integer() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_integer_to_integer = equal(col("int128_column"), lit(12345_i128));
    assert!(builder.build(Some(expr_integer_to_integer)).is_ok());
}

#[test]
fn we_can_directly_check_whether_bigint_columns_ge_int128() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_integer_to_integer = ge(col("bigint_column"), lit(-12345_i128));
    let actual = builder
        .build(Some(expr_integer_to_integer))
        .unwrap()
        .unwrap();
    let expected = DynProofExpr::try_new_not(
        DynProofExpr::try_new_inequality(
            DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                "bigint_column".into(),
                ColumnType::BigInt,
            ))),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Int128(-12345))),
            true,
        )
        .unwrap(),
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
        .build(Some(expr_integer_to_integer))
        .unwrap()
        .unwrap();
    let expected = DynProofExpr::try_new_not(
        DynProofExpr::try_new_inequality(
            DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                "bigint_column".into(),
                ColumnType::BigInt,
            ))),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Int128(-12345))),
            false,
        )
        .unwrap(),
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
    let result = builder.build(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_check_non_decimal_columns_eq_integer_literals() {
    let column_mapping = get_column_mappings_for_testing();
    // Non-decimal column with integer literal
    let expr = equal(col("bigint_column"), lit(12345_i64));
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_check_scaled_integers_eq_correctly() {
    let column_mapping = get_column_mappings_for_testing();
    // Decimal column with integer literal that can be appropriately scaled
    let expr = equal(col("decimal_column"), lit(12345_i128));
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_can_check_exact_scale_and_precision_eq() {
    let column_mapping = get_column_mappings_for_testing();
    // Decimal column with matching scale decimal literal
    let expr = equal(
        col("decimal_column"),
        lit("123.45".parse::<BigDecimal>().unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build(Some(expr));
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
    let result = builder.build(Some(expr));
    assert!(result.is_ok());

    let expr = equal(
        col("timestamp_microsecond_column"),
        lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123456Z").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build(Some(expr));
    assert!(result.is_ok());

    let expr = equal(
        col("timestamp_millisecond_column"),
        lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123Z").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build(Some(expr));
    assert!(result.is_ok());

    let expr = equal(
        col("timestamp_second_column"),
        lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00Z").unwrap()),
    );
    let builder = WhereExprBuilder::new(&column_mapping);
    let result = builder.build(Some(expr));
    assert!(result.is_ok());
}

#[test]
fn we_cannot_have_missing_column_as_where_clause() {
    let column_mapping = get_column_mappings_for_testing();
    let builder = WhereExprBuilder::new(&column_mapping);
    let expr_missing = col("not_a_column");
    let res = builder.build(Some(expr_missing));
    assert!(matches!(
        res,
        Result::Err(ConversionError::MissingColumnWithoutTable { .. })
    ));
}

#[test]
fn we_cannot_have_non_boolean_column_as_where_clause() {
    let column_mapping = get_column_mappings_for_testing();

    let builder = WhereExprBuilder::new(&column_mapping);

    let expr_non_boolean = col("varchar_column");
    let res = builder.build(Some(expr_non_boolean));
    assert!(matches!(
        res,
        Result::Err(ConversionError::NonbooleanWhereClause { .. })
    ));
}

#[test]
fn we_cannot_have_non_boolean_literal_as_where_clause() {
    let column_mapping = IndexMap::default();

    let builder = WhereExprBuilder::new(&column_mapping);

    let expr_non_boolean = lit(123_i128);
    let res = builder.build(Some(expr_non_boolean));
    assert!(matches!(
        res,
        Result::Err(ConversionError::NonbooleanWhereClause { .. })
    ));
}

#[test]
fn we_expect_an_error_while_trying_to_check_varchar_column_eq_decimal() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = TestSchemaAccessor::new(indexmap! {
        t.clone() => indexmap! {
            "b".into() => ColumnType::VarChar,
        },
    });

    assert!(matches!(
        QueryExpr::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123").unwrap(),
            t.schema_id().cloned().unwrap(),
            &accessor,
        ),
        Err(ConversionError::DataTypeMismatch { .. })
    ));
}

#[test]
fn we_expect_an_error_while_trying_to_check_varchar_column_ge_decimal() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = TestSchemaAccessor::new(indexmap! {
        t.clone() => indexmap! {
            "b".into() => ColumnType::VarChar,
        },
    });

    assert!(matches!(
        QueryExpr::try_new(
            SelectStatement::from_str("select * from sxt_tab where b >= 123").unwrap(),
            t.schema_id().cloned().unwrap(),
            &accessor,
        ),
        Err(ConversionError::DataTypeMismatch { .. })
    ));
}

#[test]
fn we_do_not_expect_an_error_while_trying_to_check_int128_column_eq_decimal_with_zero_scale() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = TestSchemaAccessor::new(indexmap! {
        t.clone() => indexmap! {
            "b".into() => ColumnType::Int128,
        },
    });

    assert!(QueryExpr::try_new(
        SelectStatement::from_str("select * from sxt_tab where b = 123.000").unwrap(),
        t.schema_id().cloned().unwrap(),
        &accessor,
    )
    .is_ok());
}

#[test]
fn we_do_not_expect_an_error_while_trying_to_check_bigint_column_eq_decimal_with_zero_scale() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = TestSchemaAccessor::new(indexmap! {
        t.clone() => indexmap! {
            "b".into() => ColumnType::BigInt,
        },
    });

    assert!(QueryExpr::try_new(
        SelectStatement::from_str("select * from sxt_tab where b = 123.000").unwrap(),
        t.schema_id().cloned().unwrap(),
        &accessor,
    )
    .is_ok());
}
