use super::DynProofPlan;
use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableOptions, TableRef},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ProofExpr},
    },
};
use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use core::iter::repeat_with;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <input>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionExec {
    pub(super) aliased_results: Vec<AliasedDynProofExpr>,
    pub(super) input: Box<DynProofPlan>,
}

impl ProjectionExec {
    /// Creates a new projection expression.
    pub fn new(aliased_results: Vec<AliasedDynProofExpr>, input: DynProofPlan) -> Self {
        Self {
            aliased_results,
            input: Box::new(input),
        }
    }
}

impl ProofPlan for ProjectionExec {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.input.count(builder)?;
        for aliased_expr in &self.aliased_results {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        Ok(())
    }

    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
    ) -> Result<Vec<S>, ProofError> {
        //TODO: Switch to ref to the input itself
        let table_ref = *self.input.get_table_references().iter().next().unwrap();
        let input_commitment_map: IndexMap<ColumnRef, S> = self
            .input
            .get_column_result_fields()
            .iter()
            .map(|field| ColumnRef::new(table_ref, field.name(), field.data_type()))
            .zip(self.input.verifier_evaluate(builder, accessor, None)?)
            .collect();
        self.aliased_results
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .verifier_evaluate(builder, &input_commitment_map)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let columns_evals: Vec<_> = repeat_with(|| builder.consume_intermediate_mle())
            .take(self.aliased_results.len())
            .collect();
        Ok(columns_evals)
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = self.input.get_column_references();
        self.aliased_results.iter().for_each(|aliased_expr| {
            aliased_expr.expr.get_column_references(&mut columns);
        });
        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

impl ProverEvaluate for ProjectionExec {
    #[tracing::instrument(name = "ProjectionExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let input_table = self.input.result_evaluate(alloc, table_map);
        Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|aliased_expr| {
                (
                    aliased_expr.alias,
                    aliased_expr.expr.result_evaluate(alloc, &input_table),
                )
            }),
            TableOptions::new(Some(input_table.num_rows())),
        )
        .expect("Failed to create table from iterator")
    }

    fn first_round_evaluate(&self, builder: &mut FirstRoundBuilder) {
        self.input.first_round_evaluate(builder);
    }

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
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let input_table = self.input.final_round_evaluate(builder, alloc, table_map);
        // 1. Evaluate result expressions
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|aliased_expr| {
                (
                    aliased_expr.alias,
                    aliased_expr
                        .expr
                        .prover_evaluate(builder, alloc, &input_table),
                )
            }),
            TableOptions::new(Some(input_table.num_rows())),
        )
        .expect("Failed to create table from iterator");
        // 2. Produce MLEs
        res.inner_table().values().for_each(|column| {
            builder.produce_intermediate_mle(column.as_scalar(alloc));
        });
        res
    }
}
