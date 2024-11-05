use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnRef, ColumnType, ColumnarValue, CommitmentAccessor, DataAccessor,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
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
pub struct AggregateExpr {
    op: AggregationOperator,
    expr: Box<DynProofExpr>,
}

impl AggregateExpr {
    /// Create a new aggregate expression
    pub fn new(op: AggregationOperator, expr: Box<DynProofExpr>) -> Self {
        Self { op, expr }
    }
}

impl ProofExpr for AggregateExpr {
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
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> ColumnarValue<'a, S> {
        self.expr.result_evaluate(alloc, accessor)
    }

    #[tracing::instrument(name = "AggregateExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        self.expr.prover_evaluate(builder, alloc, accessor)
    }

    fn verifier_evaluate<C: Commitment>(
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
