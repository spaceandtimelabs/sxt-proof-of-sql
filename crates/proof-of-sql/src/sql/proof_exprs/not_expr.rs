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
use serde::{Deserialize, Serialize};

/// Provable logical NOT expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotExpr {
    expr: Box<DynProofExpr>,
}

impl NotExpr {
    /// Create logical NOT expression
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        Self { expr }
    }
}

impl ProofExpr for NotExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "NotExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let expr_column: Column<'a, S> = self.expr.result_evaluate(alloc, table);
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "NotExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let expr_column: Column<'a, S> = self.expr.prover_evaluate(builder, alloc, table);
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        res
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        one_eval: S,
    ) -> Result<S, ProofError> {
        let eval = self.expr.verifier_evaluate(builder, accessor, one_eval)?;
        Ok(one_eval - eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
