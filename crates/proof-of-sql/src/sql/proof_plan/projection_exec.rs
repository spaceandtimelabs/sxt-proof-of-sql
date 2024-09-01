use super::{AliasedDynProofExpr, ProofExpr, TableExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor, OwnedTable,
        },
        proof::ProofError,
    },
    sql::proof::{
        CountBuilder, Indexes, ProofBuilder, ProofPlan, ProverEvaluate, ResultBuilder,
        VerificationBuilder,
    },
};
use bumpalo::Bump;
use core::iter::repeat_with;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionExec<C: Commitment> {
    pub(super) aliased_results: Vec<AliasedDynProofExpr<C>>,
    pub(super) table: TableExpr,
}

impl<C: Commitment> ProjectionExec<C> {
    /// Creates a new projection expression.
    pub fn new(aliased_results: Vec<AliasedDynProofExpr<C>>, table: TableExpr) -> Self {
        Self {
            aliased_results,
            table,
        }
    }
}

impl<C: Commitment> ProofPlan<C> for ProjectionExec<C> {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        for aliased_expr in self.aliased_results.iter() {
            aliased_expr.expr.count(builder)?;
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

    #[allow(unused_variables)]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<(), ProofError> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        let _columns_evals = Vec::from_iter(
            repeat_with(|| builder.consume_result_mle()).take(self.aliased_results.len()),
        );
        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::new();
        self.aliased_results.iter().for_each(|aliased_expr| {
            aliased_expr.expr.get_column_references(&mut columns);
        });
        columns
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for ProjectionExec<C> {
    #[tracing::instrument(name = "ProjectionExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        let columns = Vec::from_iter(self.aliased_results.iter().map(|aliased_expr| {
            aliased_expr
                .expr
                .result_evaluate(builder.table_length(), alloc, accessor)
        }));
        builder.set_result_indexes(Indexes::Dense(0..(builder.table_length() as u64)));
        for col in columns {
            builder.produce_result_column(col);
        }
    }

    #[tracing::instrument(name = "ProjectionExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        self.aliased_results.iter().for_each(|aliased_expr| {
            aliased_expr.expr.prover_evaluate(builder, alloc, accessor);
        });
    }
}
