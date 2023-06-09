use crate::base::database::{ColumnRef, CommitmentAccessor, DataAccessor};
use crate::base::polynomial::ArkScalar;
use crate::sql::ast::BoolExpr;
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use std::collections::HashSet;

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

    #[tracing::instrument(
        name = "proofs.sql.ast.not_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        let selection = self.expr.prover_evaluate(builder, alloc, counts, accessor);
        alloc.alloc_slice_fill_with(selection.len(), |i| !selection[i])
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) -> ArkScalar {
        let eval = self.expr.verifier_evaluate(builder, counts, accessor);
        builder.mle_evaluations.one_evaluation - eval
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
