use super::{
    AddSubtractExpr, AggregateExpr, AndExpr, ColumnExpr, EqualsExpr, InequalityExpr, LiteralExpr,
    MultiplyExpr, NotExpr, OrExpr, ProvableExpr,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, LiteralValue},
        proof::ProofError,
    },
    sql::{
        parse::{type_check_binary_operation, ConversionError, ConversionResult},
        proof::{CountBuilder, ProofBuilder, VerificationBuilder},
    },
};
use bumpalo::Bump;
use proof_of_sql_parser::intermediate_ast::{AggregationOperator, BinaryOperator};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug};

/// Enum of AST column expression types that implement `ProvableExpr`. Is itself a `ProvableExpr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProvableExprPlan<C: Commitment> {
    /// Column
    Column(ColumnExpr<C>),
    /// Provable logical AND expression
    And(AndExpr<C>),
    /// Provable logical OR expression
    Or(OrExpr<C>),
    /// Provable logical NOT expression
    Not(NotExpr<C>),
    /// Provable CONST expression
    Literal(LiteralExpr<C::Scalar>),
    /// Provable AST expression for an equals expression
    Equals(EqualsExpr<C>),
    /// Provable AST expression for an inequality expression
    Inequality(InequalityExpr<C>),
    /// Provable numeric `+` / `-` expression
    AddSubtract(AddSubtractExpr<C>),
    /// Provable numeric `*` expression
    Multiply(MultiplyExpr<C>),
    /// Provable aggregate expression
    Aggregate(AggregateExpr<C>),
}
impl<C: Commitment> ProvableExprPlan<C> {
    /// Create column expression
    pub fn new_column(column_ref: ColumnRef) -> Self {
        Self::Column(ColumnExpr::new(column_ref))
    }
    /// Create logical AND expression
    pub fn try_new_and(
        lhs: ProvableExprPlan<C>,
        rhs: ProvableExprPlan<C>,
    ) -> ConversionResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::And(AndExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical OR expression
    pub fn try_new_or(
        lhs: ProvableExprPlan<C>,
        rhs: ProvableExprPlan<C>,
    ) -> ConversionResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Or(OrExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical NOT expression
    pub fn try_new_not(expr: ProvableExprPlan<C>) -> ConversionResult<Self> {
        expr.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Not(NotExpr::new(Box::new(expr))))
    }
    /// Create CONST expression
    pub fn new_literal(value: LiteralValue<C::Scalar>) -> Self {
        Self::Literal(LiteralExpr::new(value))
    }
    /// Create a new equals expression
    pub fn try_new_equals(
        lhs: ProvableExprPlan<C>,
        rhs: ProvableExprPlan<C>,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if !type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Equal) {
            Err(ConversionError::DataTypeMismatch(
                lhs_datatype.to_string(),
                rhs_datatype.to_string(),
            ))
        } else {
            Ok(Self::Equals(EqualsExpr::new(Box::new(lhs), Box::new(rhs))))
        }
    }
    /// Create a new inequality expression
    pub fn try_new_inequality(
        lhs: ProvableExprPlan<C>,
        rhs: ProvableExprPlan<C>,
        is_lte: bool,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if !type_check_binary_operation(
            &lhs_datatype,
            &rhs_datatype,
            BinaryOperator::LessThanOrEqual,
        ) {
            Err(ConversionError::DataTypeMismatch(
                lhs_datatype.to_string(),
                rhs_datatype.to_string(),
            ))
        } else {
            Ok(Self::Inequality(InequalityExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                is_lte,
            )))
        }
    }

    /// Create a new add expression
    pub fn try_new_add(
        lhs: ProvableExprPlan<C>,
        rhs: ProvableExprPlan<C>,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if !type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Add) {
            Err(ConversionError::DataTypeMismatch(
                lhs_datatype.to_string(),
                rhs_datatype.to_string(),
            ))
        } else {
            Ok(Self::AddSubtract(AddSubtractExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                false,
            )))
        }
    }

    /// Create a new subtract expression
    pub fn try_new_subtract(
        lhs: ProvableExprPlan<C>,
        rhs: ProvableExprPlan<C>,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if !type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Subtract) {
            Err(ConversionError::DataTypeMismatch(
                lhs_datatype.to_string(),
                rhs_datatype.to_string(),
            ))
        } else {
            Ok(Self::AddSubtract(AddSubtractExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                true,
            )))
        }
    }

    /// Create a new multiply expression
    pub fn try_new_multiply(
        lhs: ProvableExprPlan<C>,
        rhs: ProvableExprPlan<C>,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if !type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Multiply) {
            Err(ConversionError::DataTypeMismatch(
                lhs_datatype.to_string(),
                rhs_datatype.to_string(),
            ))
        } else {
            Ok(Self::Multiply(MultiplyExpr::new(
                Box::new(lhs),
                Box::new(rhs),
            )))
        }
    }

    /// Create a new aggregate expression
    pub fn new_aggregate(op: AggregationOperator, expr: ProvableExprPlan<C>) -> Self {
        Self::Aggregate(AggregateExpr::new(op, Box::new(expr)))
    }

    /// Check that the plan has the correct data type
    fn check_data_type(&self, data_type: ColumnType) -> ConversionResult<()> {
        if self.data_type() == data_type {
            Ok(())
        } else {
            Err(ConversionError::InvalidDataType {
                actual: self.data_type(),
                expected: data_type,
            })
        }
    }
}

impl<C: Commitment> ProvableExpr<C> for ProvableExprPlan<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        match self {
            ProvableExprPlan::Column(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::And(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::Or(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::Not(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::Literal(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::Equals(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::Inequality(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::AddSubtract(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::Multiply(expr) => ProvableExpr::<C>::count(expr, builder),
            ProvableExprPlan::Aggregate(expr) => ProvableExpr::<C>::count(expr, builder),
        }
    }

    fn data_type(&self) -> ColumnType {
        match self {
            ProvableExprPlan::Column(expr) => expr.data_type(),
            ProvableExprPlan::AddSubtract(expr) => expr.data_type(),
            ProvableExprPlan::Multiply(expr) => expr.data_type(),
            ProvableExprPlan::Aggregate(expr) => expr.data_type(),
            ProvableExprPlan::Literal(expr) => ProvableExpr::<C>::data_type(expr),
            ProvableExprPlan::And(_)
            | ProvableExprPlan::Or(_)
            | ProvableExprPlan::Not(_)
            | ProvableExprPlan::Equals(_)
            | ProvableExprPlan::Inequality(_) => ColumnType::Boolean,
        }
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        match self {
            ProvableExprPlan::Column(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::And(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Or(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Not(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Literal(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Equals(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Inequality(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::AddSubtract(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Multiply(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Aggregate(expr) => {
                ProvableExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        match self {
            ProvableExprPlan::Column(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::And(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Or(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Not(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Literal(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Equals(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Inequality(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::AddSubtract(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Multiply(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Aggregate(expr) => {
                ProvableExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        match self {
            ProvableExprPlan::Column(expr) => {
                ProvableExpr::<C>::verifier_evaluate(expr, builder, accessor)
            }
            ProvableExprPlan::And(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Or(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Not(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Literal(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Equals(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Inequality(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::AddSubtract(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Multiply(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Aggregate(expr) => expr.verifier_evaluate(builder, accessor),
        }
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        match self {
            ProvableExprPlan::Column(expr) => {
                ProvableExpr::<C>::get_column_references(expr, columns)
            }
            ProvableExprPlan::And(expr) => ProvableExpr::<C>::get_column_references(expr, columns),
            ProvableExprPlan::Or(expr) => ProvableExpr::<C>::get_column_references(expr, columns),
            ProvableExprPlan::Not(expr) => ProvableExpr::<C>::get_column_references(expr, columns),
            ProvableExprPlan::Literal(expr) => {
                ProvableExpr::<C>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Equals(expr) => {
                ProvableExpr::<C>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Inequality(expr) => {
                ProvableExpr::<C>::get_column_references(expr, columns)
            }
            ProvableExprPlan::AddSubtract(expr) => {
                ProvableExpr::<C>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Multiply(expr) => {
                ProvableExpr::<C>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Aggregate(expr) => {
                ProvableExpr::<C>::get_column_references(expr, columns)
            }
        }
    }
}
