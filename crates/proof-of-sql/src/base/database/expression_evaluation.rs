use super::{ExpressionEvaluationError, ExpressionEvaluationResult};
use crate::base::{
    database::{OwnedColumn, OwnedTable},
    math::decimal::{try_into_to_scalar, Precision},
    scalar::Scalar,
};
use alloc::{format, string::ToString, vec};
use sqlparser::ast::{BinaryOperator, Expr, UnaryOperator, Ident, Value};

impl<S: Scalar> OwnedTable<S> {
    /// Evaluate an expression on the table.
    pub fn evaluate(&self, expr: &Expr) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        match expr {
            Expr::Identifier(identifier) => self.evaluate_column(identifier),
            Expr::Value(lit) => self.evaluate_literal(lit),
            Expr::BinaryOp { op, left, right } => self.evaluate_binary_expr(op.clone(), left, right),
            Expr::UnaryOp { op, expr } => self.evaluate_unary_expr(*op, expr),
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Expr {expr:?} is not supported yet"),
            }),
        }
    }

    fn evaluate_column(
        &self,
        identifier: &Ident,
    ) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        Ok(self
            .inner_table()
            .get(identifier)
            .ok_or(ExpressionEvaluationError::ColumnNotFound {
                error: identifier.to_string(),
            })?
            .clone())
    }

    fn evaluate_literal(&self, lit: &Value) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let len = self.num_rows();
        match lit {
            Value::Boolean(b) => Ok(OwnedColumn::Boolean(vec![*b; len])),
            Value::BigInt(i) => Ok(OwnedColumn::BigInt(vec![*i; len])),
            Value::Int128(i) => Ok(OwnedColumn::Int128(vec![*i; len])),
            Value::Decimal(d) => {
                let scale = d.scale();
                let precision = Precision::new(d.precision())?;
                let scalar = try_into_to_scalar(d, precision, scale)?;
                Ok(OwnedColumn::Decimal75(precision, scale, vec![scalar; len]))
            }
            Value::SingleQuotedString(s)
            |Value::DoubleQuotedString(s) => Ok(OwnedColumn::VarChar(vec![s.clone(); len])),
            Value::Timestamp(its) => Ok(OwnedColumn::TimestampTZ(
                its.timeunit(),
                its.timezone(),
                vec![its.timestamp().timestamp(); len],
            )),
        }
    }

    fn evaluate_unary_expr(
        &self,
        op: UnaryOperator,
        expr: &Expr,
    ) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let column = self.evaluate(expr)?;
        match op {
            UnaryOperator::Not => Ok(column.element_wise_not()?),
            _ => panic!("Unary operator not implemented: {op}"),
        }
    }

    fn evaluate_binary_expr(
        &self,
        op: BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match op {
            BinaryOperator::And => Ok(left.element_wise_and(&right)?),
            BinaryOperator::Or => Ok(left.element_wise_or(&right)?),
            BinaryOperator::Eq => Ok(left.element_wise_eq(&right)?),
            BinaryOperator::GtEq => Ok(left.element_wise_ge(&right)?),
            BinaryOperator::LtEq => Ok(left.element_wise_le(&right)?),
            BinaryOperator::Plus => Ok((left + right)?),
            BinaryOperator::Minus => Ok((left - right)?),
            BinaryOperator::Multiply => Ok((left * right)?),
            BinaryOperator::Divide => Ok((left / right)?),
            _ => panic!("Binary operator not implemented: {op}"),
        }
    }
}
