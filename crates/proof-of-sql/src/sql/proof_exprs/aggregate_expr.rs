use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        map::IndexSet,
        proof::ProofError,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use proof_of_sql_parser::intermediate_ast::AggregationOperator;
use serde::{Deserialize, Serialize};

/// Provable aggregate expression
///
/// Currently it doesn't do much since aggregation logic is implemented elsewhere
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AggregateExpr<C: Commitment> {
    op: AggregationOperator,
    expr: Box<DynProofExpr<C>>,
}

impl<C: Commitment> AggregateExpr<C> {
    /// Create a new aggregate expression
    pub fn new(op: AggregationOperator, expr: Box<DynProofExpr<C>>) -> Self {
        Self { op, expr }
    }
}

impl<C: Commitment> ProofExpr<C> for AggregateExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.expr.count(builder)
    }

    fn data_type(&self) -> ColumnType {
        match self.op {
            AggregationOperator::Count => ColumnType::BigInt,
            AggregationOperator::Sum => self.expr.data_type(),
            _ => todo!("Aggregation operator not supported here yet"),
        }
    }

    #[tracing::instrument(name = "AggregateExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        self.expr.result_evaluate(table_length, alloc, accessor)
    }

    #[tracing::instrument(name = "AggregateExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        self.expr.prover_evaluate(builder, alloc, accessor)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        self.expr.verifier_evaluate(builder, accessor)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
