use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::ast::{BoolExpr, TableExpr};
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::DynPartialEq;

/// Provable logical NOT expression
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct NotExpr {
    expr: Box<dyn BoolExpr>,
}

impl NotExpr {
    /// Create logical NOT expression
    pub fn new(expr: Box<dyn BoolExpr>) -> Self {
        Self { expr }
    }
}

impl BoolExpr for NotExpr {
    fn count(&self, counts: &mut ProofCounts) {
        self.expr.count(counts);
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        table: &TableExpr,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        let selection = self
            .expr
            .prover_evaluate(builder, alloc, table, counts, accessor);
        alloc.alloc_slice_fill_with(selection.len(), |i| !selection[i])
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        table: &TableExpr,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) -> Scalar {
        let eval = self
            .expr
            .verifier_evaluate(builder, table, counts, accessor);
        builder.mle_evaluations.one_evaluation - eval
    }
}
