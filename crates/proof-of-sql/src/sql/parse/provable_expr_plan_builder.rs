use super::ConversionError;
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, LiteralValue},
        math::decimal::{try_into_to_scalar, DecimalError, Precision},
    },
    sql::ast::{ColumnExpr, ProvableExprPlan},
};
use proof_of_sql_parser::{
    intermediate_ast::{BinaryOperator, Expression, Literal, UnaryOperator},
    Identifier,
};
use std::collections::HashMap;

/// Builder that enables building a `proofs::sql::ast::ProvableExprPlan` from
/// a `proof_of_sql_parser::intermediate_ast::Expression`.
pub struct ProvableExprPlanBuilder<'a> {
    column_mapping: &'a HashMap<Identifier, ColumnRef>,
}

impl<'a> ProvableExprPlanBuilder<'a> {
    /// Creates a new `ProvableExprPlanBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a HashMap<Identifier, ColumnRef>) -> Self {
        Self { column_mapping }
    }
    /// Builds a `proofs::sql::ast::ProvableExprPlan` from a `proof_of_sql_parser::intermediate_ast::Expression`
    pub fn build<C: Commitment>(
        &self,
        expr: &Expression,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        self.visit_expr(expr)
    }
}

// Private interface
impl ProvableExprPlanBuilder<'_> {
    fn visit_expr<C: Commitment>(
        &self,
        expr: &Expression,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        match expr {
            Expression::Column(identifier) => self.visit_column(*identifier),
            Expression::Literal(lit) => self.visit_literal(lit),
            Expression::Binary { op, left, right } => self.visit_binary_expr(*op, left, right),
            Expression::Unary { op, expr } => self.visit_unary_expr(*op, expr),
            _ => Err(ConversionError::Unprovable(format!(
                "Expression {:?} is not supported yet",
                expr
            ))),
        }
    }

    fn visit_column<C: Commitment>(
        &self,
        identifier: Identifier,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        Ok(ProvableExprPlan::Column(ColumnExpr::new(
            *self.column_mapping.get(&identifier).ok_or(
                ConversionError::MissingColumnWithoutTable(Box::new(identifier)),
            )?,
        )))
    }

    fn visit_literal<C: Commitment>(
        &self,
        lit: &Literal,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        match lit {
            Literal::Boolean(b) => Ok(ProvableExprPlan::new_literal(LiteralValue::Boolean(*b))),
            Literal::BigInt(i) => Ok(ProvableExprPlan::new_literal(LiteralValue::BigInt(*i))),
            Literal::Int128(i) => Ok(ProvableExprPlan::new_literal(LiteralValue::Int128(*i))),
            Literal::Decimal(d) => {
                let scale = d.scale();
                let precision = Precision::new(d.precision()).map_err(|_| {
                    ConversionError::Decimal(DecimalError::InvalidPrecision(
                        d.precision().to_string(),
                    ))
                })?;
                Ok(ProvableExprPlan::new_literal(LiteralValue::Decimal75(
                    precision,
                    scale,
                    try_into_to_scalar(d, precision, scale)?,
                )))
            }
            Literal::VarChar(s) => Ok(ProvableExprPlan::new_literal(LiteralValue::VarChar((
                s.clone(),
                s.into(),
            )))),
        }
    }

    fn visit_unary_expr<C: Commitment>(
        &self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        let expr = self.visit_expr(expr);
        match op {
            UnaryOperator::Not => ProvableExprPlan::try_new_not(expr?),
        }
    }

    fn visit_binary_expr<C: Commitment>(
        &self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        match op {
            BinaryOperator::And => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                ProvableExprPlan::try_new_and(left?, right?)
            }
            BinaryOperator::Or => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                ProvableExprPlan::try_new_or(left?, right?)
            }
            BinaryOperator::Equal => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                ProvableExprPlan::try_new_equals(left?, right?)
            }
            BinaryOperator::GreaterThanOrEqual => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                ProvableExprPlan::try_new_inequality(left?, right?, false)
            }
            BinaryOperator::LessThanOrEqual => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                ProvableExprPlan::try_new_inequality(left?, right?, true)
            }
            BinaryOperator::Add => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                ProvableExprPlan::try_new_add(left?, right?)
            }
            BinaryOperator::Subtract => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                ProvableExprPlan::try_new_subtract(left?, right?)
            }
            BinaryOperator::Multiply | BinaryOperator::Division => Err(
                ConversionError::Unprovable(format!("Binary operator {:?} is not supported", op)),
            ),
        }
    }
}
