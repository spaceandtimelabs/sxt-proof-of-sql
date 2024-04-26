use super::ConversionError;
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, ColumnType},
        math::decimal::match_decimal,
    },
    sql::ast::ProvableExprPlan,
};
use proofs_sql::{
    decimal_unknown::DecimalUnknown,
    intermediate_ast::{BinaryOperator, Expression, Literal, UnaryOperator},
    Identifier,
};
use std::collections::HashMap;

/// Builder that enables building a `proofs::sql::ast::BoolExpr` from a `proofs_sql::intermediate_ast::Expression` that is
/// intended to be used as the where clause in a filter expression or group by expression.
pub struct WhereExprBuilder<'a> {
    column_mapping: &'a HashMap<Identifier, ColumnRef>,
}
impl<'a> WhereExprBuilder<'a> {
    /// Creates a new `WhereExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a HashMap<Identifier, ColumnRef>) -> Self {
        Self { column_mapping }
    }
    /// Builds a `proofs::sql::ast::BoolExpr` from a `proofs_sql::intermediate_ast::Expression` that is
    /// intended to be used as the where clause in a filter expression or group by expression.
    pub fn build<C: Commitment>(
        self,
        where_expr: Option<Box<Expression>>,
    ) -> Result<Option<ProvableExprPlan<C>>, ConversionError> {
        where_expr
            .map(|where_expr| self.visit_expr(*where_expr))
            .transpose()
    }
}

// Private interface
impl WhereExprBuilder<'_> {
    fn visit_expr<C: Commitment>(
        &self,
        expr: proofs_sql::intermediate_ast::Expression,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        match expr {
            Expression::Binary { op, left, right } => self.visit_binary_expr(op, *left, *right),
            Expression::Unary { op, expr } => self.visit_unary_expr(op, *expr),
            _ => panic!("The parser must ensure that the expression is a boolean expression"),
        }
    }

    fn visit_unary_expr<C: Commitment>(
        &self,
        op: UnaryOperator,
        expr: Expression,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        let expr = self.visit_expr(expr);

        match op {
            UnaryOperator::Not => Ok(ProvableExprPlan::new_not(expr?)),
        }
    }

    fn visit_binary_expr<C: Commitment>(
        &self,
        op: BinaryOperator,
        left: Expression,
        right: Expression,
    ) -> Result<ProvableExprPlan<C>, ConversionError> {
        match op {
            BinaryOperator::And => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                Ok(ProvableExprPlan::new_and(left?, right?))
            }
            BinaryOperator::Or => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                Ok(ProvableExprPlan::new_or(left?, right?))
            }
            BinaryOperator::Equal => {
                let (left, right) = self.process_comparison_expr::<C>(left, right)?;
                Ok(ProvableExprPlan::new_equals(left, right))
            }
            BinaryOperator::GreaterThanOrEqual => {
                let (left, right) = self.process_comparison_expr::<C>(left, right)?;
                Ok(ProvableExprPlan::new_inequality(left, right, false))
            }
            BinaryOperator::LessThanOrEqual => {
                let (left, right) = self.process_comparison_expr::<C>(left, right)?;
                Ok(ProvableExprPlan::new_inequality(left, right, true))
            }
            _ => panic!("The parser must ensure that the expression is a boolean expression"),
        }
    }

    /// Ensure that left is a Column and right is a Literal, then get the column reference and the literal value
    fn process_comparison_expr<C: Commitment>(
        &self,
        left: Expression,
        right: Expression,
    ) -> Result<(ColumnRef, C::Scalar), ConversionError> {
        let left = match left {
            Expression::Column(identifier) => *self.column_mapping.get(&identifier).unwrap(),
            _ => panic!("The parser must ensure that the left side is a column"),
        };

        let right = match (right, left.column_type()) {
            (Expression::Literal(Literal::Decimal(d)), column_type) => match column_type {
                ColumnType::Decimal75(_, scale) => match_decimal(&d, *scale)?,
                ColumnType::Int128 if d.scale > 0 => {
                    return Err(ConversionError::DataTypeMismatch(
                        d.value().to_owned(),
                        "Int128".to_owned(),
                    ));
                }
                ColumnType::BigInt if d.scale > 0 => {
                    return Err(ConversionError::DataTypeMismatch(
                        d.value().to_owned(),
                        "Int64".to_owned(),
                    ));
                }
                // 123.000 should match to 123, guarded by the match above
                ColumnType::Int128 => match_decimal(&d, 0)?,
                ColumnType::BigInt => match_decimal(&d, 0)?,
                ColumnType::VarChar | ColumnType::Scalar | ColumnType::Boolean => {
                    return Err(ConversionError::DataTypeMismatch(
                        format!("Decimal75: {}", d.value()),
                        left.column_type().to_string(),
                    ));
                }
            },
            (Expression::Literal(Literal::Int128(int)), ColumnType::Decimal75(_, scale)) => {
                match_decimal(&DecimalUnknown::new(&int.to_string()), *scale)?
            }
            (Expression::Literal(Literal::Int128(value)), _) => value.into(),
            (Expression::Literal(Literal::VarChar(value)), _) => value.into(),
            _ => panic!("Unexpected expression or column type"),
        };
        Ok((left, right))
    }
}
