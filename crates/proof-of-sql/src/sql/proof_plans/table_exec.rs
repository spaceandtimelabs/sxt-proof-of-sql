use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
            OwnedTable, TableRef,
        },
        map::{IndexSet, indexset},
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
    fn count(
        &self,
        _builder: &mut CountBuilder,
    ) -> Result<(), ProofError> {
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.table_ref)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.table_ref)
    }

    #[allow(unused_variables)]
    fn verifier_evaluate<C: Commitment>(
        &self,
        builder: &mut VerificationBuilder<C>,
        _accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
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
        _input_length: usize,
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
        table.iter().for_each(|column| {
            builder.produce_intermediate_mle(column.as_scalar(alloc));
        });
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_create_a_table_exec() {
        let table_ref = TableRef::new("namespace.table_name".parse().unwrap());
        let schema = vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt)];
        let table_exec = TableExec::new(table_ref, schema);
        assert_eq!(table_exec.table_ref, table_ref);
        assert_eq!(table_exec.schema, schema);
    }
    
}