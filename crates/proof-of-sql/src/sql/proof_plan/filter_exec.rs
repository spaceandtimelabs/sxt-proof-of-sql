use super::{dyn_proof_expr::DynProofExpr, FilterResultExpr, ProvableExpr, TableExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
            OwnedTable,
        },
        proof::ProofError,
    },
    sql::proof::{
        CountBuilder, HonestProver, Indexes, ProofBuilder, ProofExecutionPlan, ProverEvaluate,
        ProverHonestyMarker, ResultBuilder, VerificationBuilder,
    },
};
use bumpalo::Bump;
use core::marker::PhantomData;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OstensibleFilterExec<C: Commitment, H: ProverHonestyMarker> {
    pub(super) results: Vec<FilterResultExpr>,
    pub(super) table: TableExpr,
    pub(super) where_clause: DynProofExpr<C>,
    phantom: PhantomData<H>,
}

impl<C: Commitment, H: ProverHonestyMarker> OstensibleFilterExec<C, H> {
    /// Creates a new filter expression.
    pub fn new(
        results: Vec<FilterResultExpr>,
        table: TableExpr,
        where_clause: DynProofExpr<C>,
    ) -> Self {
        Self {
            results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }

    /// Returns the result expressions.
    pub fn get_results(&self) -> &[FilterResultExpr] {
        &self.results[..]
    }
}

impl<C: Commitment, H: ProverHonestyMarker> ProofExecutionPlan<C> for OstensibleFilterExec<C, H>
where
    OstensibleFilterExec<C, H>: ProverEvaluate<C::Scalar>,
{
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in self.results.iter() {
            expr.count(builder);
        }
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.table.table_ref)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.table.table_ref)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<(), ProofError> {
        let selection_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        for expr in self.results.iter() {
            expr.verifier_evaluate(builder, accessor, &selection_eval);
        }
        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let mut columns = Vec::with_capacity(self.results.len());
        for col in self.results.iter() {
            columns.push(col.get_column_field());
        }
        columns
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::new();

        for col in self.results.iter() {
            columns.insert(col.get_column_reference());
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

pub type FilterExec<C> = OstensibleFilterExec<C, HonestProver>;
impl<C: Commitment> ProverEvaluate<C::Scalar> for FilterExec<C> {
    #[tracing::instrument(name = "FilterExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        // evaluate where clause
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause
                .result_evaluate(builder.table_length(), alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // set result indexes
        let indexes = selection
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| i as u64)
            .collect();
        builder.set_result_indexes(Indexes::Sparse(indexes));

        // evaluate result columns
        for expr in self.results.iter() {
            expr.result_evaluate(builder, accessor);
        }
    }

    #[tracing::instrument(name = "FilterExec::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        // evaluate where clause
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        for expr in self.results.iter() {
            expr.prover_evaluate(builder, alloc, accessor, selection);
        }
    }
}
