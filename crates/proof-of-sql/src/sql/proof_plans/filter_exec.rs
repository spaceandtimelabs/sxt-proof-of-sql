use super::{fold_columns, fold_vals};
use crate::{
    base::{
        database::{
            filter_util::filter_columns, Column, ColumnField, ColumnRef, OwnedTable, Table,
            TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            CountBuilder, FinalRoundBuilder, FirstRoundBuilder, HonestProver, ProofPlan,
            ProverEvaluate, ProverHonestyMarker, SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, ProofExpr, TableExpr},
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::{iter::repeat_with, marker::PhantomData};
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
///
/// This differs from the [`FilterExec`] in that the result is not a sparse table.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OstensibleFilterExec<H: ProverHonestyMarker> {
    pub(super) aliased_results: Vec<AliasedDynProofExpr>,
    pub(super) table: TableExpr,
    /// TODO: add docs
    pub(crate) where_clause: DynProofExpr,
    phantom: PhantomData<H>,
}

impl<H: ProverHonestyMarker> OstensibleFilterExec<H> {
    /// Creates a new filter expression.
    pub fn new(
        aliased_results: Vec<AliasedDynProofExpr>,
        table: TableExpr,
        where_clause: DynProofExpr,
    ) -> Self {
        Self {
            aliased_results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }
}

impl<H: ProverHonestyMarker> ProofPlan for OstensibleFilterExec<H>
where
    OstensibleFilterExec<H>: ProverEvaluate,
{
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for aliased_expr in &self.aliased_results {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
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
        let input_one_eval = *one_eval_map
            .get(&self.table.table_ref)
            .expect("One eval not found");
        // 1. selection
        let selection_eval =
            self.where_clause
                .verifier_evaluate(builder, accessor, input_one_eval)?;
        // 2. columns
        let columns_evals = Vec::from_iter(
            self.aliased_results
                .iter()
                .map(|aliased_expr| {
                    aliased_expr
                        .expr
                        .verifier_evaluate(builder, accessor, input_one_eval)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        // 3. filtered_columns
        let filtered_columns_evals: Vec<_> = repeat_with(|| builder.consume_intermediate_mle())
            .take(self.aliased_results.len())
            .collect();
        assert!(filtered_columns_evals.len() == self.aliased_results.len());

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        let output_one_eval = builder.consume_one_evaluation();

        verify_filter(
            builder,
            alpha,
            beta,
            input_one_eval,
            output_one_eval,
            &columns_evals,
            selection_eval,
            &filtered_columns_evals,
        )?;
        Ok(TableEvaluation::new(
            filtered_columns_evals,
            output_one_eval,
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();

        for aliased_expr in &self.aliased_results {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref])
    }
}

/// Alias for a filter expression with a honest prover.
pub type FilterExec = OstensibleFilterExec<HonestProver>;

impl ProverEvaluate for FilterExec {
    #[tracing::instrument(name = "FilterExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> (Table<'a, S>, Vec<usize>) {
        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self.where_clause.result_evaluate(alloc, table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.result_evaluate(alloc, table))
            .collect();

        // Compute filtered_columns and indexes
        let (filtered_columns, _) = filter_columns(alloc, &columns, selection);
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias)
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");
        builder.request_post_result_challenges(2);
        (res, vec![output_length])
    }

    #[tracing::instrument(name = "FilterExec::final_round_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> =
            self.where_clause.prover_evaluate(builder, alloc, table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, table))
            .collect();
        // Compute filtered_columns
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
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
            selection,
            &filtered_columns,
            table.num_rows(),
            result_len,
        );
        Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias)
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator")
    }
}

#[allow(clippy::unnecessary_wraps, clippy::too_many_arguments)]
pub(crate) fn verify_filter<S: Scalar>(
    builder: &mut VerificationBuilder<S>,
    alpha: S,
    beta: S,
    one_eval: S,
    chi_eval: S,
    c_evals: &[S],
    s_eval: S,
    d_evals: &[S],
) -> Result<(), ProofError> {
    let c_fold_eval = alpha * one_eval + fold_vals(beta, c_evals);
    let d_bar_fold_eval = alpha * one_eval + fold_vals(beta, d_evals);
    let c_star_eval = builder.consume_intermediate_mle();
    let d_star_eval = builder.consume_intermediate_mle();

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &SumcheckSubpolynomialType::ZeroSum,
        c_star_eval * s_eval - d_star_eval,
    );

    // c_fold * c_star - input_ones = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &SumcheckSubpolynomialType::Identity,
        c_fold_eval * c_star_eval - one_eval,
    );

    // d_bar_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &SumcheckSubpolynomialType::Identity,
        d_bar_fold_eval * d_star_eval - chi_eval,
    );

    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
pub(crate) fn prove_filter<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    c: &[Column<S>],
    s: &'a [bool],
    d: &[Column<S>],
    n: usize,
    m: usize,
) {
    let input_ones = alloc.alloc_slice_fill_copy(n, true);
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

    // c_fold * c_star - input_ones = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(input_ones as &[_])]),
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
