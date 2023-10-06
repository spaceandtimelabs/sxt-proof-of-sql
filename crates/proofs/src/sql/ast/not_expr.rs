use crate::{
    base::{
        database::{ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::ArkScalar,
    },
    sql::{
        ast::BoolExpr,
        proof::{CountBuilder, ProofBuilder, VerificationBuilder},
    },
};
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable logical NOT expression
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct NotExpr {
    expr: Box<dyn BoolExpr>,
}

impl NotExpr {
    /// Create logical NOT expression
    pub fn new(expr: Box<dyn BoolExpr>) -> Self {
        Self { expr }
    }
}

#[typetag::serde]
impl BoolExpr for NotExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.expr.count(builder)
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
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        let selection = self.expr.prover_evaluate(builder, alloc, accessor);
        alloc.alloc_slice_fill_with(selection.len(), |i| !selection[i])
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<ArkScalar, ProofError> {
        let eval = self.expr.verifier_evaluate(builder, accessor)?;
        Ok(builder.mle_evaluations.one_evaluation - eval)
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
