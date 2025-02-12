use super::ProofExpr;
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
    utils::log,
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
    pub(crate) value: LiteralValue,
}

impl LiteralExpr {
    /// Create literal expression
    pub fn new(value: LiteralValue) -> Self {
        Self { value }
    }
}

impl ProofExpr for LiteralExpr {
    fn data_type(&self) -> ColumnType {
        self.value.column_type()
    }

    #[tracing::instrument(name = "LiteralExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let res = Column::from_literal_with_length(&self.value, table.num_rows(), alloc);

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "LiteralExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let table_length = table.num_rows();
        let res = Column::from_literal_with_length(&self.value, table_length, alloc);

        log::log_memory_usage("End");

        res
    }

    fn verifier_evaluate<S: Scalar, B: VerificationBuilder<S>>(
        &self,
        _builder: &mut B,
        _accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        Ok(chi_eval * self.value.to_scalar())
    }

    fn get_column_references(&self, _columns: &mut IndexSet<ColumnRef>) {}
}
