#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnRef, ColumnType, OwnedTable, OwnedTableTestAccessor, TestAccessor},
            math::decimal::Precision,
            scalar::{ArkScalar as S, Scalar},
        },
        record_batch,
        sql::parse::{query_expr_tests::record_batch_to_accessor, QueryExpr, WhereExprBuilder},
    };
    use blitzar::proof::InnerProductProof;
    use curve25519_dalek::RistrettoPoint;
    use proofs_sql::{
        decimal_unknown::DecimalUnknown,
        intermediate_ast::{BinaryOperator, Expression, Literal},
        Identifier, SelectStatement,
    };
    use std::{collections::HashMap, str::FromStr};

    fn run_test_case(column_mapping: &HashMap<Identifier, ColumnRef>, expr: Expression) {
        let builder = WhereExprBuilder::new(column_mapping);
        let result = builder.build::<RistrettoPoint>(Some(Box::new(expr)));
        assert!(result.is_ok(), "Test case should succeed without panic.");
    }

    fn get_column_mappings_for_testing() -> HashMap<Identifier, ColumnRef> {
        let mut column_mapping = HashMap::new();
        // Setup column mapping
        column_mapping.insert(
            Identifier::try_new("decimal_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("decimal_column").unwrap(),
                ColumnType::Decimal75(Precision::new(6).unwrap(), 2),
            ),
        );
        column_mapping.insert(
            Identifier::try_new("int_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("int_column").unwrap(),
                ColumnType::Int128,
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
        column_mapping
    }

    #[test]
    fn we_cannot_round_decimals_down_to_match() {
        let mut column_mapping = HashMap::new();
        column_mapping.insert(
            Identifier::try_new("test_column").unwrap(),
            ColumnRef::new(
                "sxt.sxt_tab".parse().unwrap(),
                Identifier::try_new("c").unwrap(),
                ColumnType::Decimal75(Precision::new(6).unwrap(), 1),
            ),
        );

        let builder = WhereExprBuilder::new(&column_mapping);
        let left_expr = Expression::Column(Identifier::try_new("test_column").unwrap());
        let right_expr = Expression::Literal(Literal::Decimal(DecimalUnknown::new("123.456")));

        let expr = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        };

        // Error because we cannot round a decimal down
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr)))
            .is_err());
    }

    #[test]
    fn we_can_directly_compare_integer_to_integer_columns() {
        let column_mapping = get_column_mappings_for_testing();
        let builder = WhereExprBuilder::new(&column_mapping);
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("int_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        assert!(builder
            .build::<RistrettoPoint>(Some(Box::new(expr_integer_to_integer)))
            .is_ok());
    }

    #[test]
    fn we_can_match_varchar_to_varchar() {
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
    fn we_can_match_non_decimal_columns_to_integer_literals() {
        let column_mapping = get_column_mappings_for_testing();

        // Non-decimal column with integer literal
        let expr_integer_to_integer = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("int_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Int128(12345))),
        };
        run_test_case(&column_mapping, expr_integer_to_integer);
    }

    #[test]
    fn we_can_match_to_scaled_integers_correctly() {
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
    fn we_can_match_to_exact_scale_and_precision() {
        let column_mapping = get_column_mappings_for_testing();

        // Decimal column with matching scale decimal literal
        let expr_decimal = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Column(
                Identifier::try_new("decimal_column").unwrap(),
            )),
            right: Box::new(Expression::Literal(Literal::Decimal(DecimalUnknown::new(
                "123.45",
            )))),
        };
        run_test_case(&column_mapping, expr_decimal);
    }

    #[test]
    #[should_panic(expected = "The parser must ensure that the expression is a boolean expression")]
    fn unexpected_expression_panics() {
        let column_mapping = HashMap::new();

        let builder = WhereExprBuilder::new(&column_mapping);
        // Creating an unexpected expression type
        let expr_unexpected = Expression::Literal(Literal::Int128(123));
        builder
            .build::<RistrettoPoint>(Some(Box::new(expr_unexpected)))
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "The parser must ensure that the left side is a column")]
    fn left_side_not_column_panics() {
        let column_mapping = HashMap::new();

        let builder = WhereExprBuilder::new(&column_mapping);
        // Intentionally setting the left expression to a non-column type to trigger a panic
        let left_expr = Expression::Literal(Literal::Int128(123));
        let right_expr = Expression::Literal(Literal::Int128(456));

        let expr = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        };

        // This should trigger a panic due to the left side not being a column
        let _ = builder.build::<RistrettoPoint>(Some(Box::new(expr)));
    }

    #[test]
    fn we_expect_an_error_while_trying_to_match_decimal_to_varchar_column() {
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
    fn we_expect_an_error_while_trying_to_match_decimal_to_int128_column() {
        let t = "sxt.sxt_tab".parse().unwrap();
        let accessor = record_batch_to_accessor(
            t,
            record_batch!(
                "b" => [123_i128],
            ),
            0,
        );

        assert!(QueryExpr::<RistrettoPoint>::try_new(
            SelectStatement::from_str("select * from sxt_tab where b = 123.456").unwrap(),
            t.schema_id(),
            &accessor,
        )
        .is_err());
    }

    #[test]
    fn we_do_not_expect_an_error_while_trying_to_match_decimal_with_zero_scale_to_int_column() {
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

    #[cfg(test)]
    pub fn run_query(
        query_str: &str,
        expected_precision: u8,
        expected_scale: i8,
        test_decimal_values: Vec<S>,
        expected_decimal_values: Vec<S>,
    ) {
        // Setup common data and accessor for each run

        use crate::{owned_table, sql::proof::QueryProof};

        let mut accessor = OwnedTableTestAccessor::new_empty();
        let mut data: OwnedTable<S> = owned_table!("a" => [1i64, 2, 3], "b" => ["t", "u", "v"]);
        data.append_decimal_columns_for_testing(
            "c",
            expected_precision,
            expected_scale,
            test_decimal_values,
        );

        accessor.add_table("sxt.table".parse().unwrap(), data, 0);

        let query = QueryExpr::try_new(
            query_str.parse().unwrap(),
            "sxt".parse().unwrap(),
            &accessor,
        )
        .unwrap();
        let (proof, serialized_result) =
            QueryProof::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
        let owned_table_result = proof
            .verify(query.proof_expr(), &accessor, &serialized_result, &())
            .unwrap()
            .table;

        // Adjust expected result based on the precision and scale provided
        let mut expected_result = owned_table!("a" => [1_i64, 3], "b" => ["t", "v"]);
        expected_result.append_decimal_columns_for_testing(
            "c",
            expected_precision,
            expected_scale,
            expected_decimal_values,
        );
        // Verify the result matches the expectation
        assert_eq!(owned_table_result, expected_result);
    }

    #[test]
    fn we_can_query_integers_against_decimals() {
        run_query(
            "SELECT * FROM table WHERE c = 12345",
            7,
            2,
            vec![S::from(1234500), S::ZERO, S::from(1234500)],
            vec![S::from(1234500), S::from(1234500)],
        );
    }

    #[test]
    fn we_can_query_negative_integers_against_decimals() {
        run_query(
            "SELECT * FROM table WHERE c = -12345",
            7,
            2,
            vec![-S::from(1234500), S::ZERO, -S::from(1234500)],
            vec![-S::from(1234500), -S::from(1234500)],
        );
    }
}
