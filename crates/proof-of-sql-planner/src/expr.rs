use super::{
    column_to_column_ref, placeholder_to_placeholder_expr, scalar_value_to_literal_value,
    PlannerError, PlannerResult,
};
use datafusion::logical_expr::{
    expr::{Alias, Between, Cast, InList, Placeholder},
    BinaryExpr, Expr, Operator,
};
use indexmap::IndexSet;
use proof_of_sql::{
    base::database::{ColumnType, LiteralValue},
    sql::{
        proof_exprs::{DynProofExpr, ProofExpr},
        scale_cast_binary_op,
    },
};
use sqlparser::ast::Ident;

/// Recursively extract all column identifiers referenced in an expression
pub(crate) fn get_column_idents_from_expr(expr: &Expr) -> IndexSet<Ident> {
    match expr {
        Expr::Column(col) => {
            let mut set = IndexSet::new();
            set.insert(col.name.as_str().into());
            set
        }
        Expr::BinaryExpr(BinaryExpr { left, right, .. }) => {
            let mut left_idents = get_column_idents_from_expr(left);
            left_idents.extend(get_column_idents_from_expr(right));
            left_idents
        }
        Expr::Not(inner) => get_column_idents_from_expr(inner),
        Expr::InList(InList { expr, list, .. }) => {
            let mut idents = get_column_idents_from_expr(expr);
            for value in list {
                idents.extend(get_column_idents_from_expr(value));
            }
            idents
        }
        Expr::Alias(Alias { expr, .. }) | Expr::Cast(Cast { expr, .. }) => {
            get_column_idents_from_expr(expr)
        }
        Expr::AggregateFunction(agg) => agg
            .args
            .iter()
            .flat_map(get_column_idents_from_expr)
            .collect(),
        Expr::Between(Between {
            expr, low, high, ..
        }) => {
            let mut idents = get_column_idents_from_expr(expr);
            idents.extend(get_column_idents_from_expr(low));
            idents.extend(get_column_idents_from_expr(high));
            idents
        }
        _ => IndexSet::new(),
    }
}

/// Convert a [`BinaryExpr`] to [`DynProofExpr`]
fn binary_expr_to_proof_expr(
    left: &Expr,
    right: &Expr,
    op: Operator,
    schema: &[(Ident, ColumnType)],
) -> PlannerResult<DynProofExpr> {
    let left_proof_expr = expr_to_proof_expr(left, schema)?;
    let right_proof_expr = expr_to_proof_expr(right, schema)?;
    binary_proof_exprs_to_proof_expr(left_proof_expr, right_proof_expr, op)
}

/// Apply a binary [`Operator`] to two already-converted [`DynProofExpr`]s.
///
/// Scale-casting is performed here for operators that require it, so callers
/// that already hold `DynProofExpr`s can skip the `Expr` conversion step.
#[expect(
    clippy::missing_panics_doc,
    reason = "Output of comparisons is always boolean"
)]
fn binary_proof_exprs_to_proof_expr(
    left_proof_expr: DynProofExpr,
    right_proof_expr: DynProofExpr,
    op: Operator,
) -> PlannerResult<DynProofExpr> {
    let (left_proof_expr, right_proof_expr) = match op {
        Operator::Eq
        | Operator::NotEq
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
        Operator::NotEq => Ok(DynProofExpr::try_new_not(DynProofExpr::try_new_equals(
            left_proof_expr,
            right_proof_expr,
        )?)
        .expect("An equality expression must have a boolean data type...")),
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
        _ => Err(PlannerError::UnsupportedBinaryOperator { op }),
    }
}

/// Convert a `DataFusion` [`Expr`] into a provable [`DynProofExpr`], using `schema`
/// to resolve column references and their types.
///
/// # Errors
/// Returns a [`PlannerError`] if the expression (or any subexpression) is unsupported,
/// references an unknown column, or cannot be lowered to a `DynProofExpr`.
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
        Expr::InList(InList {
            expr,
            list,
            negated,
        }) => {
            // Lower `a IN (v_1, ..., v_k)`. The list length is uncapped in both branches.
            //
            //   - numeric `a`: use the product form `(a - v_1) * ... * (a - v_k) = 0`.
            //     A field is an integral domain, so the product is zero iff some factor is.
            //   - non-numeric `a` (e.g. varchar): the typed `-` operator rejects these, so
            //     fall back to `a = v_1 OR ... OR a = v_k`. Equality subtracts the embedded
            //     hashes internally, which is exactly what a membership zero-test needs.
            let needle = expr_to_proof_expr(expr, schema)?;
            // Split off the first value. `None` means `a IN ()`, which is always false;
            // keeping that here (not an early return) lets the `negated` wrap below
            // still apply, so `a NOT IN ()` correctly negates to `true`.
            let comparison = match list.split_first() {
                None => DynProofExpr::new_literal(LiteralValue::Boolean(false)),
                Some((first, rest)) => {
                    if needle.data_type().is_numeric() {
                        // product form: (a - v_1) * ... * (a - v_k) = 0. `scale_cast_binary_op`
                        // aligns the needle to each `v_i`'s scale before subtracting; this is a
                        // numeric-only concern, so it stays out of the varchar branch.
                        let term = |value: &Expr| -> PlannerResult<_> {
                            let (n, v) = scale_cast_binary_op(
                                needle.clone(),
                                expr_to_proof_expr(value, schema)?,
                            )?;
                            Ok(DynProofExpr::try_new_subtract(n, v)?)
                        };
                        let product = rest.iter().try_fold(
                            term(first)?,
                            |acc, value| -> PlannerResult<_> {
                                Ok(DynProofExpr::try_new_multiply(acc, term(value)?)?)
                            },
                        )?;
                        let zero = DynProofExpr::new_literal(LiteralValue::BigInt(0));
                        let (product, zero) = scale_cast_binary_op(product, zero)?;
                        DynProofExpr::try_new_equals(product, zero)?
                    } else {
                        // a = v_1 OR ... OR a = v_k. The needle is non-numeric, so there is no
                        // scale to align; `=` compares the lowered operands directly.
                        let eq = |value: &Expr| -> PlannerResult<_> {
                            Ok(DynProofExpr::try_new_equals(
                                needle.clone(),
                                expr_to_proof_expr(value, schema)?,
                            )?)
                        };
                        rest.iter()
                            .try_fold(eq(first)?, |acc, value| -> PlannerResult<_> {
                                Ok(DynProofExpr::try_new_or(acc, eq(value)?)?)
                            })?
                    }
                }
            };
            if *negated {
                Ok(DynProofExpr::try_new_not(comparison)?)
            } else {
                Ok(comparison)
            }
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
                    Ok(
                        DynProofExpr::try_new_cast(from_expr.clone(), to_type).map_or_else(
                            |_| DynProofExpr::try_new_scaling_cast(from_expr, to_type),
                            Ok,
                        )?,
                    )
                }
            }
        }
        Expr::Between(Between {
            expr,
            negated,
            low,
            high,
        }) => between_to_proof_expr(expr, *negated, low, high, schema),
        _ => Err(PlannerError::UnsupportedLogicalExpression {
            expr: Box::new(expr.clone()),
        }),
    }
}

/// Convert a [`Between`] expression to [`DynProofExpr`]
fn between_to_proof_expr(
    expr: &Expr,
    negated: bool,
    low: &Expr,
    high: &Expr,
    schema: &[(Ident, ColumnType)],
) -> PlannerResult<DynProofExpr> {
    let expr_proof = expr_to_proof_expr(expr, schema)?;
    let low_proof = expr_to_proof_expr(low, schema)?;
    let high_proof = expr_to_proof_expr(high, schema)?;
    // expr < low OR expr > high
    let out_of_range = binary_proof_exprs_to_proof_expr(
        binary_proof_exprs_to_proof_expr(expr_proof.clone(), low_proof, Operator::Lt)?,
        binary_proof_exprs_to_proof_expr(expr_proof, high_proof, Operator::Gt)?,
        Operator::Or,
    )?;
    if negated {
        Ok(out_of_range)
    } else {
        Ok(DynProofExpr::try_new_not(out_of_range)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::df_util::*;
    use arrow::datatypes::DataType;
    use core::ops::{Add, Mul, Sub};
    use datafusion::{
        catalog::TableReference,
        common::{Column, ScalarValue},
        logical_expr::{expr::Placeholder, lit, Cast},
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

    // IN
    #[test]
    fn we_can_convert_in_list_to_proof_expr() {
        // `a IN (...)` lowers to `(product of differences) = 0`, i.e. a top-level equals.
        let expr = df_column("namespace.table_name", "column")
            .in_list(vec![lit(1_i64), lit(2_i64), lit(3_i64)], false);
        let schema = vec![("column".into(), ColumnType::BigInt)];
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::Equals(_)
        ));
    }

    #[test]
    fn we_can_convert_not_in_list_to_proof_expr() {
        // `NOT IN` wraps the lowered `IN` expression in a `NOT`.
        let expr = df_column("namespace.table_name", "column").in_list(vec![lit(1_i64)], true);
        let schema = vec![("column".into(), ColumnType::BigInt)];
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::Not(_)
        ));
    }

    #[test]
    fn we_convert_an_empty_in_list_to_a_false_literal() {
        // `a IN ()` is always false.
        let expr = df_column("namespace.table_name", "column").in_list(vec![], false);
        let schema = vec![("column".into(), ColumnType::BigInt)];
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::new_literal(LiteralValue::Boolean(false))
        );
    }

    #[test]
    fn we_convert_an_empty_not_in_list_to_a_true_literal() {
        // `a NOT IN ()` is always true: the empty-list `false` must still flow
        // through the `negated` NOT wrap rather than short-circuiting.
        let expr = df_column("namespace.table_name", "column").in_list(vec![], true);
        let schema = vec![("column".into(), ColumnType::BigInt)];
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(DynProofExpr::new_literal(LiteralValue::Boolean(false)))
                .unwrap()
        );
    }

    #[test]
    fn we_convert_a_varchar_in_list_to_an_or_chain() {
        // `-` rejects varchar, so non-numeric `IN` falls back to `a = v_1 OR a = v_2`.
        let expr =
            df_column("namespace.table_name", "column").in_list(vec![lit("a"), lit("b")], false);
        let schema = vec![("column".into(), ColumnType::VarChar)];
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::Or(_)
        ));
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
    fn we_can_convert_logical_not_eq_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::BigInt),
            ("column2".into(), ColumnType::BigInt),
        ];

        let expr = df_column("namespace.table_name", "column1")
            .not_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_equals(
                    DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(Some("namespace"), "table_name"),
                        "column1".into(),
                        ColumnType::BigInt,
                    )),
                    DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(Some("namespace"), "table_name"),
                        "column2".into(),
                        ColumnType::BigInt,
                    ))
                )
                .unwrap()
            )
            .unwrap()
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
        let expr = Expr::OuterReferenceColumn(
            DataType::Int32,
            Column::new(None::<TableReference>, "column"),
        );
        assert!(matches!(
            expr_to_proof_expr(&expr, &Vec::new()),
            Err(PlannerError::UnsupportedLogicalExpression { .. })
        ));
    }

    // Between
    #[test]
    fn we_can_convert_between_expr_to_proof_expr() {
        let schema = vec![("column1".into(), ColumnType::BigInt)];

        let col = df_column("namespace.table_name", "column1");
        let low = Expr::Literal(ScalarValue::Int64(Some(10)));
        let high = Expr::Literal(ScalarValue::Int64(Some(20)));
        let expr = col.between(low, high);

        let col_expr = DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::BigInt,
        ));
        let low_expr = DynProofExpr::new_literal(LiteralValue::BigInt(10));
        let high_expr = DynProofExpr::new_literal(LiteralValue::BigInt(20));

        let expected = DynProofExpr::try_new_not(
            DynProofExpr::try_new_or(
                DynProofExpr::try_new_inequality(col_expr.clone(), low_expr, true).unwrap(),
                DynProofExpr::try_new_inequality(col_expr, high_expr, false).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), expected);
    }

    #[test]
    fn we_can_convert_not_between_expr_to_proof_expr() {
        let schema = vec![("column1".into(), ColumnType::BigInt)];

        let col = df_column("namespace.table_name", "column1");
        let low = Expr::Literal(ScalarValue::Int64(Some(10)));
        let high = Expr::Literal(ScalarValue::Int64(Some(20)));
        let expr = col.not_between(low, high);

        let col_expr = DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::BigInt,
        ));
        let low_expr = DynProofExpr::new_literal(LiteralValue::BigInt(10));
        let high_expr = DynProofExpr::new_literal(LiteralValue::BigInt(20));

        let expected = DynProofExpr::try_new_or(
            DynProofExpr::try_new_inequality(col_expr.clone(), low_expr, true).unwrap(),
            DynProofExpr::try_new_inequality(col_expr, high_expr, false).unwrap(),
        )
        .unwrap();

        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_between_expr() {
        let col = df_column("table", "val");
        let low = Expr::Literal(ScalarValue::Int64(Some(1)));
        let high = Expr::Literal(ScalarValue::Int64(Some(100)));
        let expr = col.between(low, high);
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["val".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_get_proof_expr_for_timestamps_of_different_scale() {
        let lhs = Expr::Literal(ScalarValue::TimestampSecond(Some(1), None));
        let rhs = Expr::Literal(ScalarValue::TimestampNanosecond(Some(1), None));
        binary_expr_to_proof_expr(&lhs, &rhs, Operator::Gt, &Vec::new()).unwrap();
    }

    // get_column_idents_from_expr tests
    #[test]
    fn we_can_extract_single_column_ident() {
        let expr = df_column("table", "column_a");
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["column_a".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_binary_expr() {
        let expr = df_column("table", "a").add(df_column("table", "b"));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into(), "b".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_nested_binary_expr() {
        // (a + b) * c
        let expr = df_column("table", "a")
            .add(df_column("table", "b"))
            .mul(df_column("table", "c"));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_not_expr() {
        let expr = Expr::Not(Box::new(df_column("table", "bool_col")));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["bool_col".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_alias_expr() {
        let expr = df_column("table", "col_x").alias("alias_name");
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["col_x".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_cast_expr() {
        let expr = Expr::Cast(Cast::new(
            Box::new(df_column("table", "num_col")),
            DataType::Int64,
        ));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["num_col".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_aggregate_function() {
        let expr = Expr::AggregateFunction(datafusion::logical_expr::expr::AggregateFunction {
            func_def: datafusion::logical_expr::expr::AggregateFunctionDefinition::BuiltIn(
                datafusion::physical_plan::aggregates::AggregateFunction::Sum,
            ),
            args: vec![df_column("table", "value")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        });
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["value".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_aggregate_function_with_multiple_args() {
        let expr = Expr::AggregateFunction(datafusion::logical_expr::expr::AggregateFunction {
            func_def: datafusion::logical_expr::expr::AggregateFunctionDefinition::BuiltIn(
                datafusion::physical_plan::aggregates::AggregateFunction::Sum,
            ),
            args: vec![
                df_column("table", "col1"),
                df_column("table", "col2"),
                df_column("table", "col3"),
            ],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        });
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["col1".into(), "col2".into(), "col3".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_no_column_idents_from_literal() {
        let expr = Expr::Literal(ScalarValue::Int32(Some(42)));
        let result = get_column_idents_from_expr(&expr);
        assert!(result.is_empty());
    }

    #[test]
    fn we_can_extract_column_idents_from_complex_nested_expr() {
        // NOT (a > b AND c < d)
        let inner = df_column("table", "a")
            .gt(df_column("table", "b"))
            .and(df_column("table", "c").lt(df_column("table", "d")));
        let expr = Expr::Not(Box::new(inner));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into(), "d".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_in_list_expr() {
        // a IN (b, c) references the needle column and every column in the list.
        let expr = df_column("table", "a").in_list(
            vec![df_column("table", "b"), df_column("table", "c")],
            false,
        );
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_preserving_order() {
        // IndexSet should preserve insertion order
        let expr = df_column("table", "z")
            .add(df_column("table", "a"))
            .add(df_column("table", "m"));
        let result = get_column_idents_from_expr(&expr);
        let idents: Vec<Ident> = result.into_iter().collect();
        assert_eq!(idents, vec!["z".into(), "a".into(), "m".into()]);
    }

    #[test]
    fn we_can_handle_duplicate_column_references() {
        // a + a should only have 'a' once
        let expr = df_column("table", "a").add(df_column("table", "a"));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_columns_from_comparison_operations() {
        let expr = df_column("table", "price")
            .gt(df_column("table", "threshold"))
            .and(df_column("table", "active").eq(Expr::Literal(ScalarValue::Boolean(Some(true)))));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["price".into(), "threshold".into(), "active".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }
}
