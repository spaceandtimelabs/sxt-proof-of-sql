use crate::{
    base::{
        database::{Column, ColumnField, ColumnRef, DataAccessor, OwnedTable, TableRef},
        map::{indexset, IndexMap, IndexSet},
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
use core::iter::repeat_with;
use serde::{Deserialize, Serialize};

/// Source [`ProofPlan`] for (sub)queries with table source such as `SELECT col from tab;`
/// Inspired by `DataFusion` data source [`ExecutionPlan`]s such as [`ArrowExec`] and [`CsvExec`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TableExec {
    /// Table reference
    pub table_ref: TableRef,
    /// Schema of the table
    pub schema: Vec<ColumnField>,
}

impl TableExec {
    /// Creates a new [`TableExec`].
    #[must_use]
    pub fn new(table_ref: TableRef, schema: Vec<ColumnField>) -> Self {
        Self { table_ref, schema }
    }

    /// Returns the entire table.
    fn get_table<'a, S: Scalar>(&self, accessor: &'a dyn DataAccessor<S>) -> Vec<Column<'a, S>> {
        self.schema
            .iter()
            .map(|field| {
                accessor.get_column(ColumnRef::new(
                    self.table_ref,
                    field.name(),
                    field.data_type(),
                ))
            })
            .collect()
    }
}

impl ProofPlan for TableExec {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        builder.count_intermediate_mles(self.schema.len());
        Ok(())
    }

    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
    ) -> Result<Vec<S>, ProofError> {
        Ok(repeat_with(|| builder.consume_intermediate_mle())
            .take(self.schema.len())
            .collect::<Vec<_>>())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.schema.clone()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.schema
            .iter()
            .map(|field| ColumnRef::new(self.table_ref, field.name(), field.data_type()))
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.table_ref}
    }
}

impl ProverEvaluate for TableExec {
    #[tracing::instrument(name = "TableExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        _alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        self.get_table(accessor)
    }

    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}

    #[tracing::instrument(name = "TableExec::final_round_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let table = self.get_table(accessor);
        for column in &table {
            builder.produce_intermediate_mle(column.as_scalar(alloc));
        }
        table
    }
}
