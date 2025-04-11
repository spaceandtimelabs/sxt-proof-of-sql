use crate::{
    base::{database::ColumnType, math::decimal::Precision},
    sql::{
        proof_exprs::{DynProofExpr, ProofExpr},
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::string::ToString;
use core::cmp::Ordering;

/// Add a layer of decimal scaling cast to the expression
/// so that we can do binary operations on it
#[expect(clippy::missing_panics_doc, reason = "Precision can not be invalid")]
fn decimal_scale_cast_expr(
    from_proof_expr: DynProofExpr,
    from_scale: i8,
    to_scale: i8,
) -> AnalyzeResult<DynProofExpr> {
    if !from_proof_expr.data_type().is_numeric() {
        return Err(AnalyzeError::DataTypeMismatch {
            left_type: from_proof_expr.data_type().to_string(),
            right_type: "Some numeric type".to_string(),
        });
    }
    let from_precision_value = from_proof_expr.data_type().precision_value().unwrap_or(0);
    let to_precision_value = u8::try_from(
        i16::from(from_precision_value) + i16::from(to_scale - from_scale).min(75_i16),
    )
    .expect("Precision is definitely valid");
    DynProofExpr::try_new_scaling_cast(
        from_proof_expr,
        ColumnType::Decimal75(
            Precision::new(to_precision_value).expect("Precision is definitely valid"),
            to_scale,
        ),
    )
}

/// Scale cast one side so that both sides have the same scale
///
/// We use this function so that binary ops for numeric types no longer
/// need to keep track of scale
pub fn scale_cast_binary_op(
    left_proof_expr: DynProofExpr,
    right_proof_expr: DynProofExpr,
) -> AnalyzeResult<(DynProofExpr, DynProofExpr)> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::database::{ColumnRef, TableRef};

    #[expect(non_snake_case)]
    fn COLUMN1_BOOLEAN() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::Boolean,
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
    fn COLUMN1_DECIMAL_3_MINUS2() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::Decimal75(
                Precision::new(3).expect("Precision is definitely valid"),
                -2,
            ),
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN1_DECIMAL_10_5() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::Decimal75(
                Precision::new(10).expect("Precision is definitely valid"),
                5,
            ),
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN3_DECIMAL_75_10() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column3".into(),
            ColumnType::Decimal75(
                Precision::new(75).expect("Precision is definitely valid"),
                10,
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
            DynProofExpr::try_new_scaling_cast(
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
            Err(AnalyzeError::DataTypeMismatch { .. })
        ));
    }

    // scale_cast_binary_op
    #[test]
    fn we_can_convert_scale_cast_binary_op_upcasting_left() {
        let left_array = [
            COLUMN1_SMALLINT(),
            COLUMN1_DECIMAL_10_5(),
            COLUMN1_DECIMAL_3_MINUS2(),
        ];
        let right = COLUMN3_DECIMAL_75_10();
        for left in left_array {
            let proof_exprs = scale_cast_binary_op(left.clone(), right.clone()).unwrap();
            assert_eq!(
                proof_exprs,
                (
                    DynProofExpr::try_new_scaling_cast(
                        left,
                        ColumnType::Decimal75(
                            Precision::new(15).expect("Precision is definitely valid"),
                            10
                        )
                    )
                    .unwrap(),
                    COLUMN3_DECIMAL_75_10()
                )
            );
        }
    }

    #[test]
    fn we_can_convert_scale_cast_binary_op_upcasting_right() {
        let left = COLUMN3_DECIMAL_75_10();
        let right_array = [
            COLUMN1_SMALLINT(),
            COLUMN1_DECIMAL_10_5(),
            COLUMN1_DECIMAL_3_MINUS2(),
        ];
        for right in right_array {
            let proof_exprs = scale_cast_binary_op(left.clone(), right.clone()).unwrap();
            assert_eq!(
                proof_exprs,
                (
                    COLUMN3_DECIMAL_75_10(),
                    DynProofExpr::try_new_scaling_cast(
                        right,
                        ColumnType::Decimal75(
                            Precision::new(15).expect("Precision is definitely valid"),
                            10
                        )
                    )
                    .unwrap()
                )
            );
        }
    }

    #[test]
    fn we_can_convert_scale_cast_binary_op_equal() {
        let left = COLUMN1_DECIMAL_10_5();
        let right = COLUMN2_DECIMAL_25_5();
        let proof_exprs = scale_cast_binary_op(left.clone(), right.clone()).unwrap();
        assert_eq!(proof_exprs, (left, right));
    }
}
