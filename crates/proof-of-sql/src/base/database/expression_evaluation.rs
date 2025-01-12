use super::{ExpressionEvaluationError, ExpressionEvaluationResult};
use crate::base::{
    database::{OwnedColumn, OwnedTable},
    math::decimal::{try_convert_intermediate_decimal_to_scalar, DecimalError, Precision},
    scalar::Scalar,
};
use alloc::{format, string::ToString, vec};
use bigdecimal::BigDecimal;
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
use sqlparser::ast::{
    BinaryOperator, DataType, ExactNumberInfo, Expr, Ident, UnaryOperator, Value,
};

impl<S: Scalar> OwnedTable<S> {
    /// Evaluate an expression on the table.
    pub fn evaluate(&self, expr: &Expr) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        match expr {
            Expr::Identifier(ident) => self.evaluate_column(ident),
            Expr::Value(_) | Expr::TypedString { .. } => self.evaluate_literal(expr),
            Expr::BinaryOp { op, left, right } => self.evaluate_binary_expr(op, left, right),
            Expr::UnaryOp { op, expr } => self.evaluate_unary_expr(*op, expr),
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
    /// Evaluates a literal expression and returns its corresponding column representation.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - `BigDecimal::parse_bytes` fails to parse a valid decimal string.
    /// - `Precision::try_from` fails due to invalid precision or scale values.
    fn evaluate_literal(&self, value: &Expr) -> ExpressionEvaluationResult<OwnedColumn<S>> {
        let len = self.num_rows();
        match value {
            Expr::Value(Value::Boolean(b)) => Ok(OwnedColumn::Boolean(vec![*b; len])),
            Expr::Value(Value::Number(n, _)) => {
                let num = n
                    .parse::<i128>()
                    .map_err(|_| DecimalError::InvalidDecimal {
                        error: format!("Invalid number: {n}"),
                    })?;
                if num >= i128::from(i64::MIN) && num <= i128::from(i64::MAX) {
                    Ok(OwnedColumn::BigInt(vec![num.try_into().unwrap(); len]))
                } else {
                    Ok(OwnedColumn::Int128(vec![num; len]))
                }
            }
            Expr::Value(Value::SingleQuotedString(s)) => {
                Ok(OwnedColumn::VarChar(vec![s.clone(); len]))
            }
            Expr::TypedString { data_type, value } => match data_type {
                DataType::Decimal(ExactNumberInfo::PrecisionAndScale(raw_precision, raw_scale)) => {
                    let decimal = BigDecimal::parse_bytes(value.as_bytes(), 10).unwrap();
                    let precision = Precision::try_from(*raw_precision).map_err(|_| {
                        DecimalError::InvalidPrecision {
                            error: raw_precision.to_string(),
                        }
                    })?;
                    let scale =
                        i8::try_from(*raw_scale).map_err(|_| DecimalError::InvalidScale {
                            scale: raw_scale.to_string(),
                        })?;
                    let scalar =
                        try_convert_intermediate_decimal_to_scalar(&decimal, precision, scale)?;
                    Ok(OwnedColumn::Decimal75(precision, scale, vec![scalar; len]))
                }
                DataType::Timestamp(Some(time_unit), time_zone) => {
                    let time_unit = PoSQLTimeUnit::from_precision(*time_unit).map_err(|err| {
                        DecimalError::InvalidDecimal {
                            error: format!("Invalid time unit precision: {err}"),
                        }
                    })?;

                    let timestamp_value =
                        value
                            .parse::<i64>()
                            .map_err(|_| DecimalError::InvalidDecimal {
                                error: format!("Invalid timestamp value: {value}"),
                            })?;
                    Ok(OwnedColumn::TimestampTZ(
                        time_unit,
                        *time_zone,
                        vec![timestamp_value; len],
                    ))
                }
                _ => Err(ExpressionEvaluationError::Unsupported {
                    expression: "Unsupported TypedString data type".to_string(),
                }),
            },
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: "Unsupported expression type".to_string(),
            }),
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
            // Handle unsupported unary operators
            _ => Err(ExpressionEvaluationError::Unsupported {
                expression: format!("Unary operator '{op}' is not supported."),
            }),
        }
    }

    fn evaluate_binary_expr(
        &self,
        op: &BinaryOperator,
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
