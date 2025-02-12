use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
    utils::log,
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
    // Remove the count method
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
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let res = self.expr.result_evaluate(alloc, table);

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "AggregateExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let res = self.expr.prover_evaluate(builder, alloc, table);

        log::log_memory_usage("End");

        res
    }

    fn verifier_evaluate<S: Scalar, B: VerificationBuilder<S>>(
        &self,
        builder: &mut B,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        self.expr.verifier_evaluate(builder, accessor, chi_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
