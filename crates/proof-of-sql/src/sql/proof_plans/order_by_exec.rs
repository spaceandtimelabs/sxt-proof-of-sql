use super::{
    filter_exec::{prove_filter, verify_filter},
    DynProofPlan,
};
use crate::{
    base::{
        database::{
            order_by_util::OrderIndexDirectionPairs, ColumnField, ColumnRef, OwnedTable, Table,
            TableEvaluation, TableOptions, TableRef,
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
use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use core::iter::repeat;
use itertools::repeat_n;
use serde::{Deserialize, Serialize};

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan> ORDER BY <OrderByExpr>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrderByExec {
    pub(super) input: Box<DynProofPlan>,
    pub(super) order_index_dir_pairs: OrderIndexDirectionPairs,
}

impl OrderByExec {
    /// Creates a new ORDER BY execution plan.
    #[allow(dead_code)]
    pub fn new(input: Box<DynProofPlan>, order_index_dir_pairs: OrderIndexDirectionPairs) -> Self {
        Self { input, order_index_dir_pairs }
    }
}

impl ProofPlan for OrderByExec
where
    OrderByExec: ProverEvaluate,
{
    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        // 1. columns
        let input_table_eval =
            self.input
                .verifier_evaluate(builder, accessor, None, chi_eval_map)?;
        let output_chi_eval = builder.try_consume_chi_evaluation()?;
        let columns_evals = input_table_eval.column_evals();
        // 2. selection
        // The selected range is (offset_index, max_index]
        let offset_chi_eval = builder.try_consume_chi_evaluation()?;
        let max_chi_eval = builder.try_consume_chi_evaluation()?;
        let selection_eval = max_chi_eval - offset_chi_eval;
        // 3. filtered_columns
        let filtered_columns_evals =
            builder.try_consume_final_round_mle_evaluations(columns_evals.len())?;
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;

        verify_filter(
            builder,
            alpha,
            beta,
            input_table_eval.chi_eval(),
            output_chi_eval,
            columns_evals,
            selection_eval,
            &filtered_columns_evals,
        )?;
        Ok(TableEvaluation::new(
            filtered_columns_evals,
            output_chi_eval,
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.input.get_column_result_fields()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.input.get_column_references()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

impl ProverEvaluate for OrderByExec {
    #[tracing::instrument(name = "OrderByExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        // 1. columns
        let input = self.input.first_round_evaluate(builder, alloc, table_map);
        let input_length = input.num_rows();
        let columns = input.columns().copied().collect::<Vec<_>>();
        // 2. R is a permutation of A
        // 3. R is monotonic on the selected column
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(output_length);
        builder.produce_chi_evaluation_length(offset_index);
        builder.produce_chi_evaluation_length(max_index);

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "OrderByExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        // 1. columns
        let input = self.input.final_round_evaluate(builder, alloc, table_map);
        let columns = input.columns().copied().collect::<Vec<_>>();
        // 2. select
        let select = get_slice_select(input.num_rows(), self.skip, self.fetch);
        let select_ref: &'a [_] = alloc.alloc_slice_copy(&select);
        let output_length = select.iter().filter(|b| **b).count();
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, &select);
        // 3. Produce MLEs
        filtered_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        prove_filter::<S>(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            select_ref,
            &filtered_columns,
            input.num_rows(),
            result_len,
        );
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.get_column_result_fields()
                .into_iter()
                .map(|expr| expr.name())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        res
    }
}
