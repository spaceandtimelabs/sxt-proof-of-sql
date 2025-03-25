use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table},
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
    ) -> NullableColumn<'a, S> {
        log::log_memory_usage("Start");

        let expr_column = self.expr.result_evaluate(alloc, table).values;
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        NullableColumn::new(res)
    }

    #[tracing::instrument(name = "NotExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> NullableColumn<'a, S> {
        log::log_memory_usage("Start");

        let expr_column = self.expr.prover_evaluate(builder, alloc, table).values;
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        NullableColumn::new(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<(S, Option<S>), ProofError> {
        let (eval, _) = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;
        Ok((chi_eval - eval, None))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
