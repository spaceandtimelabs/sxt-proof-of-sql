use super::{ExpressionEvaluationError, ExpressionEvaluationResult};
use crate::base::{
    database::{owned_column::OwnedNullableColumn, OwnedColumn, OwnedTable},
    math::{
        decimal::{try_convert_intermediate_decimal_to_scalar, DecimalError, Precision},
        BigDecimalExt,
    },
    scalar::Scalar,
};
use alloc::{format, string::ToString, vec};
use proof_of_sql_parser::intermediate_ast::{Expression, Literal};
use sqlparser::ast::{BinaryOperator, Ident, UnaryOperator};

impl<S: Scalar> OwnedTable<S> {
    /// Evaluate an expression on the table.
    pub fn evaluate(&self, expr: &Expression) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        // Delegate to evaluate_nullable and unwrap the result if it's not nullable
        let nullable_result = self.evaluate_nullable(expr)?;

        // If the result has no NULL values, return the values directly
        if !nullable_result.is_nullable() {
            Ok(nullable_result.values)
        } else {
            // If the result has NULL values, we need to handle them
            Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Expression {expr:?} resulted in NULL values, but NULL values are not supported in this context"),
            })
        }
    }

    /// Evaluate an expression on the table, potentially returning NULL values.
    pub fn evaluate_nullable(
        &self,
        expr: &Expression,
    ) -> ExpressionEvaluationResult<OwnedNullableColumn<S>> {
        match expr {
            Expression::Column(identifier) => {
                self.evaluate_nullable_column(&Ident::from(*identifier))
            }
            Expression::Literal(lit) => self.evaluate_nullable_literal(lit),
            Expression::Binary { op, left, right } => {
                self.evaluate_nullable_binary_expr(&(*op).into(), left, right)
            }
            Expression::Unary { op, expr } => self.evaluate_nullable_unary_expr((*op).into(), expr),
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Expression {expr:?} is not supported yet"),
            }),
        }
    }

    fn evaluate_column(&self, identifier: &Ident) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        Ok(self
            .inner_table()
            .get(identifier)
            .ok_or(ExpressionEvaluationError::ColumnNotFound {
                error: identifier.to_string(),
            })?
            .clone())
    }

    fn evaluate_nullable_column(
        &self,
        identifier: &Ident,
    ) -> ExpressionEvaluationResult<OwnedNullableColumn<S>> {
        // Get the column from the table
        let column = self.evaluate_column(identifier)?;

        // Convert to a non-nullable OwnedNullableColumn
        Ok(OwnedNullableColumn::new(column))
    }

    fn evaluate_literal(&self, lit: &Literal) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let len = self.num_rows();
        match lit {
            Literal::Boolean(b) => Ok(OwnedColumn::Boolean(vec![*b; len])),
            Literal::BigInt(i) => Ok(OwnedColumn::BigInt(vec![*i; len])),
            Literal::Int128(i) => Ok(OwnedColumn::Int128(vec![*i; len])),
            Literal::Decimal(d) => {
                let raw_scale = d.scale();
                let scale = raw_scale
                    .try_into()
                    .map_err(|_| DecimalError::InvalidScale {
                        scale: raw_scale.to_string(),
                    })?;
                let precision = Precision::try_from(d.precision())?;
                let scalar = try_convert_intermediate_decimal_to_scalar(d, precision, scale)?;
                Ok(OwnedColumn::Decimal75(precision, scale, vec![scalar; len]))
            }
            Literal::VarChar(s) => Ok(OwnedColumn::VarChar(vec![s.clone(); len])),
            Literal::VarBinary(bytes) => Ok(OwnedColumn::VarBinary(vec![bytes.clone(); len])),
            Literal::Timestamp(its) => Ok(OwnedColumn::TimestampTZ(
                its.timeunit(),
                its.timezone(),
                vec![its.timestamp().timestamp(); len],
            )),
        }
    }

    fn evaluate_nullable_literal(
        &self,
        lit: &Literal,
    ) -> ExpressionEvaluationResult<OwnedNullableColumn<S>> {
        // Evaluate the literal as a non-nullable column
        let column = self.evaluate_literal(lit)?;

        // Convert to a non-nullable OwnedNullableColumn
        Ok(OwnedNullableColumn::new(column))
    }

    fn evaluate_nullable_unary_expr(
        &self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> ExpressionEvaluationResult<OwnedNullableColumn<S>> {
        let column = self.evaluate_nullable(expr)?;
        match op {
            UnaryOperator::Not => Ok(column.element_wise_not()?),
            // Handle unsupported unary operators
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Unary operator '{op}' is not supported."),
            }),
        }
    }

    fn evaluate_nullable_binary_expr(
        &self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> ExpressionEvaluationResult<OwnedNullableColumn<S>> {
        let left = self.evaluate_nullable(left)?;
        let right = self.evaluate_nullable(right)?;
        match op {
            BinaryOperator::And => Ok(left.element_wise_and(&right)?),
            BinaryOperator::Or => Ok(left.element_wise_or(&right)?),
            BinaryOperator::Eq => Ok(left.element_wise_eq(&right)?),
            BinaryOperator::Gt => Ok(left.element_wise_gt(&right)?),
            BinaryOperator::Lt => Ok(left.element_wise_lt(&right)?),
            BinaryOperator::Plus => Ok(left.element_wise_add(&right)?),
            BinaryOperator::Minus => Ok(left.element_wise_sub(&right)?),
            BinaryOperator::Multiply => Ok(left.element_wise_mul(&right)?),
            BinaryOperator::Divide => Ok(left.element_wise_div(&right)?),
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Binary operator '{op}' is not supported."),
            }),
        }
    }
}
