use super::LogicalPlanError;
use crate::base::database::{ColumnRef, LiteralValue};
use alloc::{boxed::Box, format};
use serde::{Deserialize, Serialize};
use sqlparser::ast::BinaryOperator as SqlBinaryOperator;

/// Enum of column expressions that are either provable or supported in postprocessing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Column
    Column(ColumnRef),
    /// Provable CONST expression
    Literal(LiteralValue),
    /// Binary operation
    Binary {
        /// Left hand side of the binary operation
        left: Box<Expr>,
        /// Right hand side of the binary operation
        right: Box<Expr>,
        /// Binary operator
        op: BinaryOperator,
    },
    /// NOT expression
    Not(Box<Expr>),
}

/// Enum of binary operators we support
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    /// Equals
    Eq,
    /// Not equals
    Neq,
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Greater than or equals
    Gte,
    /// Less than or equals
    Lte,
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Add
    Add,
    /// Subtract
    Sub,
    /// Multiply
    Mul,
    /// Divide
    Div,
}

impl TryFrom<SqlBinaryOperator> for BinaryOperator {
    type Error = LogicalPlanError;

    fn try_from(op: SqlBinaryOperator) -> Result<Self, Self::Error> {
        match op {
            SqlBinaryOperator::Eq => Ok(Self::Eq),
            SqlBinaryOperator::NotEq => Ok(Self::Neq),
            SqlBinaryOperator::Gt => Ok(Self::Gt),
            SqlBinaryOperator::Lt => Ok(Self::Lt),
            SqlBinaryOperator::GtEq => Ok(Self::Gte),
            SqlBinaryOperator::LtEq => Ok(Self::Lte),
            SqlBinaryOperator::And => Ok(Self::And),
            SqlBinaryOperator::Or => Ok(Self::Or),
            SqlBinaryOperator::Plus => Ok(Self::Add),
            SqlBinaryOperator::Minus => Ok(Self::Sub),
            SqlBinaryOperator::Multiply => Ok(Self::Mul),
            SqlBinaryOperator::Divide => Ok(Self::Div),
            _ => Err(LogicalPlanError::Unsupported {
                message: format!("Unsupported binary operator: {op:?}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Binary operators
    #[test]
    fn we_can_convert_supported_sqlparser_binary_operators() {
        // Let's test all our supported binary operators.
        let test_cases = vec![
            (SqlBinaryOperator::Eq, BinaryOperator::Eq),
            (SqlBinaryOperator::NotEq, BinaryOperator::Neq),
            (SqlBinaryOperator::Gt, BinaryOperator::Gt),
            (SqlBinaryOperator::Lt, BinaryOperator::Lt),
            (SqlBinaryOperator::GtEq, BinaryOperator::Gte),
            (SqlBinaryOperator::LtEq, BinaryOperator::Lte),
            (SqlBinaryOperator::And, BinaryOperator::And),
            (SqlBinaryOperator::Or, BinaryOperator::Or),
            (SqlBinaryOperator::Plus, BinaryOperator::Add),
            (SqlBinaryOperator::Minus, BinaryOperator::Sub),
            (SqlBinaryOperator::Multiply, BinaryOperator::Mul),
            (SqlBinaryOperator::Divide, BinaryOperator::Div),
        ];

        for (sql_op, expected) in test_cases {
            let result = BinaryOperator::try_from(sql_op).unwrap();
            assert_eq!(result, expected,);
        }
    }

    #[test]
    fn we_cannot_convert_unsupported_sqlparser_binary_operators() {
        // Let's test an unsupported operator.
        let unsupported_op = SqlBinaryOperator::Spaceship;
        let result = BinaryOperator::try_from(unsupported_op);
        assert!(matches!(result, Err(LogicalPlanError::Unsupported { .. })));
    }
}
