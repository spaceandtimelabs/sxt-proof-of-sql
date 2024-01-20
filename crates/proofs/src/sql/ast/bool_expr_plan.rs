use super::{AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, InequalityExpr, NotExpr, OrExpr};
use crate::{
    base::{
        database::{ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::ArkScalar,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug};

/// Enum of AST column expression types that implement `BoolExpr`. Is itself a `BoolExpr`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum BoolExprPlan {
    /// Provable logical AND expression
    And(AndExpr<Self>),
    /// Provable logical OR expression
    Or(OrExpr<Self>),
    /// Provable logical NOT expression
    Not(NotExpr<Self>),
    /// Provable logical CONST expression
    ConstBool(ConstBoolExpr),
    /// Provable AST expression for an equals expression
    Equals(EqualsExpr),
    /// Provable AST expression for an inequality expression
    Inequality(InequalityExpr),
}
impl BoolExprPlan {
    /// Create logical AND expression
    pub fn new_and(lhs: BoolExprPlan, rhs: BoolExprPlan) -> Self {
        Self::And(AndExpr::new(Box::new(lhs), Box::new(rhs)))
    }
    /// Create logical OR expression
    pub fn new_or(lhs: BoolExprPlan, rhs: BoolExprPlan) -> Self {
        Self::Or(OrExpr::new(Box::new(lhs), Box::new(rhs)))
    }
    /// Create logical NOT expression
    pub fn new_not(expr: BoolExprPlan) -> Self {
        Self::Not(NotExpr::new(Box::new(expr)))
    }
    /// Create logical CONST expression
    pub fn new_const_bool(value: bool) -> Self {
        Self::ConstBool(ConstBoolExpr::new(value))
    }
    /// Create a new equals expression
    pub fn new_equals(column_ref: ColumnRef, value: ArkScalar) -> Self {
        Self::Equals(EqualsExpr::new(column_ref, value))
    }
    /// Create a new inequality expression
    pub fn new_inequality(column_ref: ColumnRef, value: ArkScalar, is_lte: bool) -> Self {
        Self::Inequality(InequalityExpr::new(column_ref, value, is_lte))
    }
}

impl BoolExpr for BoolExprPlan {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        match self {
            BoolExprPlan::And(expr) => expr.count(builder),
            BoolExprPlan::Or(expr) => expr.count(builder),
            BoolExprPlan::Not(expr) => expr.count(builder),
            BoolExprPlan::ConstBool(expr) => expr.count(builder),
            BoolExprPlan::Equals(expr) => expr.count(builder),
            BoolExprPlan::Inequality(expr) => expr.count(builder),
        }
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        match self {
            BoolExprPlan::And(expr) => expr.result_evaluate(table_length, alloc, accessor),
            BoolExprPlan::Or(expr) => expr.result_evaluate(table_length, alloc, accessor),
            BoolExprPlan::Not(expr) => expr.result_evaluate(table_length, alloc, accessor),
            BoolExprPlan::ConstBool(expr) => expr.result_evaluate(table_length, alloc, accessor),
            BoolExprPlan::Equals(expr) => expr.result_evaluate(table_length, alloc, accessor),
            BoolExprPlan::Inequality(expr) => expr.result_evaluate(table_length, alloc, accessor),
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, ArkScalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        match self {
            BoolExprPlan::And(expr) => expr.prover_evaluate(builder, alloc, accessor),
            BoolExprPlan::Or(expr) => expr.prover_evaluate(builder, alloc, accessor),
            BoolExprPlan::Not(expr) => expr.prover_evaluate(builder, alloc, accessor),
            BoolExprPlan::ConstBool(expr) => expr.prover_evaluate(builder, alloc, accessor),
            BoolExprPlan::Equals(expr) => expr.prover_evaluate(builder, alloc, accessor),
            BoolExprPlan::Inequality(expr) => expr.prover_evaluate(builder, alloc, accessor),
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<RistrettoPoint>,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
    ) -> Result<ArkScalar, ProofError> {
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
            BoolExprPlan::And(expr) => expr.get_column_references(columns),
            BoolExprPlan::Or(expr) => expr.get_column_references(columns),
            BoolExprPlan::Not(expr) => expr.get_column_references(columns),
            BoolExprPlan::ConstBool(expr) => expr.get_column_references(columns),
            BoolExprPlan::Equals(expr) => expr.get_column_references(columns),
            BoolExprPlan::Inequality(expr) => expr.get_column_references(columns),
        }
    }
}
