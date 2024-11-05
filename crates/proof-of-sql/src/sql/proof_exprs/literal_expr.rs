use super::ProofExpr;
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnRef, ColumnType, ColumnarValue, CommitmentAccessor, DataAccessor,
            LiteralValue,
        },
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
pub struct LiteralExpr {
    value: LiteralValue,
}

impl LiteralExpr {
    /// Create literal expression
    pub fn new(value: LiteralValue) -> Self {
        Self { value }
    }
}

impl ProofExpr for LiteralExpr {
    fn count(&self, _builder: &mut CountBuilder) -> Result<(), ProofError> {
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        self.value.column_type()
    }

    #[tracing::instrument(name = "LiteralExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> ColumnarValue<'a, S> {
        ColumnarValue::Literal(self.value.clone())
    }

    #[tracing::instrument(name = "LiteralExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        let table_length = builder.table_length();
        Column::from_literal_with_length(&self.value, table_length, alloc)
    }

    fn verifier_evaluate<C: Commitment>(
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
