use super::{filter_columns, BoolExpr, ColumnExpr, TableExpr};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor},
        proof::ProofError,
    },
    sql::proof::{
        CountBuilder, HonestProver, Indexes, ProofBuilder, ProofExpr, ProverEvaluate,
        ProverHonestyMarker, ResultBuilder, VerificationBuilder,
    },
};
use bumpalo::Bump;
use core::iter::repeat_with;
use dyn_partial_eq::DynPartialEq;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, marker::PhantomData};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
///
/// This differs from the [`FilterExpr`] in that the result is not a sparse table.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OstensibleDenseFilterExpr<H: ProverHonestyMarker> {
    pub(super) results: Vec<ColumnExpr>,
    pub(super) table: TableExpr,
    pub(super) where_clause: Box<dyn BoolExpr>,
    phantom: PhantomData<H>,
}

// This is required because derive(DynPartialEq) does not work with generics
impl<H: ProverHonestyMarker> DynPartialEq for OstensibleDenseFilterExpr<H> {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn box_eq(&self, other: &dyn core::any::Any) -> bool {
        other.downcast_ref::<Self>().map_or(false, |a| self == a)
    }
}

impl<H: ProverHonestyMarker> OstensibleDenseFilterExpr<H> {
    /// Creates a new dense_filter expression.
    pub fn new(
        results: Vec<ColumnExpr>,
        table: TableExpr,
        where_clause: Box<dyn BoolExpr>,
    ) -> Self {
        Self {
            results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }

    /// Returns the result expressions.
    pub fn get_results(&self) -> &[ColumnExpr] {
        &self.results[..]
    }
}

impl<H: ProverHonestyMarker> ProofExpr for OstensibleDenseFilterExpr<H>
where
    OstensibleDenseFilterExpr<H>: ProverEvaluate,
{
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in self.results.iter() {
            expr.count(builder);
            builder.count_result_columns(1);
        }
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.table.table_ref)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.table.table_ref)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.dense_filter_expr.verifier_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<(), ProofError> {
        // 1. selection
        let selection_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let columns_evals = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.verifier_evaluate(builder, accessor)),
        );
        // 3. indexes
        let indexes_eval = builder
            .mle_evaluations
            .result_indexes_evaluation
            .ok_or(ProofError::VerificationError("invalid indexes"))?;
        // 4. filtered_columns
        let filtered_columns_evals =
            Vec::from_iter(repeat_with(|| builder.consume_result_mle()).take(self.results.len()));
        // todo!: build the proof components that show that the filtered_columns_evals are actually correct.
        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let mut columns = Vec::with_capacity(self.results.len());
        for col in self.results.iter() {
            columns.push(col.get_column_field());
        }
        columns
    }

    fn get_column_references(&self) -> HashSet<ColumnRef> {
        let mut columns = HashSet::new();

        for col in self.results.iter() {
            columns.insert(col.get_column_reference());
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

/// Alias for a dense filter expression with a honest prover.
pub type DenseFilterExpr = OstensibleDenseFilterExpr<HonestProver>;
impl ProverEvaluate for DenseFilterExpr {
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        // 1. selection
        let selection = self
            .where_clause
            .result_evaluate(builder.table_length(), alloc, accessor);
        // 2. columns
        let columns = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.result_evaluate(accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(result_len as u64)));
        // 4. set filtered_columns
        for col in filtered_columns {
            builder.produce_result_column(col);
        }
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.dense_filter_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        // 1. selection
        let selection = self.where_clause.prover_evaluate(builder, alloc, accessor);
        // 2. columns
        let columns = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.prover_evaluate(builder, accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        // todo!: build the proof components that show that the filtered_columns are actually correct.
    }
}
