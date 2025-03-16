use super::{ExpressionEvaluationError, ExpressionEvaluationResult};
use alloc::{format, string::ToString, vec};
use datafusion::{
    common::ScalarValue,
    logical_expr::{BinaryExpr, Expr, Operator},
};
use proof_of_sql::base::{
    arrow::scalar_and_i256_conversions::convert_i256_to_scalar,
    database::{OwnedColumn, OwnedTable},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::Scalar,
};
use sqlparser::ast::Ident;

/// Evaluate a `DataFusion` logical expression on an [`OwnedTable`].
pub fn evaluate_expr<S: Scalar>(
    table: &OwnedTable<S>,
    expr: &Expr,
) -> ExpressionEvaluationResult<OwnedColumn<S>> {
    match expr {
        Expr::Column(column) => {
            let ident = Ident::new(column.name.as_str());
            evaluate_column(&ident, table)
        }
        Expr::Literal(lit) => evaluate_literal(lit, table.num_rows()),
        Expr::BinaryExpr(BinaryExpr { left, op, right }) => {
            let left_input = evaluate_expr(table, left)?;
            let right_input = evaluate_expr(table, right)?;
            evaluate_binary_expr(*op, &left_input, &right_input)
        }
        Expr::Not(expr) => {
            let input = evaluate_expr(table, expr)?;
            Ok(input.element_wise_not()?)
        }
        _ => Err(ExpressionEvaluationError::Unsupported {
            expression: format!("Expression {expr:?} is not supported yet"),
        }),
    }
}

fn evaluate_column<S: Scalar>(
    ident: &Ident,
    table: &OwnedTable<S>,
) -> ExpressionEvaluationResult<OwnedColumn<S>> {
    Ok(table
        .inner_table()
        .get(ident)
        .ok_or(ExpressionEvaluationError::ColumnNotFound {
            error: ident.to_string(),
        })?
        .clone())
}

fn evaluate_literal<S: Scalar>(
    lit: &ScalarValue,
    len: usize,
) -> ExpressionEvaluationResult<OwnedColumn<S>> {
    match lit {
        ScalarValue::Boolean(Some(b)) => Ok(OwnedColumn::Boolean(vec![*b; len])),
        ScalarValue::Int8(Some(i)) => Ok(OwnedColumn::TinyInt(vec![*i; len])),
        ScalarValue::Int16(Some(i)) => Ok(OwnedColumn::SmallInt(vec![*i; len])),
        ScalarValue::Int32(Some(i)) => Ok(OwnedColumn::Int(vec![*i; len])),
        ScalarValue::Int64(Some(i)) => Ok(OwnedColumn::BigInt(vec![*i; len])),
        ScalarValue::UInt8(Some(i)) => Ok(OwnedColumn::Uint8(vec![*i; len])),
        ScalarValue::Utf8(Some(s)) => Ok(OwnedColumn::VarChar(vec![s.clone(); len])),
        ScalarValue::Binary(Some(b)) => Ok(OwnedColumn::VarBinary(vec![b.clone(); len])),
        ScalarValue::TimestampSecond(Some(v), None) => Ok(OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            vec![*v; len],
        )),
        ScalarValue::TimestampMillisecond(Some(v), None) => Ok(OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Millisecond,
            PoSQLTimeZone::utc(),
            vec![*v; len],
        )),
        ScalarValue::TimestampMicrosecond(Some(v), None) => Ok(OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Microsecond,
            PoSQLTimeZone::utc(),
            vec![*v; len],
        )),
        ScalarValue::TimestampNanosecond(Some(v), None) => Ok(OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Nanosecond,
            PoSQLTimeZone::utc(),
            vec![*v; len],
        )),
        ScalarValue::Decimal128(Some(v), precision, scale) => Ok(OwnedColumn::Decimal75(
            Precision::new(*precision)?,
            *scale,
            vec![S::from(v); len],
        )),
        ScalarValue::Decimal256(Some(v), precision, scale) => Ok(OwnedColumn::Decimal75(
            Precision::new(*precision)?,
            *scale,
            vec![
                convert_i256_to_scalar(v).ok_or_else(|| {
                    ExpressionEvaluationError::Unsupported {
                        expression: "Decimal256 conversion failed.".to_string(),
                    }
                },)?;
                len
            ],
        )),
        _ => Err(ExpressionEvaluationError::Unsupported {
            expression: format!("Literal {lit:?} is not supported yet"),
        }),
    }
}

fn evaluate_binary_expr<S: Scalar>(
    op: Operator,
    left: &OwnedColumn<S>,
    right: &OwnedColumn<S>,
) -> ExpressionEvaluationResult<OwnedColumn<S>> {
    match op {
        Operator::And => Ok(left.element_wise_and(right)?),
        Operator::Or => Ok(left.element_wise_or(right)?),
        Operator::Eq => Ok(left.element_wise_eq(right)?),
        Operator::Gt => Ok(left.element_wise_gt(right)?),
        Operator::Lt => Ok(left.element_wise_lt(right)?),
        Operator::NotEq => Ok((left.element_wise_eq(right)?).element_wise_not()?),
        Operator::GtEq => Ok((left.element_wise_lt(right)?).element_wise_not()?),
        Operator::LtEq => Ok((left.element_wise_gt(right)?).element_wise_not()?),
        Operator::Plus => Ok(left.element_wise_add(right)?),
        Operator::Minus => Ok(left.element_wise_sub(right)?),
        Operator::Multiply => Ok(left.element_wise_mul(right)?),
        Operator::Divide => Ok(left.element_wise_div(right)?),
        _ => Err(ExpressionEvaluationError::Unsupported {
            expression: format!("Binary operator '{op}' is not supported."),
        }),
    }
}
