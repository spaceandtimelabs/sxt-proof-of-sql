use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, OwnedTable, TableRef,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ProofExpr, TableExpr},
    },
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::iter::repeat_with;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionExec {
    pub(super) aliased_results: Vec<AliasedDynProofExpr>,
    pub(super) table: TableExpr,
}

impl ProjectionExec {
    /// Creates a new projection expression.
    pub fn new(aliased_results: Vec<AliasedDynProofExpr>, table: TableExpr) -> Self {
        Self {
            aliased_results,
            table,
        }
    }
}

impl ProofPlan for ProjectionExec {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        for aliased_expr in &self.aliased_results {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        Ok(())
    }

    #[allow(unused_variables)]
    fn verifier_evaluate<C: Commitment>(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(repeat_with(|| builder.consume_intermediate_mle())
            .take(self.aliased_results.len())
            .collect::<Vec<_>>())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();
        self.aliased_results.iter().for_each(|aliased_expr| {
            aliased_expr.expr.get_column_references(&mut columns);
        });
        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref])
    }
}

impl ProverEvaluate for ProjectionExec {
    #[tracing::instrument(name = "ProjectionExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        input_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .result_evaluate(input_length, alloc, accessor)
            })
            .collect();
        columns
    }

    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}

    #[tracing::instrument(
        name = "ProjectionExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        // 1. Evaluate result expressions
        let res: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor))
            .collect();
        // 2. Produce MLEs
        res.clone().into_iter().for_each(|column| {
            builder.produce_intermediate_mle(column.as_scalar(alloc));
        });
        res
    }
}
