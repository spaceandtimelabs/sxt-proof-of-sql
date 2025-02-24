//! This module contains the conversion functions for converting sqlparser types to our own Logical Plan.
use super::{BinaryOperator, LogicalPlanError};
use crate::base::{
    database::LiteralValue,
    math::{
        decimal::{DecimalError, DecimalResult, IntermediateDecimalError, Precision},
        i256::I256,
        BigDecimalExt,
    },
};
use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use sqlparser::ast;

/// Parse a decimal value from a string
fn parse_decimal(value: &str) -> DecimalResult<LiteralValue> {
    let dec: BigDecimal =
        value
            .parse()
            .map_err(|e| DecimalError::IntermediateDecimalConversionError {
                source: IntermediateDecimalError::ParseError { error: e },
            })?;
    let precision = u8::try_from(dec.precision()).map_err(|_| DecimalError::InvalidPrecision {
        error: format!("Precision {} is too large", dec.precision()),
    })?;
    let scale = i8::try_from(dec.scale()).map_err(|_| DecimalError::InvalidScale {
        scale: format!("Scale {} is too large", dec.scale()),
    })?;
    let bigint = dec
        .try_into_bigint_with_precision_and_scale(precision, scale)
        .map_err(|source| DecimalError::IntermediateDecimalConversionError { source })?;
    // Since I256::from_num_bigint() doesn't error out on numbers out of range we need to check here
    if bigint.bits() > 255 || (bigint.clone() + BigInt::from(1_i64)).bits() > 255 {
        return Err(DecimalError::InvalidPrecision {
            error: format!("{bigint} is out of range for I256"),
        });
    }
    let i256 = I256::from_num_bigint(&bigint);
    let real_precision = Precision::new(precision).map_err(|e| DecimalError::InvalidPrecision {
        error: format!("Precision {e} is too large"),
    })?;
    Ok(LiteralValue::Decimal75(real_precision, scale, i256))
}

impl TryFrom<ast::Value> for LiteralValue {
    type Error = LogicalPlanError;

    fn try_from(value: ast::Value) -> Result<Self, Self::Error> {
        match value {
            ast::Value::Number(n, _) => {
                parse_decimal(&n).map_err(|source| LogicalPlanError::DecimalParseError { source })
            }
            ast::Value::SingleQuotedString(s) => Ok(LiteralValue::VarChar(s)),
            ast::Value::Boolean(b) => Ok(LiteralValue::Boolean(b)),
            _ => Err(LogicalPlanError::UnsupportedValue { value }),
        }
    }
}

impl TryFrom<ast::BinaryOperator> for BinaryOperator {
    type Error = LogicalPlanError;

    fn try_from(op: ast::BinaryOperator) -> Result<Self, Self::Error> {
        match op {
            ast::BinaryOperator::Eq => Ok(BinaryOperator::Eq),
            ast::BinaryOperator::NotEq => Ok(BinaryOperator::NotEq),
            ast::BinaryOperator::Gt => Ok(BinaryOperator::Gt),
            ast::BinaryOperator::Lt => Ok(BinaryOperator::Lt),
            ast::BinaryOperator::GtEq => Ok(BinaryOperator::GtEq),
            ast::BinaryOperator::LtEq => Ok(BinaryOperator::LtEq),
            ast::BinaryOperator::And => Ok(BinaryOperator::And),
            ast::BinaryOperator::Or => Ok(BinaryOperator::Or),
            ast::BinaryOperator::Plus => Ok(BinaryOperator::Plus),
            ast::BinaryOperator::Minus => Ok(BinaryOperator::Minus),
            ast::BinaryOperator::Multiply => Ok(BinaryOperator::Multiply),
            ast::BinaryOperator::Divide => Ok(BinaryOperator::Divide),
            _ => Err(LogicalPlanError::UnsupportedBinaryOperator { op }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Decimal parsing
    #[test]
    fn we_can_parse_a_decimal() {
        let decimal_str = "123.45";
        let result = parse_decimal(decimal_str).unwrap();
        assert_eq!(
            result,
            LiteralValue::Decimal75(Precision::new(5).unwrap(), 2, I256::from(12345))
        );
    }

    // Binary operators
    #[test]
    fn we_can_convert_supported_sqlparser_binary_operators() {
        // Let's test all our supported binary operators.
        let test_cases = vec![
            (ast::BinaryOperator::Eq, BinaryOperator::Eq),
            (ast::BinaryOperator::NotEq, BinaryOperator::NotEq),
            (ast::BinaryOperator::Gt, BinaryOperator::Gt),
            (ast::BinaryOperator::Lt, BinaryOperator::Lt),
            (ast::BinaryOperator::GtEq, BinaryOperator::GtEq),
            (ast::BinaryOperator::LtEq, BinaryOperator::LtEq),
            (ast::BinaryOperator::And, BinaryOperator::And),
            (ast::BinaryOperator::Or, BinaryOperator::Or),
            (ast::BinaryOperator::Plus, BinaryOperator::Plus),
            (ast::BinaryOperator::Minus, BinaryOperator::Minus),
            (ast::BinaryOperator::Multiply, BinaryOperator::Multiply),
            (ast::BinaryOperator::Divide, BinaryOperator::Divide),
        ];

        for (sql_op, expected) in test_cases {
            let result = BinaryOperator::try_from(sql_op).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn we_cannot_convert_unsupported_sqlparser_binary_operators() {
        // Let's test an unsupported operator.
        let unsupported_op = ast::BinaryOperator::Spaceship;
        let result = BinaryOperator::try_from(unsupported_op);
        assert!(matches!(
            result,
            Err(LogicalPlanError::UnsupportedBinaryOperator { .. })
        ));
    }
}
