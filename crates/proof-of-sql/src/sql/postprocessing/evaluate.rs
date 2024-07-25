use super::{PostprocessingError, PostprocessingResult};
use crate::{
    base::{
        database::{ColumnType, OwnedColumn, OwnedTable},
        math::decimal::{scale_scalar, try_into_to_scalar, Precision},
        scalar::Scalar,
    },
    sql::ast::{
        add_subtract_owned_columns, multiply_owned_columns, try_add_subtract_column_types,
        try_divide_column_types, try_divide_owned_columns, try_multiply_column_types,
    },
};
use core::cmp::Ordering;
use proof_of_sql_parser::{
    intermediate_ast::{BinaryOperator, Expression, Literal, UnaryOperator},
    Identifier,
};

/// Evaluator that evaluates a `proof_of_sql_parser::intermediate_ast::Expression`
/// on an `OwnedTable` and returns an `OwnedColumn`.
pub struct PostprocessingEvaluator<S: Scalar> {
    table: OwnedTable<S>,
}

impl<S: Scalar> PostprocessingEvaluator<S> {
    /// Creates a new `PostprocessingEvaluator` with the given owned table.
    pub fn new(table: &OwnedTable<S>) -> Self {
        Self {
            table: table.clone(),
        }
    }
    /// Evaluates the given expression on the owned table and returns the result.
    pub fn evaluate(&self, expr: &Expression) -> PostprocessingResult<OwnedColumn<S>> {
        self.visit_expr(expr)
    }
}

// Private interface
impl<S: Scalar> PostprocessingEvaluator<S> {
    fn visit_expr(&self, expr: &Expression) -> PostprocessingResult<OwnedColumn<S>> {
        match expr {
            Expression::Column(identifier) => self.visit_column(*identifier),
            Expression::Literal(lit) => self.visit_literal(lit),
            Expression::Binary { op, left, right } => self.visit_binary_expr(*op, left, right),
            Expression::Unary { op, expr } => self.visit_unary_expr(*op, expr),
            _ => Err(PostprocessingError::Unsupported(format!(
                "Expression {:?} is not supported yet",
                expr
            ))),
        }
    }

    fn visit_column(&self, identifier: Identifier) -> PostprocessingResult<OwnedColumn<S>> {
        Ok(self
            .table
            .inner_table()
            .get(&identifier)
            .ok_or(PostprocessingError::ColumnNotFound(identifier.to_string()))?
            .clone())
    }

    fn visit_literal(&self, lit: &Literal) -> PostprocessingResult<OwnedColumn<S>> {
        let len = self.table.num_rows();
        match lit {
            Literal::Boolean(b) => Ok(OwnedColumn::Boolean(vec![*b; len])),
            Literal::BigInt(i) => Ok(OwnedColumn::BigInt(vec![*i; len])),
            Literal::Int128(i) => Ok(OwnedColumn::Int128(vec![*i; len])),
            Literal::Decimal(d) => {
                let scale = d.scale();
                let precision = Precision::new(d.precision())?;
                let scalar = try_into_to_scalar(d, precision, scale)?;
                Ok(OwnedColumn::Decimal75(precision, scale, vec![scalar; len]))
            }
            Literal::VarChar(s) => Ok(OwnedColumn::VarChar(vec![s.clone(); len])),
            Literal::Timestamp(its) => Ok(OwnedColumn::TimestampTZ(
                its.timeunit,
                its.timezone,
                vec![its.timestamp.timestamp(); len],
            )),
        }
    }

    fn visit_unary_expr(
        &self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> PostprocessingResult<OwnedColumn<S>> {
        let column = self.visit_expr(expr)?;
        match op {
            UnaryOperator::Not => {
                if column.column_type() == ColumnType::Boolean {
                    // We can unwrap here because we know the column is boolean
                    Ok(OwnedColumn::Boolean(
                        column.as_boolean().unwrap().iter().map(|b| !b).collect(),
                    ))
                } else {
                    Err(PostprocessingError::UnaryOperationInvalidColumnType {
                        operator: UnaryOperator::Not,
                        operand_type: column.column_type(),
                    })
                }
            }
        }
    }

    fn visit_binary_expr(
        &self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> PostprocessingResult<OwnedColumn<S>> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;
        let length = left.len();
        assert_eq!(length, right.len());
        check_dtypes(left.column_type(), right.column_type(), op)?;
        match op {
            BinaryOperator::And => Ok(OwnedColumn::Boolean(
                left.as_boolean()
                    .unwrap()
                    .iter()
                    .zip(right.as_boolean().unwrap())
                    .map(|(l, r)| *l && *r)
                    .collect(),
            )),
            BinaryOperator::Or => Ok(OwnedColumn::Boolean(
                left.as_boolean()
                    .unwrap()
                    .iter()
                    .zip(right.as_boolean().unwrap())
                    .map(|(l, r)| *l || *r)
                    .collect(),
            )),
            BinaryOperator::Equal
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::LessThanOrEqual => {
                let lhs_scale = left.column_type().scale().unwrap_or(0);
                let rhs_scale = right.column_type().scale().unwrap_or(0);
                let max_scale = lhs_scale.max(rhs_scale);
                let lhs_upscale_factor =
                    scale_scalar(S::ONE, max_scale - lhs_scale).expect("Invalid scale factor");
                let rhs_upscale_factor =
                    scale_scalar(S::ONE, max_scale - rhs_scale).expect("Invalid scale factor");
                match op {
                    BinaryOperator::Equal => {
                        let res: Vec<bool> = (0..length)
                            .map(|i| {
                                left.scalar_at(i).unwrap() * lhs_upscale_factor
                                    == right.scalar_at(i).unwrap() * rhs_upscale_factor
                            })
                            .collect();
                        Ok(OwnedColumn::Boolean(res))
                    }
                    BinaryOperator::GreaterThanOrEqual => {
                        let res: Vec<bool> = (0..length)
                            .map(|i| {
                                let lhs = left.scalar_at(i).unwrap() * lhs_upscale_factor;
                                let rhs = right.scalar_at(i).unwrap() * rhs_upscale_factor;
                                lhs.signed_cmp(&rhs) != Ordering::Less
                            })
                            .collect();
                        Ok(OwnedColumn::Boolean(res))
                    }
                    BinaryOperator::LessThanOrEqual => {
                        let res: Vec<bool> = (0..length)
                            .map(|i| {
                                let lhs = left.scalar_at(i).unwrap() * lhs_upscale_factor;
                                let rhs = right.scalar_at(i).unwrap() * rhs_upscale_factor;
                                lhs.signed_cmp(&rhs) != Ordering::Greater
                            })
                            .collect();
                        Ok(OwnedColumn::Boolean(res))
                    }
                    _ => unreachable!(),
                }
            }
            BinaryOperator::Add => Ok(add_subtract_owned_columns(&left, &right, false)),
            BinaryOperator::Subtract => Ok(add_subtract_owned_columns(&left, &right, true)),
            BinaryOperator::Multiply => Ok(multiply_owned_columns(&left, &right)),
            BinaryOperator::Division => try_divide_owned_columns(&left, &right),
        }
    }
}

/// If the two column types are compatible for a binary operation, return the result type.
/// Otherwise, return None.
pub(crate) fn get_binary_operation_result_type(
    left_dtype: &ColumnType,
    right_dtype: &ColumnType,
    binary_operator: BinaryOperator,
) -> Option<ColumnType> {
    match binary_operator {
        BinaryOperator::And | BinaryOperator::Or => matches!(
            (left_dtype, right_dtype),
            (ColumnType::Boolean, ColumnType::Boolean)
        )
        .then_some(ColumnType::Boolean),
        BinaryOperator::Equal => (matches!(
            (left_dtype, right_dtype),
            (ColumnType::VarChar, ColumnType::VarChar)
                | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                | (ColumnType::Boolean, ColumnType::Boolean)
                | (_, ColumnType::Scalar)
                | (ColumnType::Scalar, _)
        ) || (left_dtype.is_numeric() && right_dtype.is_numeric()))
        .then_some(ColumnType::Boolean),
        BinaryOperator::GreaterThanOrEqual | BinaryOperator::LessThanOrEqual => {
            if left_dtype == &ColumnType::VarChar || right_dtype == &ColumnType::VarChar {
                return None;
            }
            // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
            if let ColumnType::Decimal75(precision, _) = left_dtype {
                if precision.value() > 38 {
                    return None;
                }
            }
            if let ColumnType::Decimal75(precision, _) = right_dtype {
                if precision.value() > 38 {
                    return None;
                }
            }
            // TODO: inequality support for timestamps
            (left_dtype.is_numeric() && right_dtype.is_numeric()
                || matches!(
                    (left_dtype, right_dtype),
                    (ColumnType::Boolean, ColumnType::Boolean)
                        | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                ))
            .then_some(ColumnType::Boolean)
        }
        BinaryOperator::Add | BinaryOperator::Subtract => {
            try_add_subtract_column_types(*left_dtype, *right_dtype).ok()
        }
        BinaryOperator::Multiply => try_multiply_column_types(*left_dtype, *right_dtype).ok(),
        BinaryOperator::Division => try_divide_column_types(*left_dtype, *right_dtype).ok(),
    }
}

fn check_dtypes(
    left_dtype: ColumnType,
    right_dtype: ColumnType,
    binary_operator: BinaryOperator,
) -> PostprocessingResult<()> {
    get_binary_operation_result_type(&left_dtype, &right_dtype, binary_operator).ok_or(
        PostprocessingError::BinaryOperationInvalidColumnType {
            operator: binary_operator,
            left_type: left_dtype,
            right_type: right_dtype,
        },
    )?;
    Ok(())
}
