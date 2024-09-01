use super::{DynProofExpr, ProvableExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

/// Provable logical NOT expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotExpr<C: Commitment> {
    expr: Box<DynProofExpr<C>>,
}

impl<C: Commitment> NotExpr<C> {
    /// Create logical NOT expression
    pub fn new(expr: Box<DynProofExpr<C>>) -> Self {
        Self { expr }
    }
}

impl<C: Commitment> ProvableExpr<C> for NotExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.expr.count(builder)
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "NotExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let expr_column: Column<'a, C::Scalar> =
            self.expr.result_evaluate(table_length, alloc, accessor);
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]))
    }

    #[tracing::instrument(name = "NotExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let expr_column: Column<'a, C::Scalar> =
            self.expr.prover_evaluate(builder, alloc, accessor);
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]))
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let eval = self.expr.verifier_evaluate(builder, accessor)?;
        Ok(builder.mle_evaluations.one_evaluation - eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
