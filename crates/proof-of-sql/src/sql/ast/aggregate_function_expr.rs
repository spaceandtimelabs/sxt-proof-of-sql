use super::ProvableExprPlan;
use crate::base::commitment::Commitment;
use proof_of_sql_parser::intermediate_ast::AggregationOperator;

use super::{ProvableExpr, ProvableExprPlan};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable aggregate function expression
///
/// Currently it doesn't do much since aggregation logic is implemented elsewhere
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AggregateFunctionExpr<C: Commitment> {
    op: AggregationOperator,
    expr: Box<ProvableExprPlan<C>>,
}

impl<C: Commitment> AggregateFunctionExpr<C> {
    /// Create a new aggregate function expression
    pub fn new(op: AggregationOperator, expr: Box<ProvableExprPlan<C>>) -> Self {
        Self { op, exprs }
    }
}

impl<C: Commitment> ProvableExpr<C> for NotExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        match self.op {
            AggregationOperator::Count => ColumnType::BigInt,
            AggregationOperator::Sum => self.expr.data_type(),
            _ => todo!("Aggregation operator not supported here yet"),
        }
    }

    #[tracing::instrument(name = "AggregateFunctionExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        self.expr.result_evaluate(table_length, alloc, accessor)
    }

    #[tracing::instrument(name = "AggregateFunctionExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
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

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.expr.get_column_references(columns)
    }
}
