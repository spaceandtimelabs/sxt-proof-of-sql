use super::{AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, InequalityExpr, NotExpr, OrExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug};

/// Enum of AST column expression types that implement `BoolExpr`. Is itself a `BoolExpr`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum BoolExprPlan<C: Commitment> {
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
impl<C: Commitment> BoolExprPlan<C> {
    /// Create logical AND expression
    pub fn new_and(lhs: BoolExprPlan<C>, rhs: BoolExprPlan<C>) -> Self {
        Self::And(AndExpr::new(Box::new(lhs), Box::new(rhs)))
    }
    /// Create logical OR expression
    pub fn new_or(lhs: BoolExprPlan<C>, rhs: BoolExprPlan<C>) -> Self {
        Self::Or(OrExpr::new(Box::new(lhs), Box::new(rhs)))
    }
    /// Create logical NOT expression
    pub fn new_not(expr: BoolExprPlan<C>) -> Self {
        Self::Not(NotExpr::new(Box::new(expr)))
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
}

impl<C: Commitment> BoolExpr<C> for BoolExprPlan<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        match self {
            BoolExprPlan::And(expr) => BoolExpr::<C>::count(expr, builder),
            BoolExprPlan::Or(expr) => BoolExpr::<C>::count(expr, builder),
            BoolExprPlan::Not(expr) => BoolExpr::<C>::count(expr, builder),
            BoolExprPlan::ConstBool(expr) => BoolExpr::<C>::count(expr, builder),
            BoolExprPlan::Equals(expr) => BoolExpr::<C>::count(expr, builder),
            BoolExprPlan::Inequality(expr) => BoolExpr::<C>::count(expr, builder),
        }
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [bool] {
        match self {
            BoolExprPlan::And(expr) => {
                BoolExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            BoolExprPlan::Or(expr) => {
                BoolExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            BoolExprPlan::Not(expr) => {
                BoolExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            BoolExprPlan::ConstBool(expr) => {
                BoolExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            BoolExprPlan::Equals(expr) => {
                BoolExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            BoolExprPlan::Inequality(expr) => {
                BoolExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
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
            BoolExprPlan::And(expr) => {
                BoolExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            BoolExprPlan::Or(expr) => {
                BoolExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            BoolExprPlan::Not(expr) => {
                BoolExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            BoolExprPlan::ConstBool(expr) => {
                BoolExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            BoolExprPlan::Equals(expr) => {
                BoolExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            BoolExprPlan::Inequality(expr) => {
                BoolExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        match self {
            BoolExprPlan::And(expr) => expr.verifier_evaluate(builder, accessor),
            BoolExprPlan::Or(expr) => expr.verifier_evaluate(builder, accessor),
            BoolExprPlan::Not(expr) => expr.verifier_evaluate(builder, accessor),
            BoolExprPlan::ConstBool(expr) => expr.verifier_evaluate(builder, accessor),
            BoolExprPlan::Equals(expr) => expr.verifier_evaluate(builder, accessor),
            BoolExprPlan::Inequality(expr) => expr.verifier_evaluate(builder, accessor),
        }
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        match self {
            BoolExprPlan::And(expr) => BoolExpr::<C>::get_column_references(expr, columns),
            BoolExprPlan::Or(expr) => BoolExpr::<C>::get_column_references(expr, columns),
            BoolExprPlan::Not(expr) => BoolExpr::<C>::get_column_references(expr, columns),
            BoolExprPlan::ConstBool(expr) => BoolExpr::<C>::get_column_references(expr, columns),
            BoolExprPlan::Equals(expr) => BoolExpr::<C>::get_column_references(expr, columns),
            BoolExprPlan::Inequality(expr) => BoolExpr::<C>::get_column_references(expr, columns),
        }
    }
}
