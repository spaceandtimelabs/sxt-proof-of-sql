use super::{AndExpr, ConstBoolExpr, EqualsExpr, InequalityExpr, NotExpr, OrExpr, ProvableExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        proof::ProofError,
    },
    sql::{
        parse::{ConversionError, ConversionResult},
        proof::{CountBuilder, ProofBuilder, VerificationBuilder},
    },
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug};

/// Enum of AST column expression types that implement `ProvableExpr`. Is itself a `ProvableExpr`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ProvableExprPlan<C: Commitment> {
    /// Provable logical AND expression
    And(AndExpr<C, Self>),
    /// Provable logical OR expression
    Or(OrExpr<C, Self>),
    /// Provable logical NOT expression
    Not(NotExpr<C, Self>),
    /// Provable logical CONST expression
    ConstBool(ConstBoolExpr),
    /// Provable AST expression for an equals expression
    Equals(EqualsExpr<C::Scalar>),
    /// Provable AST expression for an inequality expression
    Inequality(InequalityExpr<C::Scalar>),
}
impl<C: Commitment> ProvableExprPlan<C> {
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
    /// Create logical CONST expression
    pub fn new_const_bool(value: bool) -> Self {
        Self::ConstBool(ConstBoolExpr::new(value))
    }
    /// Create a new equals expression
    pub fn new_equals(column_ref: ColumnRef, value: C::Scalar) -> Self {
        Self::Equals(EqualsExpr::new(column_ref, value))
    }
    /// Create a new inequality expression
    pub fn new_inequality(column_ref: ColumnRef, value: C::Scalar, is_lte: bool) -> Self {
        Self::Inequality(InequalityExpr::new(column_ref, value, is_lte))
    }

    /// Check that the plan has the correct data type
    pub fn check_data_type(&self, data_type: ColumnType) -> ConversionResult<()> {
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

impl<C: Commitment> ProvableExpr<C, bool> for ProvableExprPlan<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        match self {
            ProvableExprPlan::And(expr) => ProvableExpr::<C, bool>::count(expr, builder),
            ProvableExprPlan::Or(expr) => ProvableExpr::<C, bool>::count(expr, builder),
            ProvableExprPlan::Not(expr) => ProvableExpr::<C, bool>::count(expr, builder),
            ProvableExprPlan::ConstBool(expr) => ProvableExpr::<C, bool>::count(expr, builder),
            ProvableExprPlan::Equals(expr) => ProvableExpr::<C, bool>::count(expr, builder),
            ProvableExprPlan::Inequality(expr) => ProvableExpr::<C, bool>::count(expr, builder),
        }
    }

    /// When `ProvableExprPlan` becomes generic over the column type, this will need to be updated
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [bool] {
        match self {
            ProvableExprPlan::And(expr) => {
                ProvableExpr::<C, bool>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Or(expr) => {
                ProvableExpr::<C, bool>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Not(expr) => {
                ProvableExpr::<C, bool>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::ConstBool(expr) => {
                ProvableExpr::<C, bool>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Equals(expr) => {
                ProvableExpr::<C, bool>::result_evaluate(expr, table_length, alloc, accessor)
            }
            ProvableExprPlan::Inequality(expr) => {
                ProvableExpr::<C, bool>::result_evaluate(expr, table_length, alloc, accessor)
            }
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [bool] {
        match self {
            ProvableExprPlan::And(expr) => {
                ProvableExpr::<C, bool>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Or(expr) => {
                ProvableExpr::<C, bool>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Not(expr) => {
                ProvableExpr::<C, bool>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::ConstBool(expr) => {
                ProvableExpr::<C, bool>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Equals(expr) => {
                ProvableExpr::<C, bool>::prover_evaluate(expr, builder, alloc, accessor)
            }
            ProvableExprPlan::Inequality(expr) => {
                ProvableExpr::<C, bool>::prover_evaluate(expr, builder, alloc, accessor)
            }
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        match self {
            ProvableExprPlan::And(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Or(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Not(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::ConstBool(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Equals(expr) => expr.verifier_evaluate(builder, accessor),
            ProvableExprPlan::Inequality(expr) => expr.verifier_evaluate(builder, accessor),
        }
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        match self {
            ProvableExprPlan::And(expr) => {
                ProvableExpr::<C, bool>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Or(expr) => {
                ProvableExpr::<C, bool>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Not(expr) => {
                ProvableExpr::<C, bool>::get_column_references(expr, columns)
            }
            ProvableExprPlan::ConstBool(expr) => {
                ProvableExpr::<C, bool>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Equals(expr) => {
                ProvableExpr::<C, bool>::get_column_references(expr, columns)
            }
            ProvableExprPlan::Inequality(expr) => {
                ProvableExpr::<C, bool>::get_column_references(expr, columns)
            }
        }
    }
}
