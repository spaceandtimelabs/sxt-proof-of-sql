use super::{
    column_to_column_ref, placeholder_to_placeholder_expr, scalar_value_to_literal_value,
    PlannerError, PlannerResult,
};
use core::cmp::Ordering;
use datafusion::{
    common::DFSchema,
    logical_expr::{
        expr::{Alias, Placeholder},
        BinaryExpr, Expr, Operator,
    },
};
use proof_of_sql::{
    base::{database::ColumnType, math::decimal::Precision},
    sql::{
        proof_exprs::{DynProofExpr, ProofExpr},
        AnalyzeError,
    },
};

/// Add a layer of decimal scaling cast to the expression
/// so that we can do binary operations on it
#[expect(clippy::missing_panics_doc, reason = "Precision can not be invalid")]
fn decimal_scale_cast_expr(
    from_proof_expr: DynProofExpr,
    from_scale: i8,
    to_scale: i8,
) -> PlannerResult<DynProofExpr> {
    if !from_proof_expr.data_type().is_numeric() {
        return Err(PlannerError::AnalyzeError {
            source: AnalyzeError::DataTypeMismatch {
                left_type: from_proof_expr.data_type().to_string(),
                right_type: "Some numeric type".to_string(),
            },
        });
    }
    let from_precision_value = from_proof_expr.data_type().precision_value().unwrap_or(0);
    let to_precision_value = u8::try_from(
        i16::from(from_precision_value) + i16::from(to_scale - from_scale).min(75_i16),
    )
    .expect("Precision is definitely valid");
    Ok(DynProofExpr::try_new_decimal_scaling_cast(
        from_proof_expr,
        ColumnType::Decimal75(
            Precision::new(to_precision_value).expect("Precision is definitely valid"),
            to_scale,
        ),
    )?)
}

/// Scale cast one side so that both sides have the same scale
///
/// We use this function so that binary ops for numeric types no longer
/// need to keep track of scale
fn scale_cast_binary_op(
    left_proof_expr: DynProofExpr,
    right_proof_expr: DynProofExpr,
) -> PlannerResult<(DynProofExpr, DynProofExpr)> {
    let left_type = left_proof_expr.data_type();
    let right_type = right_proof_expr.data_type();
    let left_scale = left_type.scale().unwrap_or(0);
    let right_scale = right_type.scale().unwrap_or(0);
    let scale = left_scale.max(right_scale);
    match left_scale.cmp(&right_scale) {
        Ordering::Less => Ok((
            decimal_scale_cast_expr(left_proof_expr, left_scale, scale)?,
            right_proof_expr,
        )),
        Ordering::Greater => Ok((
            left_proof_expr,
            decimal_scale_cast_expr(right_proof_expr, right_scale, scale)?,
        )),
        Ordering::Equal => Ok((left_proof_expr, right_proof_expr)),
    }
}

/// Convert a [`BinaryExpr`] to [`DynProofExpr`]
#[expect(
    clippy::missing_panics_doc,
    reason = "Output of comparisons is always boolean"
)]
fn binary_expr_to_proof_expr(
    left: &Expr,
    right: &Expr,
    op: Operator,
    schema: &DFSchema,
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
pub fn expr_to_proof_expr(expr: &Expr, schema: &DFSchema) -> PlannerResult<DynProofExpr> {
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

    // decimal_scale_cast_expr
    #[test]
    fn we_can_convert_decimal_scale_cast_expr() {
        let expr = COLUMN1_SMALLINT();
        let scale = 0;
        let to_scale = 5;
        let proof_expr = decimal_scale_cast_expr(expr, scale, to_scale).unwrap();
        assert_eq!(
            proof_expr,
            DynProofExpr::try_new_decimal_scaling_cast(
                COLUMN1_SMALLINT(),
                ColumnType::Decimal75(
                    Precision::new(10).expect("Precision is definitely valid"),
                    5
                )
            )
            .unwrap()
        );
    }

    #[test]
    fn we_cannot_convert_nonnumeric_types_using_decimal_scale_cast_expr() {
        let expr = COLUMN1_BOOLEAN();
        let scale = 0;
        let to_scale = 5;
        let proof_expr = decimal_scale_cast_expr(expr, scale, to_scale);
        assert!(matches!(
            proof_expr,
            Err(PlannerError::AnalyzeError {
                source: AnalyzeError::DataTypeMismatch { .. }
            })
        ));
    }

    // Alias
    #[test]
    fn we_can_convert_alias_to_proof_expr() {
        // Column
        let expr = df_column("namespace.table_name", "column").alias("alias");
        let schema = df_schema("namespace.table_name", vec![("column", DataType::Int32)]);
        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), COLUMN_INT());
    }

    // Column
    #[test]
    fn we_can_convert_column_expr_to_proof_expr() {
        // Column
        let expr = df_column("namespace.table_name", "column");
        let schema = df_schema("namespace.table_name", vec![("column", DataType::Int32)]);
        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), COLUMN_INT());
    }

    // BinaryExpr
    #[test]
    fn we_can_convert_comparison_binary_expr_to_proof_expr() {
        let schema = df_schema(
            "namespace.table_name",
            vec![("column1", DataType::Int16), ("column2", DataType::Int64)],
        );

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
        let schema = df_schema(
            "namespace.table_name",
            vec![
                ("column1", DataType::Int16),
                ("column2", DataType::Decimal256(25, 5)),
                ("column3", DataType::Decimal256(75, 5)),
            ],
        );

        // Eq
        let expr = df_column("namespace.table_name", "column1")
            .eq(df_column("namespace.table_name", "column3"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_equals(
                DynProofExpr::try_new_decimal_scaling_cast(
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
                DynProofExpr::try_new_decimal_scaling_cast(
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
                DynProofExpr::try_new_decimal_scaling_cast(
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
                    DynProofExpr::try_new_decimal_scaling_cast(
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
                    DynProofExpr::try_new_decimal_scaling_cast(
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
        let schema = df_schema(
            "namespace.table_name",
            vec![("column1", DataType::Int16), ("column2", DataType::Int64)],
        );

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
        let schema = df_schema(
            "namespace.table_name",
            vec![
                ("column1", DataType::Int16),
                ("column2", DataType::Decimal256(25, 5)),
                ("column3", DataType::Decimal256(75, 5)),
            ],
        );

        // Add
        let expr = df_column("namespace.table_name", "column1")
            .add(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_add(
                DynProofExpr::try_new_decimal_scaling_cast(
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
                DynProofExpr::try_new_decimal_scaling_cast(
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
        let schema = df_schema(
            "namespace.table_name",
            vec![
                ("column1", DataType::Boolean),
                ("column2", DataType::Boolean),
            ],
        );

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
        let schema = df_schema(
            "namespace.table_name",
            vec![
                ("column1", DataType::Boolean),
                ("column2", DataType::Boolean),
            ],
        );
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema),
            Err(PlannerError::UnsupportedBinaryOperator { .. })
        ));
    }

    // Literal
    #[test]
    fn we_can_convert_literal_expr_to_proof_expr() {
        let expr = Expr::Literal(ScalarValue::Int32(Some(1)));
        let schema = df_schema("namespace.table_name", vec![]);
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::new_literal(LiteralValue::Int(1))
        );
    }

    // Not
    #[test]
    fn we_can_convert_not_expr_to_proof_expr() {
        let expr = Expr::Not(Box::new(df_column("table_name", "column")));
        let schema = df_schema("table_name", vec![("column", DataType::Boolean)]);
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
        let schema = df_schema("namespace.table_name", vec![]);
        let expression = expr_to_proof_expr(&expr, &schema).unwrap();
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
        let schema = df_schema("namespace.table_name", vec![]);
        let expression = expr_to_proof_expr(&expr, &schema).unwrap_err();
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
        let schema = df_schema("namespace.table_name", vec![]);
        let expression = expr_to_proof_expr(&expr, &schema).unwrap_err();
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
        let schema = df_schema("namespace.table_name", vec![]);
        let expression = expr_to_proof_expr(&expr, &schema).unwrap_err();
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
        let schema = df_schema("namespace.table_name", vec![]);
        let expression = expr_to_proof_expr(&expr, &schema).unwrap();
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
        let schema = df_schema("namespace.table_name", vec![]);
        let expression = expr_to_proof_expr(&expr, &schema).unwrap();
        assert_eq!(
            expression,
            DynProofExpr::try_new_placeholder(1, ColumnType::Int).unwrap()
        );
    }

    // Unsupported logical expression
    #[test]
    fn we_cannot_convert_unsupported_expr_to_proof_expr() {
        let expr = Expr::Unnest(Unnest::new(Expr::Literal(ScalarValue::Int32(Some(100)))));
        let schema = df_schema("namespace.table_name", vec![]);
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema),
            Err(PlannerError::UnsupportedLogicalExpression { .. })
        ));
    }
}
