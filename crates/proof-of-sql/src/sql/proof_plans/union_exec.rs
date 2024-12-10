use super::{fold_columns, fold_vals, DynProofPlan};
use crate::{
    base::{
        database::{
            union_util::table_union, Column, ColumnField, ColumnRef, OwnedTable, Table,
            TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        polynomial::MultilinearExtension,
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{
        CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
        SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::iter::repeat_with;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan>
///     UNION ALL
///     <ProofPlan>
///     ...
///     UNION ALL
///     <ProofPlan>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnionExec {
    pub(super) inputs: Vec<DynProofPlan>,
    pub(super) schema: Vec<ColumnField>,
}

impl UnionExec {
    /// Creates a new union execution plan.
    ///
    /// # Panics
    /// Panics if the number of inputs is less than 2 which in practice should never happen.
    pub fn new(inputs: Vec<DynProofPlan>, schema: Vec<ColumnField>) -> Self {
        // There should be at least two inputs
        assert!(inputs.len() > 1);
        Self { inputs, schema }
    }
}

impl ProofPlan for UnionExec
where
    UnionExec: ProverEvaluate,
{
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        let num_parts = self.inputs.len();
        self.inputs
            .iter()
            .try_for_each(|input| input.count(builder))?;
        builder.count_intermediate_mles(num_parts + self.schema.len() + 1);
        builder.count_subpolynomials(num_parts + 2);
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
        let input_table_evals = self
            .inputs
            .iter()
            .map(|input| input.verifier_evaluate(builder, accessor, None, one_eval_map))
            .collect::<Result<Vec<_>, _>>()?;
        let num_parts = self.inputs.len();
        let input_column_evals = input_table_evals
            .iter()
            .map(TableEvaluation::column_evals)
            .collect::<Vec<_>>();
        let output_column_evals: Vec<_> = repeat_with(|| builder.consume_intermediate_mle())
            .take(self.schema.len())
            .collect();
        let input_one_evals = input_table_evals
            .iter()
            .map(TableEvaluation::one_eval)
            .collect::<Vec<_>>();
        let output_one_eval = builder.consume_one_evaluation();
        let gamma = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        verify_union(
            builder,
            gamma,
            beta,
            &input_column_evals,
            &output_column_evals,
            &input_one_evals,
            output_one_eval,
        );
        Ok(TableEvaluation::new(output_column_evals, output_one_eval))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.schema.clone()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.inputs
            .iter()
            .flat_map(ProofPlan::get_column_references)
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.inputs
            .iter()
            .flat_map(ProofPlan::get_table_references)
            .collect()
    }
}

impl ProverEvaluate for UnionExec {
    #[tracing::instrument(name = "UnionExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let inputs = self
            .inputs
            .iter()
            .map(|input| input.first_round_evaluate(builder, alloc, table_map))
            .collect::<Vec<_>>();
        let res = table_union(&inputs, alloc, self.schema.clone()).expect("Failed to union tables");
        builder.request_post_result_challenges(2);
        builder.produce_one_evaluation_length(res.num_rows());
        res
    }

    #[tracing::instrument(name = "UnionExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let inputs = self
            .inputs
            .iter()
            .map(|input| input.final_round_evaluate(builder, alloc, table_map))
            .collect::<Vec<_>>();
        let input_lengths = inputs.iter().map(Table::num_rows).collect::<Vec<_>>();
        let res = table_union(&inputs, alloc, self.schema.clone()).expect("Failed to union tables");
        let gamma = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        let input_columns: Vec<Vec<Column<'a, S>>> = inputs
            .iter()
            .map(|table| table.columns().copied().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let output_columns: Vec<Column<'a, S>> = res.columns().copied().collect::<Vec<_>>();
        // Produce intermediate MLEs for the union
        output_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        // Produce the proof for the union
        prove_union(
            builder,
            alloc,
            gamma,
            beta,
            &input_columns,
            &output_columns,
            &input_lengths,
            res.num_rows(),
        );
        res
    }
}

/// Verifies the union of tables.
///
/// # Panics
/// Should never panic if the code is correct.
#[allow(clippy::too_many_arguments)]
fn verify_union<S: Scalar>(
    builder: &mut VerificationBuilder<S>,
    gamma: S,
    beta: S,
    input_evals: &[&[S]],
    output_eval: &[S],
    input_one_evals: &[S],
    output_one_eval: S,
) {
    assert_eq!(input_evals.len(), input_one_evals.len());
    let c_star_evals = input_evals
        .iter()
        .zip(input_one_evals)
        .map(|(&input_eval, &input_one_eval)| {
            let c_fold_eval = gamma * fold_vals(beta, input_eval);
            let c_star_eval = builder.consume_intermediate_mle();
            // c_star + c_fold * c_star - input_ones = 0
            builder.produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                c_star_eval + c_fold_eval * c_star_eval - input_one_eval,
            );
            c_star_eval
        })
        .collect::<Vec<_>>();

    let d_bar_fold_eval = gamma * fold_vals(beta, output_eval);
    let d_star_eval = builder.consume_intermediate_mle();

    // d_star + d_bar_fold * d_star - output_ones = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        d_star_eval + d_bar_fold_eval * d_star_eval - output_one_eval,
    );

    // sum (sum c_star) - d_star = 0
    let zero_sum_terms_eval = c_star_evals
        .into_iter()
        .chain(core::iter::once(-d_star_eval))
        .sum::<S>();
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        zero_sum_terms_eval,
    );
}

/// Proves the union of tables.
///
/// # Panics
/// Should never panic if the code is correct.
#[allow(clippy::too_many_arguments)]
fn prove_union<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    gamma: S,
    beta: S,
    input_tables: &[Vec<Column<'a, S>>],
    output_table: &[Column<'a, S>],
    input_lengths: &[usize],
    output_length: usize,
) {
    // Number of `ProofPlan`s should be a constant
    assert_eq!(input_tables.len(), input_lengths.len());
    let c_stars = input_lengths
        .iter()
        .zip(input_tables.iter())
        .map(|(&input_length, input_table)| {
            // Indicator vector for the input table
            let input_ones = alloc.alloc_slice_fill_copy(input_length, true);

            let c_fold = alloc.alloc_slice_fill_copy(input_length, Zero::zero());
            fold_columns(c_fold, gamma, beta, input_table);

            let c_star = alloc.alloc_slice_copy(c_fold);
            slice_ops::add_const::<S, S>(c_star, One::one());
            slice_ops::batch_inversion(&mut c_star[..input_length]);
            let c_star_copy = alloc.alloc_slice_copy(c_star);
            builder.produce_intermediate_mle(c_star as &[_]);

            // c_star + c_fold * c_star - input_ones = 0
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![
                    (S::one(), vec![Box::new(c_star as &[_])]),
                    (
                        S::one(),
                        vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
                    ),
                    (-S::one(), vec![Box::new(input_ones as &[_])]),
                ],
            );
            c_star_copy
        })
        .collect::<Vec<_>>();
    // No need to produce intermediate MLEs for `d_fold` because it is
    // the sum of `c_fold`
    let d_fold = alloc.alloc_slice_fill_copy(output_length, Zero::zero());
    fold_columns(d_fold, gamma, beta, output_table);

    let d_star = alloc.alloc_slice_copy(d_fold);
    slice_ops::add_const::<S, S>(d_star, One::one());
    slice_ops::batch_inversion(d_star);
    builder.produce_intermediate_mle(d_star as &[_]);
    // d_star + d_fold * d_star - output_ones = 0
    let output_ones = alloc.alloc_slice_fill_copy(output_length, true);
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star as &[_])]),
            (
                S::one(),
                vec![Box::new(d_star as &[_]), Box::new(d_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(output_ones as &[_])]),
        ],
    );

    // sum (sum c_star) - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        c_stars
            .into_iter()
            .map(|c_star| {
                let boxed_c_star: Box<dyn MultilinearExtension<S>> = Box::new(c_star as &[_]);
                (S::one(), vec![boxed_c_star])
            })
            .chain(core::iter::once({
                let boxed_d_star: Box<dyn MultilinearExtension<S>> = Box::new(d_star as &[_]);
                (-S::one(), vec![boxed_d_star])
            }))
            .collect(),
    );
}
