use super::ProvableExpr;
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, LiteralValue},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteralExpr<S: Scalar> {
    value: LiteralValue<S>,
}

impl<S: Scalar> LiteralExpr<S> {
    /// Create literal expression
    pub fn new(value: LiteralValue<S>) -> Self {
        Self { value }
    }
}

impl<C: Commitment> ProvableExpr<C> for LiteralExpr<C::Scalar> {
    fn count(&self, _builder: &mut CountBuilder) -> Result<(), ProofError> {
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        self.value.column_type()
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        match self.value {
            LiteralValue::Boolean(value) => {
                Column::Boolean(alloc.alloc_slice_fill_copy(table_length, value))
            }
            LiteralValue::BigInt(value) => {
                Column::BigInt(alloc.alloc_slice_fill_copy(table_length, value))
            }
            LiteralValue::Int128(value) => {
                Column::Int128(alloc.alloc_slice_fill_copy(table_length, value))
            }
            _ => todo!(),
        }
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.literal_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        match self.value.clone() {
            LiteralValue::Boolean(value) => {
                Column::Boolean(alloc.alloc_slice_fill_copy(builder.table_length(), value))
            }
            LiteralValue::BigInt(value) => {
                Column::BigInt(alloc.alloc_slice_fill_copy(builder.table_length(), value))
            }
            LiteralValue::Int128(value) => {
                Column::Int128(alloc.alloc_slice_fill_copy(builder.table_length(), value))
            }
            _ => todo!(),
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        _accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let mut commitment = builder.mle_evaluations.one_evaluation;
        commitment *= self.value.to_scalar();
        Ok(commitment)
    }

    fn get_column_references(&self, _columns: &mut HashSet<ColumnRef>) {}
}
