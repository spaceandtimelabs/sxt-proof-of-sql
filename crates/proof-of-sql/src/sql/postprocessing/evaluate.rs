use super::PostprocessingError;
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, LiteralValue},
        math::decimal::{try_into_to_scalar, DecimalError::InvalidPrecision, Precision},
    },
    sql::{
        ast::{
            add_subtract_owned_columns, divide_owned_columns, multiply_owned_columns, ColumnExpr,
            ProvableExpr, ProvableExprPlan,
        },
        parse::PostprocessingError::DecimalPostprocessingError,
    },
};
use proof_of_sql_parser::{
    intermediate_ast::{AggregationOperator, BinaryOperator, Expression, Literal, UnaryOperator},
    Identifier,
};
use std::collections::HashMap;

/// Evaluator that evaluates a `proof_of_sql_parser::intermediate_ast::Expression`
/// on an `OwnedTable` and returns an `OwnedColumn`.
pub struct PostprocessingEvaluator<S: Scalar> {
    table: OwnedTable<S>,
    in_agg_scope: bool,
}

impl<'a> PostprocessingEvaluator<'a> {
    /// Creates a new `PostprocessingEvaluator` with the given column mapping.
    pub fn new(table: OwnedTable<S>) -> Self {
        Self {
            table,
            in_agg_scope: false,
        }
    }
    /// Creates a new `PostprocessingEvaluator` with the given column mapping and within aggregation scope.
    pub(crate) fn new_agg(table: OwnedTable<S>) -> Self {
        Self {
            table,
            in_agg_scope: true,
        }
    }
    /// Builds a `proofs::sql::ast::ProvableExprPlan` from a `proof_of_sql_parser::intermediate_ast::Expression`
    pub fn build<S: Scalar>(
        &self,
        expr: &Expression,
    ) -> Result<OwnedColumn<S>, PostprocessingError> {
        self.visit_expr(expr)
    }
}

// Private interface
impl PostprocessingEvaluator<'_> {
    fn visit_expr<S: Scalar>(
        &self,
        expr: &Expression,
    ) -> Result<OwnedColumn<S>, PostprocessingError> {
        match expr {
            Expression::Column(identifier) => self.visit_column(*identifier),
            Expression::Literal(lit) => self.visit_literal(lit),
            Expression::Binary { op, left, right } => self.visit_binary_expr(*op, left, right),
            Expression::Unary { op, expr } => self.visit_unary_expr(*op, expr),
            _ => Err(PostprocessingError::Unsupported(format!(
                "Expression {:?} is not supported yet",
                expr
            ))),
        }
    }

    fn visit_column<S: Scalar>(
        &self,
        identifier: Identifier,
    ) -> Result<OwnedColumn<S>, PostprocessingError> {
        Ok(self
            .table
            .inner_table()
            .get(&identifier)
            .ok_or(PostprocessingError::ColumnNotFound(identifier.to_string()))?)
    }

    fn visit_literal<S: Scalar>(
        &self,
        lit: &Literal,
    ) -> Result<OwnedColumn<S>, PostprocessingError> {
        let len = self.table.num_rows();
        match lit {
            Literal::Boolean(b) => Ok(OwnedColumn::Boolean(vec![*b; len])),
            Literal::BigInt(i) => Ok(OwnedColumn::BigInt(vec![*i; len])),
            Literal::Int128(i) => Ok(OwnedColumn::Int128(vec![*i; len])),
            Literal::Decimal(d) => {
                let scale = d.scale();
                let precision = Precision::new(d.precision())?;
                let scalar = try_into_to_scalar(d, precision, scale)?;
                Ok(OwnedColumn::Decimal75(precision, scale, vec![scalar; len]))
            }
            Literal::VarChar(s) => Ok(OwnedColumn::VarChar(vec![s.clone(); len])),
            Literal::Timestamp(its) => Ok(OwnedColumn::Timestamp(
                its.timeunit,
                its.timezone,
                vec![its.timestamp.timestamp(); len],
            )),
        }
    }

    fn visit_unary_expr<S: Scalar>(
        &self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> Result<OwnedColumn<S>, PostprocessingError> {
        let column = self.visit_expr(expr)?;
        match op {
            UnaryOperator::Not => {
                if column.data_type() == &DataType::Boolean {
                    Ok(OwnedColumn::Boolean(
                        column.into_iter().map(|b| !b).collect(),
                    ))
                } else {
                    Err(PostprocessingError::InvalidExpression(
                        "Unary operator NOT is only valid for boolean expressions".to_string(),
                    ))
                }
            }
        }
    }

    fn visit_binary_expr<S: Scalar>(
        &self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<OwnedColumn<S>, PostprocessingError> {
        let left = self.visit_expr(left?);
        let right = self.visit_expr(right?);
        check_dtypes(left.data_type(), right.data_type(), op)?;
        match op {
            BinaryOperator::And => Ok(OwnedColumn::Boolean(
                left.into_iter()
                    .zip(right.into_iter())
                    .map(|(l, r)| l && r)
                    .collect(),
            )),
            BinaryOperator::Or => Ok(OwnedColumn::Boolean(
                left.into_iter()
                    .zip(right.into_iter())
                    .map(|(l, r)| l || r)
                    .collect(),
            )),
            BinaryOperator::Equal
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::LessThanOrEqual => {
                let left_scale = left.data_type().scale();
                let right_scale = right.data_type().scale();
                let max_scale = left_scale.max(right_scale);
                let lhs_upscale_factor =
                    scale_scalar(S::ONE, max_scale - lhs_scale).expect("Invalid scale factor");
                let rhs_upscale_factor =
                    scale_scalar(S::ONE, max_scale - rhs_scale).expect("Invalid scale factor");
                match op {
                    BinaryOperator::Equal => {
                        let res: Vec<bool> = (0..lhs_len)
                            .map(|i| {
                                lhs.scalar_at(i) * lhs_upscale_factor
                                    == rhs.scalar_at(i) * rhs_upscale_factor
                            })
                            .collect();
                        OwnedColumn::Boolean(res)
                    }
                    BinaryOperator::GreaterThanOrEqual => {
                        let res: Vec<bool> = (0..lhs_len)
                            .map(|i| {
                                lhs.scalar_at(i)
                                    * lhs_upscale_factor
                                        .signed_cmp(rhs.scalar_at(i) * rhs_upscale_factor)
                                    != Ordering::Less
                            })
                            .collect();
                        OwnedColumn::Boolean(res)
                    }
                    BinaryOperator::LessThanOrEqual => {
                        let res: Vec<bool> = (0..lhs_len)
                            .map(|i| {
                                lhs.scalar_at(i)
                                    * lhs_upscale_factor
                                        .signed_cmp(rhs.scalar_at(i) * rhs_upscale_factor)
                                    != Ordering::Greater
                            })
                            .collect();
                        OwnedColumn::Boolean(res)
                    }
                    _ => unreachable!(),
                }
            }
            BinaryOperator::Add => add_subtract_owned_columns(left, right, false),
            BinaryOperator::Subtract => add_subtract_owned_columns(left, right, true),
            BinaryOperator::Multiply => multiply_owned_columns(left, right),
            BinaryOperator::Division => divide_owned_columns(left, right),
        }
    }
}

/// If the two column types are compatible for a binary operation, return the result type.
/// Otherwise, return None.
pub(crate) fn get_binary_operation_result_type(
    left_dtype: &ColumnType,
    right_dtype: &ColumnType,
    binary_operator: BinaryOperator,
) -> Option<ColumnType> {
    match binary_operator {
        BinaryOperator::And | BinaryOperator::Or => matches!(
            (left_dtype, right_dtype),
            (ColumnType::Boolean, ColumnType::Boolean)
        )
        .then_some(ColumnType::Boolean),
        BinaryOperator::Equal => {
            matches!(
                (left_dtype, right_dtype),
                (ColumnType::VarChar, ColumnType::VarChar)
                    | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                    | (ColumnType::Boolean, ColumnType::Boolean)
                    | (_, ColumnType::Scalar)
                    | (ColumnType::Scalar, _)
            ) || (left_dtype.is_numeric() && right_dtype.is_numeric())
                .then_some(ColumnType::Boolean)
        }
        BinaryOperator::GreaterThanOrEqual | BinaryOperator::LessThanOrEqual => {
            if left_dtype == &ColumnType::VarChar || right_dtype == &ColumnType::VarChar {
                return None;
            }
            // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
            if let ColumnType::Decimal75(precision, _) = left_dtype {
                if precision.value() > 38 {
                    return None;
                }
            }
            if let ColumnType::Decimal75(precision, _) = right_dtype {
                if precision.value() > 38 {
                    return None;
                }
            }
            // TODO: inequality support for timestamps
            left_dtype.is_numeric() && right_dtype.is_numeric()
                || matches!(
                    (left_dtype, right_dtype),
                    (ColumnType::Boolean, ColumnType::Boolean)
                )
                .then_some(ColumnType::Boolean)
        }
        BinaryOperator::Add | BinaryOperator::Subtract => {
            try_add_subtract_column_types(*left_dtype, *right_dtype).ok()
        }
        BinaryOperator::Multiply => try_multiply_column_types(*left_dtype, *right_dtype).ok(),
        BinaryOperator::Division => try_divide_column_types(*left_dtype, *right_dtype).ok(),
    }
}

fn check_dtypes(
    left_dtype: ColumnType,
    right_dtype: ColumnType,
    binary_operator: BinaryOperator,
) -> ConversionResult<()> {
    if type_check_binary_operation(&left_dtype, &right_dtype, binary_operator) {
        Ok(())
    } else {
        Err(ConversionError::DataTypeMismatch(
            left_dtype.to_string(),
            right_dtype.to_string(),
        ))
    }
}
