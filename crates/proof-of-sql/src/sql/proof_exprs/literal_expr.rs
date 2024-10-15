use super::ProofExpr;
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, LiteralValue},
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable CONST expression
///
/// This node allows us to easily represent queries like
///    select * from T
/// and
///    select * from T where 1 = 2
/// as filter expressions with a constant where clause.
///
/// While this wouldn't be as efficient as using a new custom expression for
/// such queries, it allows us to easily support projects with minimal code
/// changes, and the performance is sufficient for present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteralExpr<S: Scalar> {
    value: LiteralValue<S>,
}

impl<S: Scalar> LiteralExpr<S> {
    /// Create literal expression
    pub fn new(value: LiteralValue<S>) -> Self {
        Self { value }
    }
}

impl<C: Commitment> ProofExpr<C> for LiteralExpr<C::Scalar> {
    fn count(&self, _builder: &mut CountBuilder) -> Result<(), ProofError> {
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        self.value.column_type()
    }

    #[tracing::instrument(name = "LiteralExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        Column::from_literal_with_length(&self.value, table_length, alloc)
    }

    #[tracing::instrument(name = "LiteralExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        table_length: usize,
        _builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        Column::from_literal_with_length(&self.value, table_length, alloc)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        _accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let mut commitment = builder.mle_evaluations.input_one_evaluation;
        commitment *= self.value.to_scalar();
        Ok(commitment)
    }

    fn get_column_references(&self, _columns: &mut IndexSet<ColumnRef>) {}
}
