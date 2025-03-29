use super::{column_to_column_ref, scalar_value_to_literal_value, PlannerError, PlannerResult};
use datafusion::{
    common::DFSchema,
    logical_expr::{expr::Alias, BinaryExpr, Expr, Operator},
};
use proof_of_sql::sql::proof_exprs::DynProofExpr;

/// Convert an [`datafusion::expr::Expr`] to [`DynProofExpr`]
///
/// # Panics
/// The function should not panic if Proof of SQL is working correctly
pub fn expr_to_proof_expr(expr: &Expr, schema: &DFSchema) -> PlannerResult<DynProofExpr> {
    match expr {
        Expr::Alias(Alias { expr, .. }) => expr_to_proof_expr(expr, schema),
        Expr::Column(col) => Ok(DynProofExpr::new_column(column_to_column_ref(col, schema)?)),
        Expr::BinaryExpr(BinaryExpr { left, right, op }) => {
            let left_proof_expr = expr_to_proof_expr(left, schema)?;
            let right_proof_expr = expr_to_proof_expr(right, schema)?;
            match op {
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
                Operator::LtEq => Ok(DynProofExpr::try_new_not(
                    DynProofExpr::try_new_inequality(left_proof_expr, right_proof_expr, false)?,
                ).expect("an inequality expression must have a boolean data type, and try_new_not only fails when the datatype is non-boolean.")),
                Operator::GtEq => Ok(DynProofExpr::try_new_not(
                    DynProofExpr::try_new_inequality(left_proof_expr, right_proof_expr, true)?,
                ).expect("an inequality expression must have a boolean data type, and try_new_not only fails when the datatype is non-boolean.")),
                Operator::Plus => Ok(DynProofExpr::try_new_add(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                Operator::Minus => Ok(DynProofExpr::try_new_subtract(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                Operator::Multiply => Ok(DynProofExpr::try_new_multiply(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                Operator::And => Ok(DynProofExpr::try_new_and(
                    left_proof_expr,
                    right_proof_expr,
                )?),
                Operator::Or => Ok(DynProofExpr::try_new_or(left_proof_expr, right_proof_expr)?),
                _ => Err(PlannerError::UnsupportedBinaryOperator { op: *op }),
            }
        }
        Expr::Literal(val) => Ok(DynProofExpr::new_literal(scalar_value_to_literal_value(
            val.clone(),
        )?)),
        Expr::Not(expr) => {
            let proof_expr = expr_to_proof_expr(expr, schema)?;
            Ok(DynProofExpr::try_new_not(proof_expr)?)
        }
        Expr::Cast(cast) => {
            let from_expr = expr_to_proof_expr(&cast.expr, schema)?;
            let to_type = cast.data_type.clone().try_into().map_err(|_| {
                PlannerError::UnsupportedDataType {
                    data_type: cast.data_type.clone(),
                }
            })?;
            Ok(DynProofExpr::try_new_cast(from_expr, to_type)?)
        }
        _ => Err(PlannerError::UnsupportedLogicalExpression { expr: expr.clone() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::df_util::*;
    use arrow::datatypes::DataType;
    use datafusion::{
        common::ScalarValue,
        logical_expr::{expr::Placeholder, Cast},
    };
    use proof_of_sql::base::database::{ColumnRef, ColumnType, LiteralValue, TableRef};

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

    #[test]
    fn we_cannot_convert_unsupported_logical_expr_to_proof_expr() {
        // Unsupported logical expression
        let expr = Expr::Placeholder(Placeholder {
            id: "$1".to_string(),
            data_type: None,
        });
        let schema = df_schema("namespace.table_name", vec![]);
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema),
            Err(PlannerError::UnsupportedLogicalExpression { .. })
        ));
    }

    #[test]
    fn we_can_convert_cast_expr_to_proof_expr() {
        // Unsupported logical expression
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
}
