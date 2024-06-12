use super::{
    dense_filter_util::{fold_columns, fold_vals},
    filter_columns,
    provable_expr_plan::ProvableExprPlan,
    ColumnExpr, ProvableExpr, TableExpr,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
        },
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{
        CountBuilder, HonestProver, Indexes, ProofBuilder, ProofExpr, ProverEvaluate,
        ProverHonestyMarker, ResultBuilder, SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use bumpalo::Bump;
use core::iter::repeat_with;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, marker::PhantomData};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
///
/// This differs from the [`FilterExpr`] in that the result is not a sparse table.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OstensibleDenseFilterExpr<C: Commitment, H: ProverHonestyMarker> {
    pub(super) results: Vec<ColumnExpr<C>>,
    pub(super) table: TableExpr,
    pub(super) where_clause: ProvableExprPlan<C>,
    phantom: PhantomData<H>,
}

impl<C: Commitment, H: ProverHonestyMarker> OstensibleDenseFilterExpr<C, H> {
    /// Creates a new dense_filter expression.
    pub fn new(
        results: Vec<ColumnExpr<C>>,
        table: TableExpr,
        where_clause: ProvableExprPlan<C>,
    ) -> Self {
        Self {
            results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }
}

impl<C: Commitment, H: ProverHonestyMarker> ProofExpr<C> for OstensibleDenseFilterExpr<C, H>
where
    OstensibleDenseFilterExpr<C, H>: ProverEvaluate<C::Scalar>,
{
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in self.results.iter() {
            expr.count(builder)?;
            builder.count_result_columns(1);
        }
        builder.count_intermediate_mles(2);
        builder.count_subpolynomials(3);
        builder.count_degree(3);
        builder.count_post_result_challenges(2);
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
    ) -> Result<(), ProofError> {
        // 1. selection
        let selection_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let columns_evals = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.verifier_evaluate(builder, accessor))
                .collect::<Result<Vec<_>, _>>()?,
        );
        // 3. indexes
        let indexes_eval = builder
            .mle_evaluations
            .result_indexes_evaluation
            .ok_or(ProofError::VerificationError("invalid indexes"))?;
        // 4. filtered_columns
        let filtered_columns_evals =
            Vec::from_iter(repeat_with(|| builder.consume_result_mle()).take(self.results.len()));

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        verify_filter(
            builder,
            alpha,
            beta,
            columns_evals,
            selection_eval,
            filtered_columns_evals,
        )
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
pub type DenseFilterExpr<C> = OstensibleDenseFilterExpr<C, HonestProver>;

impl<C: Commitment> ProverEvaluate<C::Scalar> for DenseFilterExpr<C> {
    #[tracing::instrument(name = "DenseFilterExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause
                .result_evaluate(builder.table_length(), alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let columns = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.result_evaluate(builder.table_length(), alloc, accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(result_len as u64)));
        // 4. set filtered_columns
        for col in filtered_columns {
            builder.produce_result_column(col);
        }
        builder.request_post_result_challenges(2);
    }

    #[tracing::instrument(name = "DenseFilterExpr::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let columns = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.prover_evaluate(builder, alloc, accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        prove_filter::<C::Scalar>(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            selection,
            &filtered_columns,
            result_len,
        );
    }
}

fn verify_filter<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    alpha: C::Scalar,
    beta: C::Scalar,
    c_evals: Vec<C::Scalar>,
    s_eval: C::Scalar,
    d_evals: Vec<C::Scalar>,
) -> Result<(), ProofError> {
    let one_eval = builder.mle_evaluations.one_evaluation;
    let rand_eval = builder.mle_evaluations.random_evaluation;

    let chi_eval = match builder.mle_evaluations.result_indexes_evaluation {
        Some(eval) => eval,
        None => return Err(ProofError::VerificationError("Result indexes not valid.")),
    };

    let c_fold_eval = alpha * one_eval + fold_vals(beta, &c_evals);
    let d_bar_fold_eval = alpha * one_eval + fold_vals(beta, &d_evals);
    let c_star_eval = builder.consume_intermediate_mle();
    let d_star_eval = builder.consume_intermediate_mle();

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial_evaluation(&(c_star_eval * s_eval - d_star_eval));

    // c_fold * c_star - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &(rand_eval * (c_fold_eval * c_star_eval - one_eval)),
    );

    // d_bar_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &(rand_eval * (d_bar_fold_eval * d_star_eval - chi_eval)),
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prove_filter<'a, S: Scalar + 'a>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    c: &[Column<S>],
    s: &'a [bool],
    d: &[Column<S>],
    m: usize,
) {
    let n = builder.table_length();
    let chi = alloc.alloc_slice_fill_copy(n, false);
    chi[..m].fill(true);

    let c_fold = alloc.alloc_slice_fill_copy(n, alpha);
    fold_columns(c_fold, One::one(), beta, c);
    let d_bar_fold = alloc.alloc_slice_fill_copy(n, alpha);
    fold_columns(d_bar_fold, One::one(), beta, d);

    let c_star = alloc.alloc_slice_copy(c_fold);
    let d_star = alloc.alloc_slice_copy(d_bar_fold);
    d_star[m..].fill(Zero::zero());
    slice_ops::batch_inversion(c_star);
    slice_ops::batch_inversion(&mut d_star[..m]);

    builder.produce_intermediate_mle(c_star as &[_]);
    builder.produce_intermediate_mle(d_star as &[_]);

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star as &[_]), Box::new(s)]),
            (-S::one(), vec![Box::new(d_star as &[_])]),
        ],
    );

    // c_fold * c_star - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![]),
        ],
    );

    // d_bar_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(d_star as &[_]), Box::new(d_bar_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi as &[_])]),
        ],
    );
}
