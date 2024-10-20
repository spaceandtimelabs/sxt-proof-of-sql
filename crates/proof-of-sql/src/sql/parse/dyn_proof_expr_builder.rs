use super::ConversionError;
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, LiteralValue},
        map::IndexMap,
        math::{
            decimal::{try_convert_intermediate_decimal_to_scalar, DecimalError, Precision},
            IntermediateDecimalExt,
        },
    },
    sql::{
        parse::{
            dyn_proof_expr_builder::DecimalError::{InvalidPrecision, InvalidScale},
            ConversionError::DecimalConversionError,
        },
        proof_exprs::{ColumnExpr, DynProofExpr, ProofExpr},
    },
};
use alloc::{borrow::ToOwned, boxed::Box, format, string::ToString};
use proof_of_sql_parser::{
    intermediate_ast::{AggregationOperator, BinaryOperator, Expression, Literal, UnaryOperator},
    posql_time::{PoSQLTimeUnit, PoSQLTimestampError},
    Identifier,
};

/// Builder that enables building a `proofs::sql::proof_exprs::DynProofExpr` from
/// a `proof_of_sql_parser::intermediate_ast::Expression`.
pub struct DynProofExprBuilder<'a> {
    column_mapping: &'a IndexMap<Identifier, ColumnRef>,
    in_agg_scope: bool,
}

impl<'a> DynProofExprBuilder<'a> {
    /// Creates a new `DynProofExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a IndexMap<Identifier, ColumnRef>) -> Self {
        Self {
            column_mapping,
            in_agg_scope: false,
        }
    }
    /// Creates a new `DynProofExprBuilder` with the given column mapping and within aggregation scope.
    pub(crate) fn new_agg(column_mapping: &'a IndexMap<Identifier, ColumnRef>) -> Self {
        Self {
            column_mapping,
            in_agg_scope: true,
        }
    }
    /// Builds a `proofs::sql::proof_exprs::DynProofExpr` from a `proof_of_sql_parser::intermediate_ast::Expression`
    pub fn build<C: Commitment>(
        &self,
        expr: &Expression,
    ) -> Result<DynProofExpr<C>, ConversionError> {
        self.visit_expr(expr)
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
// Private interface
impl DynProofExprBuilder<'_> {
    fn visit_expr<C: Commitment>(
        &self,
        expr: &Expression,
    ) -> Result<DynProofExpr<C>, ConversionError> {
        match expr {
            Expression::Column(identifier) => self.visit_column(*identifier),
            Expression::Literal(lit) => self.visit_literal(lit),
            Expression::Binary { op, left, right } => self.visit_binary_expr(*op, left, right),
            Expression::Unary { op, expr } => self.visit_unary_expr(*op, expr),
            Expression::Aggregation { op, expr } => self.visit_aggregate_expr(*op, expr),
            _ => Err(ConversionError::Unprovable {
                error: format!("Expression {expr:?} is not supported yet"),
            }),
        }
    }

    fn visit_column<C: Commitment>(
        &self,
        identifier: Identifier,
    ) -> Result<DynProofExpr<C>, ConversionError> {
        Ok(DynProofExpr::Column(ColumnExpr::new(
            *self.column_mapping.get(&identifier).ok_or(
                ConversionError::MissingColumnWithoutTable {
                    identifier: Box::new(identifier),
                },
            )?,
        )))
    }

    #[allow(clippy::unused_self)]
    fn visit_literal<C: Commitment>(
        &self,
        lit: &Literal,
    ) -> Result<DynProofExpr<C>, ConversionError> {
        match lit {
            Literal::Boolean(b) => Ok(DynProofExpr::new_literal(LiteralValue::Boolean(*b))),
            Literal::BigInt(i) => Ok(DynProofExpr::new_literal(LiteralValue::BigInt(*i))),
            Literal::Int128(i) => Ok(DynProofExpr::new_literal(LiteralValue::Int128(*i))),
            Literal::Decimal(d) => {
                let raw_scale = d.scale();
                let scale = raw_scale.try_into().map_err(|_| InvalidScale {
                    scale: raw_scale.to_string(),
                })?;
                let precision =
                    Precision::try_from(d.precision()).map_err(|_| DecimalConversionError {
                        source: InvalidPrecision {
                            error: d.precision().to_string(),
                        },
                    })?;
                Ok(DynProofExpr::new_literal(LiteralValue::Decimal75(
                    precision,
                    scale,
                    try_convert_intermediate_decimal_to_scalar(d, precision, scale)?,
                )))
            }
            Literal::VarChar(s) => Ok(DynProofExpr::new_literal(LiteralValue::VarChar((
                s.clone(),
                s.into(),
            )))),
            Literal::Timestamp(its) => {
                let timestamp = match its.timeunit() {
                    PoSQLTimeUnit::Nanosecond => {
                        its.timestamp().timestamp_nanos_opt().ok_or_else(|| {
                                PoSQLTimestampError::UnsupportedPrecision{ error: "Timestamp out of range: 
                                Valid nanosecond timestamps must be between 1677-09-21T00:12:43.145224192 
                                and 2262-04-11T23:47:16.854775807.".to_owned()
                        }
                        })?
                    }
                    PoSQLTimeUnit::Microsecond => its.timestamp().timestamp_micros(),
                    PoSQLTimeUnit::Millisecond => its.timestamp().timestamp_millis(),
                    PoSQLTimeUnit::Second => its.timestamp().timestamp(),
                };

                Ok(DynProofExpr::new_literal(LiteralValue::TimeStampTZ(
                    its.timeunit(),
                    its.timezone(),
                    timestamp,
                )))
            }
        }
    }

    fn visit_unary_expr<C: Commitment>(
        &self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> Result<DynProofExpr<C>, ConversionError> {
        let expr = self.visit_expr(expr);
        match op {
            UnaryOperator::Not => DynProofExpr::try_new_not(expr?),
        }
    }

    fn visit_binary_expr<C: Commitment>(
        &self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<DynProofExpr<C>, ConversionError> {
        match op {
            BinaryOperator::And => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_and(left?, right?)
            }
            BinaryOperator::Or => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_or(left?, right?)
            }
            BinaryOperator::Equal => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_equals(left?, right?)
            }
            BinaryOperator::GreaterThanOrEqual => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_inequality(left?, right?, false)
            }
            BinaryOperator::LessThanOrEqual => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_inequality(left?, right?, true)
            }
            BinaryOperator::Add => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_add(left?, right?)
            }
            BinaryOperator::Subtract => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_subtract(left?, right?)
            }
            BinaryOperator::Multiply => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_multiply(left?, right?)
            }
            BinaryOperator::Division => Err(ConversionError::Unprovable {
                error: format!("Binary operator {op:?} is not supported at this location"),
            }),
        }
    }

    fn visit_aggregate_expr<C: Commitment>(
        &self,
        op: AggregationOperator,
        expr: &Expression,
    ) -> Result<DynProofExpr<C>, ConversionError> {
        if self.in_agg_scope {
            return Err(ConversionError::InvalidExpression {
                expression: "nested aggregations are invalid".to_string(),
            });
        }
        let expr = DynProofExprBuilder::new_agg(self.column_mapping).visit_expr(expr)?;
        match (op, expr.data_type().is_numeric()) {
            (AggregationOperator::Count, _) | (AggregationOperator::Sum, true) => {
                Ok(DynProofExpr::new_aggregate(op, expr))
            }
            (AggregationOperator::Sum, false) => Err(ConversionError::InvalidExpression {
                expression: format!(
                    "Aggregation operator {op:?} doesn't work with non-numeric types"
                ),
            }),
            _ => Err(ConversionError::Unprovable {
                error: format!("Aggregation operator {op:?} is not supported at this location"),
            }),
        }
    }
}
