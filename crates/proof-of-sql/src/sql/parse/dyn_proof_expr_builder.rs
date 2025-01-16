use super::ConversionError;
use crate::{
    base::{database::ColumnRef, map::IndexMap, math::i256::I256},
    sql::proof_exprs::{ColumnExpr, DynProofExpr, ProofExpr},
};
use alloc::{boxed::Box, format, string::ToString, vec, vec::Vec};
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
use sqlparser::ast::{
    BinaryOperator, DataType, ExactNumberInfo, Expr, FunctionArg, FunctionArgExpr, Ident,
    ObjectName, UnaryOperator, Value,
};

/// Builder that enables building a `proofs::sql::proof_exprs::DynProofExpr` from
/// a `proof_of_sql_parser::intermediate_ast::Expression`.
pub struct DynProofExprBuilder<'a> {
    column_mapping: &'a IndexMap<Ident, ColumnRef>,
    in_agg_scope: bool,
}

impl<'a> DynProofExprBuilder<'a> {
    /// Creates a new `DynProofExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a IndexMap<Ident, ColumnRef>) -> Self {
        Self {
            column_mapping,
            in_agg_scope: false,
        }
    }
    /// Creates a new `DynProofExprBuilder` with the given column mapping and within aggregation scope.
    pub(crate) fn new_agg(column_mapping: &'a IndexMap<Ident, ColumnRef>) -> Self {
        Self {
            column_mapping,
            in_agg_scope: true,
        }
    }
    /// Builds a `proofs::sql::proof_exprs::DynProofExpr` from a `sqlparser::ast::Expr`
    pub fn build(&self, expr: &Expr) -> Result<DynProofExpr, ConversionError> {
        self.visit_expr(expr)
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
// Private interface
impl DynProofExprBuilder<'_> {
    fn visit_expr(&self, expr: &Expr) -> Result<DynProofExpr, ConversionError> {
        match expr {
            Expr::Identifier(identifier) => self.visit_column(identifier.clone()),
            Expr::Value(value) => self.visit_literal(&Expr::Value(value.clone())),
            Expr::BinaryOp { op, left, right } => {
                self.visit_binary_expr(op, left.as_ref(), right.as_ref())
            }
            Expr::UnaryOp { op, expr } => self.visit_unary_expr(*op, expr.as_ref()),
            Expr::Function(function) => {
                if let Some(FunctionArg::Unnamed(FunctionArgExpr::Expr(inner_expr))) =
                    function.args.first()
                {
                    return self.visit_aggregate_expr(&function.name.to_string(), inner_expr);
                }
                Err(ConversionError::Unprovable {
                    error: format!("Function {function:?} has unsupported arguments"),
                })
            }
            _ => Err(ConversionError::Unprovable {
                error: format!("Expression {expr:?} is not supported yet"),
            }),
        }
    }

    fn visit_column(&self, identifier: Ident) -> Result<DynProofExpr, ConversionError> {
        Ok(DynProofExpr::Column(ColumnExpr::new(
            self.column_mapping
                .get(&identifier)
                .ok_or(ConversionError::MissingColumnWithoutTable {
                    identifier: Box::new(identifier),
                })?
                .clone(),
        )))
    }

    /// Converts a `Expr` into a `DynProofExpr`
    ///
    /// # Panics
    /// - Panics if:
    ///   - `u8::try_from` for precision fails (precision out of range).
    ///   - `i8::try_from` for scale fails (scale out of range).
    ///   - A scalar string does not contain exactly 4 limbs.
    ///   - Parsing scalar limbs fails.
    ///
    /// # Examples
    /// ```
    /// let expr = Expr::Value(Value::Boolean(true));
    /// let dyn_expr = visit_literal(&expr).unwrap();
    /// ```
    #[allow(clippy::unused_self)]
    fn visit_literal(&self, expr: &Expr) -> Result<DynProofExpr, ConversionError> {
        match expr {
            Expr::Value(Value::Boolean(b)) => {
                Ok(DynProofExpr::new_literal(Expr::Value(Value::Boolean(*b))))
            }
            Expr::Value(Value::Number(value, _)) => value.parse::<i128>().map_or_else(
                |_| {
                    Err(ConversionError::InvalidNumberFormat {
                        value: value.clone(),
                    })
                },
                |n| {
                    let number_expr = Expr::Value(Value::Number(n.to_string(), false));
                    Ok(DynProofExpr::new_literal(number_expr))
                },
            ),
            Expr::Value(Value::SingleQuotedString(s)) => Ok(DynProofExpr::new_literal(
                Expr::Value(Value::SingleQuotedString(s.clone())),
            )),
            Expr::TypedString { data_type, value } => match data_type {
                DataType::Decimal(ExactNumberInfo::PrecisionAndScale(precision, scale)) => {
                    let parsed_value = I256::from_string(value).map_err(|_| {
                        ConversionError::InvalidDecimalFormat {
                            value: value.clone(),
                            precision: u8::try_from(*precision)
                                .expect("Precision must fit into u8"),
                            scale: i8::try_from(*scale).expect("Scale must fit into i8"),
                        }
                    })?;
                    Ok(DynProofExpr::new_literal(Expr::TypedString {
                        data_type: DataType::Decimal(ExactNumberInfo::PrecisionAndScale(
                            *precision, *scale,
                        )),
                        value: parsed_value.to_string(),
                    }))
                }
                DataType::Timestamp(Some(precision), tz) => {
                    let time_unit =
                        PoSQLTimeUnit::from_precision(*precision).unwrap_or(PoSQLTimeUnit::Second);
                    let parsed_value = value.parse::<i64>().map_err(|_| {
                        ConversionError::InvalidTimestampFormat {
                            value: value.clone(),
                        }
                    })?;
                    Ok(DynProofExpr::new_literal(Expr::TypedString {
                        data_type: DataType::Timestamp(Some(time_unit.into()), *tz),
                        value: parsed_value.to_string(),
                    }))
                }
                DataType::Custom(_, _) if data_type.to_string() == "scalar" => {
                    let scalar_str = value.strip_prefix("scalar:").unwrap_or_default();
                    let limbs: Vec<u64> = scalar_str
                        .split(',')
                        .map(|x| x.parse::<u64>().unwrap_or_default())
                        .collect();
                    assert!(limbs.len() == 4, "Scalar must have exactly 4 limbs");
                    Ok(DynProofExpr::new_literal(Expr::TypedString {
                        data_type: DataType::Custom(ObjectName(vec![]), vec![]),
                        value: format!("{},{},{},{}", limbs[0], limbs[1], limbs[2], limbs[3]),
                    }))
                }
                _ => Err(ConversionError::UnsupportedDataType {
                    data_type: data_type.to_string(),
                }),
            },
            _ => Err(ConversionError::UnsupportedLiteral {
                literal: format!("{expr:?}"),
            }),
        }
    }

    fn visit_unary_expr(
        &self,
        op: UnaryOperator,
        expr: &Expr,
    ) -> Result<DynProofExpr, ConversionError> {
        let expr = self.visit_expr(expr);
        match op {
            UnaryOperator::Not => DynProofExpr::try_new_not(expr?),
            // Handle unsupported operators
            _ => Err(ConversionError::UnsupportedOperation {
                message: format!("{op:?}"),
            }),
        }
    }

    fn visit_binary_expr(
        &self,
        op: &BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> Result<DynProofExpr, ConversionError> {
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
            BinaryOperator::Eq => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_equals(left?, right?)
            }
            BinaryOperator::GtEq => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_inequality(left?, right?, false)
            }
            BinaryOperator::LtEq => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_inequality(left?, right?, true)
            }
            BinaryOperator::Plus => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_add(left?, right?)
            }
            BinaryOperator::Minus => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_subtract(left?, right?)
            }
            BinaryOperator::Multiply => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                DynProofExpr::try_new_multiply(left?, right?)
            }
            BinaryOperator::Divide => Err(ConversionError::Unprovable {
                error: format!("Binary operator {op:?} is not supported at this location"),
            }),
            _ => {
                // Handle unsupported binary operations
                Err(ConversionError::UnsupportedOperation {
                    message: format!("{op:?}"),
                })
            }
        }
    }

    fn visit_aggregate_expr(&self, op: &str, expr: &Expr) -> Result<DynProofExpr, ConversionError> {
        if self.in_agg_scope {
            return Err(ConversionError::InvalidExpression {
                expression: "nested aggregations are invalid".to_string(),
            });
        }
        let expr = DynProofExprBuilder::new_agg(self.column_mapping).visit_expr(expr)?;

        match (op, expr.data_type().is_numeric()) {
            ("COUNT", _) | ("SUM", true) => Ok(DynProofExpr::new_aggregate(op, expr)?),
            ("SUM", false) => Err(ConversionError::InvalidExpression {
                expression: format!(
                    "Aggregation operator {op} doesn't work with non-numeric types"
                ),
            }),
            _ => Err(ConversionError::Unprovable {
                error: format!("Aggregation operator {op} is not supported at this location"),
            }),
        }
    }
}
