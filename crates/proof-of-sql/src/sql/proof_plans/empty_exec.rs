use crate::{
    base::{
        database::{ColumnField, ColumnRef, DataAccessor, OwnedTable, Table, TableRef},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
        VerificationBuilder,
    },
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Source [`ProofPlan`] for (sub)queries without table source such as `SELECT "No table here" as msg;`
/// Inspired by [`DataFusion EmptyExec`](https://docs.rs/datafusion/latest/datafusion/physical_plan/empty/struct.EmptyExec.html)
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EmptyExec {}

impl Default for EmptyExec {
    fn default() -> Self {
        Self::new()
    }
}

impl EmptyExec {
    /// Creates a new empty plan.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ProofPlan for EmptyExec {
    fn count(&self, _builder: &mut CountBuilder) -> Result<(), ProofError> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
    ) -> Result<Vec<S>, ProofError> {
        Ok(Vec::new())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        Vec::new()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        IndexSet::default()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::default()
    }
}

impl ProverEvaluate for EmptyExec {
    #[tracing::instrument(name = "EmptyExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Table<'a, S> {
        Table::<'a, S>::try_new(IndexMap::default()).unwrap()
    }

    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}

    #[tracing::instrument(name = "EmptyExec::final_round_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Table<'a, S> {
        Table::<'a, S>::try_new(IndexMap::default()).unwrap()
    }
}
