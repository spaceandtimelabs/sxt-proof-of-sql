use super::{prove_filter, verify_filter, DynProofPlan};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            filter_util::filter_columns, Column, ColumnField, ColumnRef, CommitmentAccessor,
            DataAccessor, MetadataAccessor, OwnedTable,
        },
        map::IndexSet,
        proof::ProofError,
    },
    sql::proof::{
        CountBuilder, Indexes, ProofBuilder, ProofPlan, ProverEvaluate, ResultBuilder,
        VerificationBuilder,
    },
};
use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use core::iter::repeat_with;
use serde::{Deserialize, Serialize};

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan> LIMIT <fetch> [OFFSET <skip>]
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SliceExec<C: Commitment> {
    pub(super) input: Box<DynProofPlan<C>>,
    pub(super) skip: usize,
    pub(super) fetch: Option<usize>,
}

/// Get the boolean slice selection from the number of rows, skip and fetch
fn get_slice_select(num_rows: usize, skip: usize, fetch: Option<usize>) -> Vec<bool> {
    if let Some(fetch) = fetch {
        let end = skip + fetch;
        (0..num_rows).map(|i| i >= skip && i < end).collect()
    } else {
        (0..num_rows).map(|i| i >= skip).collect()
    }
}

impl<C: Commitment> SliceExec<C> {
    /// Creates a new slice execution plan.
    pub fn new(input: Box<DynProofPlan<C>>, skip: usize, fetch: Option<usize>) -> Self {
        Self { input, skip, fetch }
    }
}

impl<C: Commitment> ProofPlan<C> for SliceExec<C>
where
    SliceExec<C>: ProverEvaluate<C::Scalar>,
{
    fn count(
        &self,
        builder: &mut CountBuilder,
        accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.input.count(builder, accessor)?;
        builder.count_intermediate_mles(self.input.get_column_result_fields().len());
        builder.count_intermediate_mles(3);
        builder.count_subpolynomials(3);
        builder.count_degree(3);
        builder.count_post_result_challenges(2);
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        //TODO: Hmm we should look for the output of the input plan
        self.input.get_length(accessor)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        self.input.get_offset(accessor)
    }

    #[allow(unused_variables)]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        // 1. columns
        // TODO: Make sure `GroupByExec` as self.input is supported
        let columns_evals = self.input.verifier_evaluate(builder, accessor, None)?;
        // 2. selection
        let selection_eval = builder.consume_intermediate_mle();
        // 3. indexes
        let indexes_eval = builder.mle_evaluations.result_indexes_evaluation.ok_or(
            ProofError::VerificationError {
                error: "invalid indexes",
            },
        )?;
        // 4. filtered_columns
        let filtered_columns_evals: Vec<_> = repeat_with(|| builder.consume_intermediate_mle())
            .take(columns_evals.len())
            .collect();
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        verify_filter(
            builder,
            alpha,
            beta,
            &columns_evals,
            selection_eval,
            &filtered_columns_evals,
        )?;
        Ok(filtered_columns_evals)
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.input.get_column_result_fields()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.input.get_column_references()
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for SliceExec<C> {
    #[tracing::instrument(name = "SliceExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        // 1. columns
        let columns = self.input.result_evaluate(builder, alloc, accessor);
        let input_num_rows = if columns.is_empty() {
            0
        } else {
            columns[0].len()
        };
        // 2. select
        let select = get_slice_select(input_num_rows, self.skip, self.fetch);
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, &select);
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(result_len as u64)));
        builder.request_post_result_challenges(2);
        filtered_columns
    }

    #[tracing::instrument(name = "SliceExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        // 1. columns
        let columns = self.input.prover_evaluate(builder, alloc, accessor);
        let input_num_rows = if columns.is_empty() {
            0
        } else {
            columns[0].len()
        };
        // 2. select
        let select = get_slice_select(input_num_rows, self.skip, self.fetch);
        let select_ref: &'a [_] = alloc.alloc_slice_copy(&select);
        builder.produce_intermediate_mle(select_ref);
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, &select);
        // 3. Produce MLEs
        filtered_columns.iter().for_each(|column| {
            builder.produce_intermediate_mle(column.as_scalar(alloc));
        });
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        prove_filter::<C::Scalar>(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            select_ref,
            &filtered_columns,
            result_len,
        );
        columns
    }
}
