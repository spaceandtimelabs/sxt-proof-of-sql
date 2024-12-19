use super::DynProofPlan;
use crate::{
    base::{
        database::{
            join_util::cross_join, ColumnField, ColumnRef, OwnedTable, Table,
            TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
        VerificationBuilder,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::iter::repeat_with;
use serde::{Deserialize, Serialize};

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan> JOIN <ProofPlan>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CrossJoinExec {
    pub(super) left: Box<DynProofPlan>,
    pub(super) right: Box<DynProofPlan>,
}

impl CrossJoinExec {
    /// Create a new `CrossJoinExec` with the given left and right plans
    pub fn new(left: Box<DynProofPlan>, right: Box<DynProofPlan>) -> Self {
        Self { left, right }
    }
}

impl ProofPlan for CrossJoinExec
where
    CrossJoinExec: ProverEvaluate,
{
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.left.count(builder)?;
        self.right.count(builder)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        // 1. columns
        // TODO: Make sure `GroupByExec` as self.input is supported
        let left_eval = self
            .left
            .verifier_evaluate(builder, accessor, None, one_eval_map)?;
        let right_eval = self
            .right
            .verifier_evaluate(builder, accessor, None, one_eval_map)?;
        let output_one_eval = builder.consume_one_eval();
        todo!()
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.left
            .get_column_result_fields()
            .into_iter()
            .chain(self.right.get_column_result_fields())
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.left
            .get_column_references()
            .into_iter()
            .chain(self.right.get_column_references())
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.left
            .get_table_references()
            .into_iter()
            .chain(self.right.get_table_references())
            .collect()
    }
}

impl ProverEvaluate for CrossJoinExec {
    #[tracing::instrument(name = "CrossJoinExec::result_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let left = self.left.first_round_evaluate(builder, alloc, table_map);
        let right = self.right.first_round_evaluate(builder, alloc, table_map);
        let res = cross_join(left, right, alloc);
        let output_length = left.num_rows() * right.num_rows();
        builder.request_post_result_challenges(2);
        builder.produce_one_evaluation_length(output_length);
        res
    }

    #[tracing::instrument(name = "CrossJoinExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let left = self.left.prover_evaluate(builder, alloc, table_map);
        let right = self.right.prover_evaluate(builder, alloc, table_map);
        let res = cross_join(left, right, alloc);
        
        
    }
}
