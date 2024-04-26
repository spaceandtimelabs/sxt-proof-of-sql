use super::ProvableExpr;
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, marker::PhantomData};

/// Provable logical NOT expression
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NotExpr<C: Commitment, B: ProvableExpr<C, bool>> {
    expr: Box<B>,
    _phantom: PhantomData<C>,
}

impl<C: Commitment, B: ProvableExpr<C, bool>> NotExpr<C, B> {
    /// Create logical NOT expression
    pub fn new(expr: Box<B>) -> Self {
        Self {
            expr,
            _phantom: PhantomData,
        }
    }
}

impl<C: Commitment, B: ProvableExpr<C, bool>> ProvableExpr<C, bool> for NotExpr<C, B> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.expr.count(builder)
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
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
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [bool] {
        let selection = self.expr.prover_evaluate(builder, alloc, accessor);
        alloc.alloc_slice_fill_with(selection.len(), |i| !selection[i])
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let eval = self.expr.verifier_evaluate(builder, accessor)?;
        Ok(builder.mle_evaluations.one_evaluation - eval)
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
