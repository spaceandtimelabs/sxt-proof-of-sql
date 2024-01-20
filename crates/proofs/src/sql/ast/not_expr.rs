use super::BoolExpr;
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
use std::collections::HashSet;

/// Provable logical NOT expression
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NotExpr<B: BoolExpr> {
    expr: Box<B>,
}

impl<B: BoolExpr> NotExpr<B> {
    /// Create logical NOT expression
    pub fn new(expr: Box<B>) -> Self {
        Self { expr }
    }
}

impl<B: BoolExpr> BoolExpr for NotExpr<B> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.expr.count(builder)
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        let selection = self.expr.result_evaluate(table_length, alloc, accessor);
        alloc.alloc_slice_fill_with(selection.len(), |i| !selection[i])
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.not_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, ArkScalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        let selection = self.expr.prover_evaluate(builder, alloc, accessor);
        alloc.alloc_slice_fill_with(selection.len(), |i| !selection[i])
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<RistrettoPoint>,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
    ) -> Result<ArkScalar, ProofError> {
        let eval = self.expr.verifier_evaluate(builder, accessor)?;
        Ok(builder.mle_evaluations.one_evaluation - eval)
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
