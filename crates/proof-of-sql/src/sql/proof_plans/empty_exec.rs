use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
    utils::log,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Source [`ProofPlan`] for (sub)queries without table source such as `SELECT "No table here" as msg;`
/// Inspired by [`DataFusion EmptyExec`](https://docs.rs/datafusion/latest/datafusion/physical_plan/empty/struct.EmptyExec.html)
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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
    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar, B: VerificationBuilder<S>>(
        &self,
        builder: &mut B,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        Ok(TableEvaluation::new(
            Vec::<S>::new(),
            builder.singleton_chi_evaluation(),
        ))
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
    #[tracing::instrument(name = "EmptyExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        // Create an empty table with one row
        let res =
            Table::<'a, S>::try_new_with_options(IndexMap::default(), TableOptions::new(Some(1)))
                .unwrap();

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "EmptyExec::final_round_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        // Create an empty table with one row
        let res =
            Table::<'a, S>::try_new_with_options(IndexMap::default(), TableOptions::new(Some(1)))
                .unwrap();

        log::log_memory_usage("End");

        res
    }
}
