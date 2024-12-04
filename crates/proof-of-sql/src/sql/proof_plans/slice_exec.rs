use super::{
    filter_exec::{prove_filter, verify_filter},
    DynProofPlan,
};
use crate::{
    base::{
        database::{
            filter_util::filter_columns, ColumnField, ColumnRef, OwnedTable, Table,
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
use core::iter::{repeat, repeat_with};
use itertools::repeat_n;
use serde::{Deserialize, Serialize};

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan> LIMIT <fetch> [OFFSET <skip>]
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
    #[allow(dead_code)]
    pub fn new(input: Box<DynProofPlan>, skip: usize, fetch: Option<usize>) -> Self {
        Self { input, skip, fetch }
    }
}

impl ProofPlan for SliceExec
where
    SliceExec: ProverEvaluate,
{
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.input.count(builder)?;
        builder.count_intermediate_mles(self.input.get_column_result_fields().len());
        builder.count_intermediate_mles(2);
        builder.count_subpolynomials(3);
        builder.count_degree(3);
        builder.count_post_result_challenges(2);
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
        // We do not support `GroupByExec` as input for now
        if matches!(*self.input, DynProofPlan::GroupBy(_)) {
            return Err(ProofError::UnsupportedQueryPlan {
                error: "GroupByExec as input for another plan is not supported",
            });
        }
        let input_table_eval =
            self.input
                .verifier_evaluate(builder, accessor, None, one_eval_map)?;
        let output_one_eval = builder.consume_one_evaluation();
        let columns_evals = input_table_eval.column_evals();
        // 2. selection
        // The selected range is (offset_index, max_index]
        let offset_one_eval = builder.consume_one_evaluation();
        let max_one_eval = builder.consume_one_evaluation();
        let selection_eval = max_one_eval - offset_one_eval;
        // 3. filtered_columns
        let filtered_columns_evals: Vec<_> = repeat_with(|| builder.consume_intermediate_mle())
            .take(columns_evals.len())
            .collect();
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        verify_filter(
            builder,
            alpha,
            beta,
            *input_table_eval.one_eval(),
            output_one_eval,
            columns_evals,
            selection_eval,
            &filtered_columns_evals,
        )?;
        Ok(TableEvaluation::new(
            filtered_columns_evals,
            output_one_eval,
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
        builder: &mut FirstRoundBuilder,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> (Table<'a, S>, Vec<usize>) {
        // 1. columns
        let (input, input_one_eval_lengths) =
            self.input.first_round_evaluate(builder, alloc, table_map);
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
        let mut one_eval_lengths = input_one_eval_lengths;
        one_eval_lengths.extend(vec![output_length, offset_index, max_index]);
        builder.request_post_result_challenges(2);
        (res, one_eval_lengths)
    }

    #[tracing::instrument(name = "SliceExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
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
        Table::<'a, S>::try_from_iter_with_options(
            self.get_column_result_fields()
                .into_iter()
                .map(|expr| expr.name())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator")
    }
}
