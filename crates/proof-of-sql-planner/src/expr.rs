use super::{
    column_to_column_ref, placeholder_to_placeholder_expr, scalar_value_to_literal_value,
    PlannerError, PlannerResult,
};
use datafusion::logical_expr::{
    expr::{Alias, Placeholder},
    BinaryExpr, Expr, Operator,
};
use proof_of_sql::{
    base::database::ColumnType,
    sql::{proof_exprs::DynProofExpr, scale_cast_binary_op},
};
use sqlparser::ast::Ident;

/// Convert a [`BinaryExpr`] to [`DynProofExpr`]
#[expect(
    clippy::missing_panics_doc,
    reason = "Output of comparisons is always boolean"
)]
fn binary_expr_to_proof_expr(
    left: &Expr,
    right: &Expr,
    op: Operator,
    schema: &[(Ident, ColumnType)],
) -> PlannerResult<DynProofExpr> {
    let left_proof_expr = expr_to_proof_expr(left, schema)?;
    let right_proof_expr = expr_to_proof_expr(right, schema)?;

    let (left_proof_expr, right_proof_expr) = match op {
        Operator::Eq
        | Operator::Lt
        | Operator::Gt
        | Operator::LtEq
        | Operator::GtEq
        | Operator::Plus
        | Operator::Minus => scale_cast_binary_op(left_proof_expr, right_proof_expr)?,
        _ => (left_proof_expr, right_proof_expr),
    };

    match op {
        Operator::And => Ok(DynProofExpr::try_new_and(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::Or => Ok(DynProofExpr::try_new_or(left_proof_expr, right_proof_expr)?),
        Operator::Multiply => Ok(DynProofExpr::try_new_multiply(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::Eq => Ok(DynProofExpr::try_new_equals(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::Lt => Ok(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            true,
        )?),
        Operator::Gt => Ok(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            false,
        )?),
        Operator::LtEq => Ok(DynProofExpr::try_new_not(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            false,
        )?)
        .expect("An inequality expression must have a boolean data type...")),
        Operator::GtEq => Ok(DynProofExpr::try_new_not(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            true,
        )?)
        .expect("An inequality expression must have a boolean data type...")),
        Operator::Plus => Ok(DynProofExpr::try_new_add(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::Minus => Ok(DynProofExpr::try_new_subtract(
            left_proof_expr,
            right_proof_expr,
        )?),
        // Any other operator is unsupported
        _ => Err(PlannerError::UnsupportedBinaryOperator { op }),
    }
}

/// Convert an [`datafusion::expr::Expr`] to [`DynProofExpr`]
///
/// # Panics
/// The function should not panic if Proof of SQL is working correctly
pub fn expr_to_proof_expr(
    expr: &Expr,
    schema: &[(Ident, ColumnType)],
) -> PlannerResult<DynProofExpr> {
    match expr {
        Expr::Alias(Alias { expr, .. }) => expr_to_proof_expr(expr, schema),
        Expr::Column(col) => Ok(DynProofExpr::new_column(column_to_column_ref(col, schema)?)),
        Expr::Placeholder(placeholder) => placeholder_to_placeholder_expr(placeholder),
        Expr::BinaryExpr(BinaryExpr { left, right, op }) => {
            binary_expr_to_proof_expr(left, right, *op, schema)
        }
        Expr::Literal(val) => Ok(DynProofExpr::new_literal(scalar_value_to_literal_value(
            val.clone(),
        )?)),
        Expr::Not(expr) => {
            let proof_expr = expr_to_proof_expr(expr, schema)?;
            Ok(DynProofExpr::try_new_not(proof_expr)?)
        }
        Expr::Cast(cast) => {
            match &*cast.expr {
                // handle cases such as `$1::int`
                Expr::Placeholder(placeholder) if placeholder.data_type.is_none() => {
                    let typed_placeholder =
                        Placeholder::new(placeholder.id.clone(), Some(cast.data_type.clone()));
                    placeholder_to_placeholder_expr(&typed_placeholder)
                }
                _ => {
                    let from_expr = expr_to_proof_expr(&cast.expr, schema)?;
                    let to_type = cast.data_type.clone().try_into().map_err(|_| {
                        PlannerError::UnsupportedDataType {
                            data_type: cast.data_type.clone(),
                        }
                    })?;
                    Ok(DynProofExpr::try_new_cast(from_expr, to_type)?)
                }
            }
        }
        _ => Err(PlannerError::UnsupportedLogicalExpression { expr: expr.clone() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::df_util::*;
    use arrow::datatypes::DataType;
    use core::ops::{Add, Mul, Sub};
    use datafusion::{
        common::ScalarValue,
        logical_expr::{
            expr::{Placeholder, Unnest},
            Cast,
        },
    };
    use proof_of_sql::base::{
        database::{ColumnRef, ColumnType, LiteralValue, TableRef},
        math::decimal::Precision,
    };

    #[expect(non_snake_case)]
    fn COLUMN_INT() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column".into(),
            ColumnType::Int,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN1_SMALLINT() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::SmallInt,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN2_BIGINT() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column2".into(),
            ColumnType::BigInt,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN1_BOOLEAN() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::Boolean,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN2_BOOLEAN() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column2".into(),
            ColumnType::Boolean,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN3_DECIMAL_75_5() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column3".into(),
            ColumnType::Decimal75(
                Precision::new(75).expect("Precision is definitely valid"),
                5,
            ),
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN2_DECIMAL_25_5() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column2".into(),
            ColumnType::Decimal75(
                Precision::new(25).expect("Precision is definitely valid"),
                5,
            ),
        ))
    }

    // Alias
    #[test]
    fn we_can_convert_alias_to_proof_expr() {
        // Column
        let expr = df_column("namespace.table_name", "column").alias("alias");
        let schema = vec![("column".into(), ColumnType::Int)];
        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), COLUMN_INT());
    }

    // Column
    #[test]
    fn we_can_convert_column_expr_to_proof_expr() {
        // Column
        let expr = df_column("namespace.table_name", "column");
        let schema = vec![("column".into(), ColumnType::Int)];
        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), COLUMN_INT());
    }

    // BinaryExpr
    #[test]
    fn we_can_convert_comparison_binary_expr_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            ("column2".into(), ColumnType::BigInt),
        ];

        // Eq
        let expr = df_column("namespace.table_name", "column1")
            .eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_equals(COLUMN1_SMALLINT(), COLUMN2_BIGINT()).unwrap()
        );

        // Lt
        let expr = df_column("namespace.table_name", "column1")
            .lt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), true).unwrap()
        );

        // Gt
        let expr = df_column("namespace.table_name", "column1")
            .gt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), false).unwrap()
        );

        // LtEq
        let expr = df_column("namespace.table_name", "column1")
            .lt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), false)
                    .unwrap()
            )
            .unwrap()
        );

        // GtEq
        let expr = df_column("namespace.table_name", "column1")
            .gt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), true)
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[expect(clippy::too_many_lines)]
    #[test]
    fn we_can_convert_comparison_binary_expr_to_proof_expr_with_scale_cast() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            (
                "column2".into(),
                ColumnType::Decimal75(Precision::new(25).unwrap(), 5),
            ),
            (
                "column3".into(),
                ColumnType::Decimal75(Precision::new(75).unwrap(), 5),
            ),
        ];

        // Eq
        let expr = df_column("namespace.table_name", "column1")
            .eq(df_column("namespace.table_name", "column3"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_equals(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN3_DECIMAL_75_5()
            )
            .unwrap()
        );

        // Lt
        let expr = df_column("namespace.table_name", "column1")
            .lt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5(),
                true
            )
            .unwrap()
        );

        // Gt
        let expr = df_column("namespace.table_name", "column1")
            .gt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5(),
                false
            )
            .unwrap()
        );

        // LtEq
        let expr = df_column("namespace.table_name", "column1")
            .lt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(
                    DynProofExpr::try_new_scaling_cast(
                        COLUMN1_SMALLINT(),
                        ColumnType::Decimal75(
                            Precision::new(10).expect("Precision is definitely valid"),
                            5
                        )
                    )
                    .unwrap(),
                    COLUMN2_DECIMAL_25_5(),
                    false
                )
                .unwrap()
            )
            .unwrap()
        );

        // GtEq
        let expr = df_column("namespace.table_name", "column1")
            .gt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(
                    DynProofExpr::try_new_scaling_cast(
                        COLUMN1_SMALLINT(),
                        ColumnType::Decimal75(
                            Precision::new(10).expect("Precision is definitely valid"),
                            5
                        )
                    )
                    .unwrap(),
                    COLUMN2_DECIMAL_25_5(),
                    true
                )
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn we_can_convert_arithmetic_binary_expr_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            ("column2".into(), ColumnType::BigInt),
        ];

        // Plus
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::Plus,
        });
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_add(COLUMN1_SMALLINT(), COLUMN2_BIGINT(),).unwrap()
        );

        // Minus
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::Minus,
        });
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_subtract(COLUMN1_SMALLINT(), COLUMN2_BIGINT(),).unwrap()
        );

        // Multiply
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::Multiply,
        });
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_multiply(COLUMN1_SMALLINT(), COLUMN2_BIGINT(),).unwrap()
        );
    }

    #[test]
    fn we_can_convert_arithmetic_binary_expr_to_proof_expr_with_scale_cast() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            (
                "column2".into(),
                ColumnType::Decimal75(Precision::new(25).unwrap(), 5),
            ),
            (
                "column3".into(),
                ColumnType::Decimal75(Precision::new(75).unwrap(), 5),
            ),
        ];

        // Add
        let expr = df_column("namespace.table_name", "column1")
            .add(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_add(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5()
            )
            .unwrap()
        );

        // Subtract
        let expr = df_column("namespace.table_name", "column1")
            .sub(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_subtract(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5()
            )
            .unwrap()
        );

        // Multiply - No scale cast!
        let expr = df_column("namespace.table_name", "column1")
            .mul(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_multiply(COLUMN1_SMALLINT(), COLUMN2_DECIMAL_25_5()).unwrap()
        );
    }

    #[test]
    fn we_can_convert_logical_binary_expr_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::Boolean),
            ("column2".into(), ColumnType::Boolean),
        ];

        // And
        let expr = df_column("namespace.table_name", "column1")
            .and(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_and(COLUMN1_BOOLEAN(), COLUMN2_BOOLEAN()).unwrap()
        );

        // Or
        let expr = df_column("namespace.table_name", "column1")
            .or(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_or(COLUMN1_BOOLEAN(), COLUMN2_BOOLEAN()).unwrap()
        );
    }

    #[test]
    fn we_cannot_convert_unsupported_binary_expr_to_proof_expr() {
        // Unsupported binary operator
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::AtArrow,
        });
        let schema = vec![
            ("column1".into(), ColumnType::Boolean),
            ("column2".into(), ColumnType::Boolean),
        ];
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema),
            Err(PlannerError::UnsupportedBinaryOperator { .. })
        ));
    }

    // Literal
    #[test]
    fn we_can_convert_literal_expr_to_proof_expr() {
        let expr = Expr::Literal(ScalarValue::Int32(Some(1)));
        assert_eq!(
            expr_to_proof_expr(&expr, &Vec::new()).unwrap(),
            DynProofExpr::new_literal(LiteralValue::Int(1))
        );
    }

    // Not
    #[test]
    fn we_can_convert_not_expr_to_proof_expr() {
        let expr = Expr::Not(Box::new(df_column("table_name", "column")));
        let schema = vec![("column".into(), ColumnType::Boolean)];
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(DynProofExpr::new_column(ColumnRef::new(
                TableRef::from_names(None, "table_name"),
                "column".into(),
                ColumnType::Boolean
            )))
            .unwrap()
        );
    }

    // Cast
    #[test]
    fn we_can_convert_cast_expr_to_proof_expr() {
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::Boolean(Some(true)))),
            DataType::Int32,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap();
        assert_eq!(
            expression,
            DynProofExpr::try_new_cast(
                DynProofExpr::new_literal(LiteralValue::Boolean(true)),
                ColumnType::Int
            )
            .unwrap()
        );
    }

    #[test]
    fn we_cannot_convert_cast_expr_to_proof_expr_when_inner_expr_to_proof_expr_fails() {
        // Unsupported logical expression
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::UInt64(Some(100)))),
            DataType::Int16,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap_err();
        assert!(matches!(
            expression,
            PlannerError::UnsupportedDataType { data_type: _ }
        ));
    }

    #[test]
    fn we_cannot_convert_cast_expr_to_proof_expr_for_unsupported_datatypes() {
        // Unsupported logical expression
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::Boolean(Some(true)))),
            DataType::UInt16,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap_err();
        assert!(matches!(
            expression,
            PlannerError::UnsupportedDataType { data_type: _ }
        ));
    }

    #[test]
    fn we_cannot_convert_cast_expr_to_proof_expr_for_datatypes_for_which_casting_is_not_supported()
    {
        // Unsupported logical expression
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::Int16(Some(100)))),
            DataType::Boolean,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap_err();
        assert!(matches!(
            expression,
            PlannerError::AnalyzeError { source: _ }
        ));
    }

    // Placeholder
    #[test]
    fn we_can_convert_placeholder_to_proof_expr() {
        let expr = Expr::Placeholder(Placeholder {
            id: "$1".to_string(),
            data_type: Some(DataType::Int32),
        });
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap();
        assert_eq!(
            expression,
            DynProofExpr::try_new_placeholder(1, ColumnType::Int).unwrap()
        );
    }

    // Placeholder with data type specified by cast
    #[test]
    fn we_can_convert_placeholder_with_data_type_specified_by_cast_to_proof_expr() {
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Placeholder(Placeholder {
                id: "$1".to_string(),
                data_type: None,
            })),
            DataType::Int32,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap();
        assert_eq!(
            expression,
            DynProofExpr::try_new_placeholder(1, ColumnType::Int).unwrap()
        );
    }

    // Unsupported logical expression
    #[test]
    fn we_cannot_convert_unsupported_expr_to_proof_expr() {
        let expr = Expr::Unnest(Unnest::new(Expr::Literal(ScalarValue::Int32(Some(100)))));
        assert!(matches!(
            expr_to_proof_expr(&expr, &Vec::new()),
            Err(PlannerError::UnsupportedLogicalExpression { .. })
        ));
    }

    #[test]
    fn we_can_get_proof_expr_for_timestamps_of_different_scale() {
        let lhs = Expr::Literal(ScalarValue::TimestampSecond(Some(1), None));
        let rhs = Expr::Literal(ScalarValue::TimestampNanosecond(Some(1), None));
        binary_expr_to_proof_expr(&lhs, &rhs, Operator::Gt, &Vec::new()).unwrap();
    }
}
