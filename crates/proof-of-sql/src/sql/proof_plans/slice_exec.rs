use super::{
    filter_exec::{prove_filter, verify_filter},
    DynProofPlan,
};
use crate::{
    base::{
        database::{
            filter_util::filter_columns, ColumnField, ColumnRef, LiteralValue, OwnedTable, Table,
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
///     <ProofPlan> LIMIT <fetch> [OFFSET <skip>]
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SliceExec {
    pub(super) input: Box<DynProofPlan>,
    pub(super) skip: usize,
    pub(super) fetch: Option<usize>,
}

/// Get the boolean slice selection from the number of rows, skip and fetch
fn get_slice_select(num_rows: usize, skip: usize, fetch: Option<usize>) -> Vec<bool> {
    repeat_n(false, skip)
        .chain(repeat_n(true, fetch.unwrap_or(num_rows)))
        .chain(repeat(false))
        .take(num_rows)
        .collect()
}

impl SliceExec {
    /// Creates a new slice execution plan.
    pub fn new(input: Box<DynProofPlan>, skip: usize, fetch: Option<usize>) -> Self {
        Self { input, skip, fetch }
    }
}

impl ProofPlan for SliceExec
where
    SliceExec: ProverEvaluate,
{
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, S>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        // 1. columns
        let input_table_eval =
            self.input
                .verifier_evaluate(builder, accessor, None, chi_eval_map, params)?;
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

impl ProverEvaluate for SliceExec {
    #[tracing::instrument(name = "SliceExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        // 1. columns
        let input = self
            .input
            .first_round_evaluate(builder, alloc, table_map, params);
        let input_length = input.num_rows();
        let columns = input.columns().copied().collect::<Vec<_>>();
        // 2. select
        let select = get_slice_select(input_length, self.skip, self.fetch);
        // The selected range is (offset_index, max_index]
        let offset_index = self.skip.min(input_length);
        let max_index = if let Some(fetch) = self.fetch {
            (self.skip + fetch).min(input_length)
        } else {
            input_length
        };
        let output_length = max_index - offset_index;
        // Compute filtered_columns
        let (filtered_columns, _) = filter_columns(alloc, &columns, &select);
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.get_column_result_fields()
                .into_iter()
                .map(|expr| expr.name())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(output_length);
        builder.produce_chi_evaluation_length(offset_index);
        builder.produce_chi_evaluation_length(max_index);

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "SliceExec::prover_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        // 1. columns
        let input = self
            .input
            .final_round_evaluate(builder, alloc, table_map, params);
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
