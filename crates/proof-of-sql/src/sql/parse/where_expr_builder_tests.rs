#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use crate::{
        base::{
            database::{ColumnRef, ColumnType, LiteralValue},
            math::decimal::Precision,
        },
        record_batch,
        sql::{
            ast::{ColumnExpr, LiteralExpr, ProvableExprPlan},
            parse::{
                query_expr_tests::record_batch_to_accessor, ConversionError, QueryExpr,
                WhereExprBuilder,
            },
        },
    };
    use curve25519_dalek::RistrettoPoint;
    use indexmap::IndexMap;
    use proof_of_sql_parser::{
        intermediate_ast::{BinaryOperator, Expression, Literal},
        intermediate_decimal::IntermediateDecimal,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestamp},
        utility::{col, equal, lit},
        Identifier, SelectStatement,
    };
    use std::str::FromStr;

    fn run_test_case(column_mapping: &IndexMap<Identifier, ColumnRef>, expr: Expression) {
        let builder = WhereExprBuilder::new(column_mapping);
        let result = builder.build::<RistrettoPoint>(Some(Box::new(expr)));
        assert!(result.is_ok(), "Test case should succeed without panic.");
    }

    fn get_column_mappings_for_testing() -> IndexMap<Identifier, ColumnRef> {
        let mut column_mapping = IndexMap::new();
        // Setup column mapping
        column_mapping.insert(
            Identifier::try_new("boolean_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("boolean_column").unwrap(),
                ColumnType::Boolean,
            ),
        );
        column_mapping.insert(
            Identifier::try_new("decimal_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("decimal_column").unwrap(),
                ColumnType::Decimal75(Precision::new(7).unwrap(), 2),
            ),
        );
        column_mapping.insert(
            Identifier::try_new("int128_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("int128_column").unwrap(),
                ColumnType::Int128,
            ),
        );
        column_mapping.insert(
            Identifier::try_new("bigint_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("bigint_column").unwrap(),
                ColumnType::BigInt,
            ),
        );

        column_mapping.insert(
            Identifier::try_new("varchar_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("varchar_column").unwrap(),
                ColumnType::VarChar,
            ),
        );
        column_mapping.insert(
            Identifier::try_new("timestamp_second_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("timestamp_second_column").unwrap(),
                ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc),
            ),
        );
        column_mapping.insert(
            Identifier::try_new("timestamp_millisecond_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("timestamp_millisecond_column").unwrap(),
                ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::Utc),
            ),
        );
        column_mapping.insert(
            Identifier::try_new("timestamp_microsecond_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("timestamp_microsecond_column").unwrap(),
                ColumnType::TimestampTZ(PoSQLTimeUnit::Microsecond, PoSQLTimeZone::Utc),
            ),
        );
        column_mapping.insert(
            Identifier::try_new("timestamp_nanosecond_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("timestamp_nanosecond_column").unwrap(),
                ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::Utc),
            ),
        );
        column_mapping
    }

    #[test]
    fn we_can_directly_check_whether_boolean_column_is_true() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_boolean = Expression::Column(Identifier::try_new("boolean_column").unwrap());
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr_boolean)))
            .is_ok());
    }

    #[test]
    fn we_can_directly_check_whether_boolean_literal_is_true() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_boolean = Expression::Literal(Literal::Boolean(false));
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr_boolean)))
            .is_ok());
    }

    #[test]
    fn we_can_directly_check_nested_eq() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_nested = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("boolean_column").unwrap(),
            )),
            right: Box::new(Expression::Binary {
                op: BinaryOperator::Equal,
                left: Box::new(Expression::Column(
                    Identifier::try_new("bigint_column").unwrap(),
                )),
                right: Box::new(Expression::Column(
                    Identifier::try_new("int128_column").unwrap(),
                )),
            }),
        };
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr_nested)))
            .is_ok());
    }

    #[test]
    fn we_can_directly_check_whether_boolean_columns_eq_boolean() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_boolean_to_boolean = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("boolean_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Boolean(false))),
        };
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr_boolean_to_boolean)))
            .is_ok());
    }

    #[test]
    fn we_can_directly_check_whether_integer_columns_eq_integer() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("int128_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr_integer_to_integer)))
            .is_ok());
    }

    #[test]
    fn we_can_directly_check_whether_bigint_columns_ge_int128() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::GreaterThanOrEqual,
            left: Box::new(Expression::Column(
                Identifier::try_new("bigint_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(-12345))),
        };
        let actual = builder
            .build::<RistrettoPoint>(Some(Box::new(expr_integer_to_integer)))
            .unwrap()
            .unwrap();
        println!("{:?}", actual);
        let expected = ProvableExprPlan::try_new_inequality(
            ProvableExprPlan::Column(ColumnExpr::new(ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("bigint_column").unwrap(),
                ColumnType::BigInt,
            ))),
            ProvableExprPlan::Literal(LiteralExpr::new(LiteralValue::Int128(-12345))),
            false,
        )
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn we_can_directly_check_whether_bigint_columns_le_int128() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::LessThanOrEqual,
            left: Box::new(Expression::Column(
                Identifier::try_new("bigint_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(-12345))),
        };
        let actual = builder
            .build::<RistrettoPoint>(Some(Box::new(expr_integer_to_integer)))
            .unwrap()
            .unwrap();
        let expected = ProvableExprPlan::try_new_inequality(
            ProvableExprPlan::Column(ColumnExpr::new(ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("bigint_column").unwrap(),
                ColumnType::BigInt,
            ))),
            ProvableExprPlan::Literal(LiteralExpr::new(LiteralValue::Int128(-12345))),
            true,
        )
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn we_can_directly_check_whether_varchar_columns_eq_varchar() {
        let column_mapping = get_column_mappings_for_testing();
        // VarChar column with VarChar literal
        let expr_varchar_to_varchar = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("varchar_column").unwrap(),
            )), // Ensure this column is defined in column_mapping
            right: Box::new(Expression::Literal(Literal::VarChar(
                "test_string".to_string(),
            ))),
        };

        run_test_case(&column_mapping, expr_varchar_to_varchar);
    }

    #[test]
    fn we_can_check_non_decimal_columns_eq_integer_literals() {
        let column_mapping = get_column_mappings_for_testing();

        // Non-decimal column with integer literal
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("int128_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        run_test_case(&column_mapping, expr_integer_to_integer);
    }

    #[test]
    fn we_can_check_scaled_integers_eq_correctly() {
        let column_mapping = get_column_mappings_for_testing();

        // Decimal column with integer literal that can be appropriately scaled
        let expr_integer_to_decimal = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("decimal_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        run_test_case(&column_mapping, expr_integer_to_decimal);
    }

    #[test]
    fn we_can_check_exact_scale_and_precision_eq() {
        let column_mapping = get_column_mappings_for_testing();

        // Decimal column with matching scale decimal literal
        let expr_decimal = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("decimal_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Decimal(
                IntermediateDecimal::try_from("123.45").unwrap(),
            ))),
        };
        run_test_case(&column_mapping, expr_decimal);
    }

    #[test]
    fn we_can_check_varying_precision_eq_for_timestamp() {
        let column_mapping = get_column_mappings_for_testing();

        run_test_case(
            &column_mapping,
            *equal(
                col("timestamp_nanosecond_column"),
                lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123456789Z").unwrap()),
            ),
        );

        run_test_case(
            &column_mapping,
            *equal(
                col("timestamp_microsecond_column"),
                lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123456Z").unwrap()),
            ),
        );

        run_test_case(
            &column_mapping,
            *equal(
                col("timestamp_millisecond_column"),
                lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00.123Z").unwrap()),
            ),
        );

        run_test_case(
            &column_mapping,
            *equal(
                col("timestamp_second_column"),
                lit(PoSQLTimestamp::try_from("1970-01-01T00:00:00Z").unwrap()),
            ),
        );
    }

    #[test]
    fn we_can_not_have_missing_column_as_where_clause() {
        let column_mapping = get_column_mappings_for_testing();

        let builder = WhereExprBuilder::new(&column_mapping);

        let expr_missing = Expression::Column(Identifier::try_new("not_a_column").unwrap());
        let res = builder.build::<RistrettoPoint>(Some(Box::new(expr_missing)));
        assert!(matches!(
            res,
            Result::Err(ConversionError::MissingColumnWithoutTable(_))
        ));
    }

    #[test]
    fn we_can_not_have_non_boolean_column_as_where_clause() {
        let column_mapping = get_column_mappings_for_testing();

        let builder = WhereExprBuilder::new(&column_mapping);

        let expr_non_boolean = Expression::Column(Identifier::try_new("varchar_column").unwrap());
        let res = builder.build::<RistrettoPoint>(Some(Box::new(expr_non_boolean)));
        assert!(matches!(
            res,
            Result::Err(ConversionError::NonbooleanWhereClause(_))
        ));
    }

    #[test]
    fn we_can_not_have_non_boolean_literal_as_where_clause() {
        let column_mapping = IndexMap::new();

        let builder = WhereExprBuilder::new(&column_mapping);

        let expr_non_boolean = Expression::Literal(Literal::Int128(123));
        let res = builder.build::<RistrettoPoint>(Some(Box::new(expr_non_boolean)));
        assert!(matches!(
            res,
            Result::Err(ConversionError::NonbooleanWhereClause(_))
        ));
    }

    #[test]
    fn we_expect_an_error_while_trying_to_check_varchar_column_eq_decimal() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => ["abc"],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_err());
    }

    #[test]
    fn we_expect_an_error_while_trying_to_check_varchar_column_ge_decimal() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => ["abc"],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b >= 123").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_err());
    }

    #[test]
    fn we_do_not_expect_an_error_while_trying_to_check_int128_column_eq_decimal_with_zero_scale() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => [123_i128],
            ),
            0,
        );

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
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => [123_i64],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123.000").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_ok());
    }
}
