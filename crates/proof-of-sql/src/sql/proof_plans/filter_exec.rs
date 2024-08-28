use super::{fold_columns, fold_vals};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            filter_util::filter_columns, Column, ColumnField, ColumnRef, CommitmentAccessor,
            DataAccessor, MetadataAccessor, OwnedTable,
        },
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            CountBuilder, HonestProver, Indexes, ProofBuilder, ProofPlan, ProverEvaluate,
            ProverHonestyMarker, ResultBuilder, SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, ProofExpr, TableExpr},
    },
};
use bumpalo::Bump;
use core::{iter::repeat_with, marker::PhantomData};
use indexmap::IndexSet;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
///
/// This differs from the [`FilterExec`] in that the result is not a sparse table.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OstensibleFilterExec<C: Commitment, H: ProverHonestyMarker> {
    pub(super) aliased_results: Vec<AliasedDynProofExpr<C>>,
    pub(super) table: TableExpr,
    /// TODO: add docs
    pub(crate) where_clause: DynProofExpr<C>,
    phantom: PhantomData<H>,
}

impl<C: Commitment, H: ProverHonestyMarker> OstensibleFilterExec<C, H> {
    /// Creates a new filter expression.
    pub fn new(
        aliased_results: Vec<AliasedDynProofExpr<C>>,
        table: TableExpr,
        where_clause: DynProofExpr<C>,
    ) -> Self {
        Self {
            aliased_results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }
}

impl<C: Commitment, H: ProverHonestyMarker> ProofPlan<C> for OstensibleFilterExec<C, H>
where
    OstensibleFilterExec<C, H>: ProverEvaluate<C::Scalar>,
{
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for aliased_expr in self.aliased_results.iter() {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
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
        _result: Option<&OwnedTable<C::Scalar>>,
        is_top_level: bool,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        // 1. selection
        let selection_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let columns_evals = Vec::from_iter(
            self.aliased_results
                .iter()
                .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
                .collect::<Result<Vec<_>, _>>()?,
        );
        // 3. indexes
        let indexes_eval = builder
            .mle_evaluations
            .result_indexes_evaluation
            .ok_or(ProofError::VerificationError("invalid indexes"))?;
        // 4. filtered_columns
        let filtered_columns_evals = Vec::from_iter(
            repeat_with(|| builder.consume_intermediate_mle()).take(self.aliased_results.len()),
        );
        assert!(filtered_columns_evals.len() == self.aliased_results.len());

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
        self.aliased_results
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::new();

        for aliased_expr in self.aliased_results.iter() {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

/// Alias for a filter expression with a honest prover.
pub type FilterExec<C> = OstensibleFilterExec<C, HonestProver>;

impl<C: Commitment> ProverEvaluate<C::Scalar> for FilterExec<C> {
    #[tracing::instrument(name = "FilterExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause
                .result_evaluate(builder.table_length(), alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let columns = Vec::from_iter(self.aliased_results.iter().map(|aliased_expr| {
            aliased_expr
                .expr
                .result_evaluate(builder.table_length(), alloc, accessor)
        }));
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(result_len as u64)));
        builder.request_post_result_challenges(2);
        filtered_columns
    }

    #[tracing::instrument(name = "FilterExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
        is_top_level: bool,
    ) -> Vec<Column<'a, C::Scalar>> {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let columns = Vec::from_iter(
            self.aliased_results
                .iter()
                .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
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
            selection,
            &filtered_columns,
            result_len,
        );
        filtered_columns
    }
}

pub(crate) fn verify_filter<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    alpha: C::Scalar,
    beta: C::Scalar,
    c_evals: &[C::Scalar],
    s_eval: C::Scalar,
    d_evals: &[C::Scalar],
) -> Result<(), ProofError> {
    let one_eval = builder.mle_evaluations.one_evaluation;

    let chi_eval = match builder.mle_evaluations.result_indexes_evaluation {
        Some(eval) => eval,
        None => return Err(ProofError::VerificationError("Result indexes not valid.")),
    };

    let c_fold_eval = alpha * one_eval + fold_vals(beta, c_evals);
    let d_bar_fold_eval = alpha * one_eval + fold_vals(beta, d_evals);
    let c_star_eval = builder.consume_intermediate_mle();
    let d_star_eval = builder.consume_intermediate_mle();

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval * s_eval - d_star_eval,
    );

    // c_fold * c_star - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        c_fold_eval * c_star_eval - one_eval,
    );

    // d_bar_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        d_bar_fold_eval * d_star_eval - chi_eval,
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prove_filter<'a, S: Scalar + 'a>(
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
