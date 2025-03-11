use super::{fold_columns, fold_vals, DynProofPlan};
use crate::{
    base::{
        database::{
            group_by_util::{aggregate_columns, AggregatedColumns},
            order_by_util::compare_indexes_by_owned_columns,
            Column, ColumnField, ColumnRef, ColumnType, OwnedTable, Table, TableEvaluation,
            TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, ProofExpr},
    },
    utils::log,
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::iter;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
///         SUM(<sum_expr1>.expr) as <sum_expr1>.alias, ..., SUM(<sum_exprN>.expr) as <sum_exprN>.alias,
///     FROM <input>
///     WHERE <where_clause>
///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
/// ```
///
/// Note: if `group_by_exprs` is empty, then the query is equivalent to removing the `GROUP BY` clause.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AggregateExec {
    pub(super) group_by_exprs: Vec<DynProofExpr>,
    pub(super) sum_exprs: Vec<AliasedDynProofExpr>,
    pub(super) input: Box<DynProofPlan>,
    pub(super) where_clause: DynProofExpr,
    pub(super) is_top_level: bool,
}

impl AggregateExec {
    /// Creates a new aggregate proof plan.
    pub fn new(
        group_by_exprs: Vec<DynProofExpr>,
        sum_exprs: Vec<AliasedDynProofExpr>,
        input: Box<DynProofPlan>,
        where_clause: DynProofExpr,
        is_top_level: bool,
    ) -> Self {
        Self {
            group_by_exprs,
            sum_exprs,
            input,
            where_clause,
            is_top_level,
        }
    }
}

impl ProofPlan for AggregateExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        let input_eval = self
            .input
            .verifier_evaluate(builder, accessor, None, chi_eval_map)?;
        let input_chi_eval = input_eval.chi_eval();
        // Build new accessors
        // TODO: Make this work with inputs with multiple tables such as join
        // and union results
        let input_schema = self.input.get_column_result_fields();
        let input_table_refs = self.input.get_table_references();
        if input_table_refs.len() > 1 {
            return Err(ProofError::UnsupportedQueryPlan {
                error: "Projections with multiple tables are not supported yet",
            });
        }
        // Covers the case of tablelessness
        let input_table_ref = if let Some(table_ref) = input_table_refs.first() {
            table_ref.clone()
        } else {
            TableRef::from_names(None, "empty")
        };
        let current_accessor = input_schema
            .iter()
            .zip(input_eval.column_evals())
            .map(|(field, eval)| {
                (
                    ColumnRef::new(
                        input_table_ref.clone(),
                        field.name().clone(),
                        field.data_type(),
                    ),
                    *eval,
                )
            })
            .collect::<IndexMap<_, _>>();
        // 1. selection
        let where_eval = self
            .where_clause
            .verifier_evaluate(builder, accessor, input_chi_eval)?;
        // 2. columns
        let group_by_evals = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, accessor, input_chi_eval))
            .collect::<Result<Vec<_>, _>>()?;
        let aggregate_evals = self
            .sum_expr
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .verifier_evaluate(builder, accessor, input_chi_eval)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // 3. filtered_columns
        let group_by_result_columns_evals =
            builder.try_consume_final_round_mle_evaluations(self.group_by_exprs.len())?;
        let sum_result_columns_evals =
            builder.try_consume_final_round_mle_evaluations(self.sum_expr.len())?;
        let count_column_eval = builder.try_consume_final_round_mle_evaluation()?;

        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let output_chi_eval = builder.try_consume_chi_evaluation()?;

        verify_group_by(
            builder,
            alpha,
            beta,
            input_chi_eval,
            output_chi_eval,
            (group_by_evals, aggregate_evals, where_eval),
            (
                group_by_result_columns_evals.clone(),
                sum_result_columns_evals.clone(),
            ),
        )?;
        match result {
            Some(table) => {
                let cols = self
                    .group_by_exprs
                    .iter()
                    .map(|col| table.inner_table().get(&col.column_id()))
                    .collect::<Option<Vec<_>>>()
                    .ok_or(ProofError::VerificationError {
                        error: "Result does not all correct group by columns.",
                    })?;
                if (0..table.num_rows() - 1)
                    .any(|i| compare_indexes_by_owned_columns(&cols, i, i + 1).is_ge())
                {
                    Err(ProofError::VerificationError {
                        error: "Result of group by not ordered as expected.",
                    })?;
                }
            }
            None => {
                Err(ProofError::UnsupportedQueryPlan {
                    error: "AggregateExec currently only supported at top level of query plan.",
                })?;
            }
        }

        let column_evals = group_by_result_columns_evals
            .into_iter()
            .chain(sum_result_columns_evals)
            .chain(iter::once(count_column_eval))
            .collect::<Vec<_>>();
        Ok(TableEvaluation::new(column_evals, output_chi_eval))
    }

    #[expect(clippy::redundant_closure_for_method_calls)]
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.group_by_exprs
            .iter()
            .map(|col| col.get_column_field())
            .chain(self.sum_expr.iter().map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
            }))
            .chain(iter::once(ColumnField::new(
                self.count_alias.clone(),
                ColumnType::BigInt,
            )))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = self.input.get_column_references();

        for col in &self.group_by_exprs {
            columns.insert(col.get_column_reference());
        }
        for aliased_expr in &self.sum_expr {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

impl ProverEvaluate for AggregateExec {
    #[tracing::instrument(name = "AggregateExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        let input = self.input.first_round_evaluate(builder, alloc, table_map);
        // 1. selection
        let selection_column: Column<'a, S> = self.where_clause.result_evaluate(alloc, &input);

        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.result_evaluate(alloc, &input))
            .collect::<Vec<_>>();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| aliased_expr.expr.result_evaluate(alloc, &input))
            .collect::<Vec<_>>();
        // Compute filtered_columns
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
            ..
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, &[], &[], selection)
            .expect("columns should be aggregatable");
        let sum_result_columns_iter = sum_result_columns.iter().map(|col| Column::Scalar(col));
        let res = Table::<'a, S>::try_from_iter(
            self.get_column_result_fields()
                .into_iter()
                .map(|field| field.name())
                .zip(
                    group_by_result_columns
                        .into_iter()
                        .chain(sum_result_columns_iter)
                        .chain(iter::once(Column::BigInt(count_column))),
                ),
        )
        .expect("Failed to create table from column references");
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(count_column.len());

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "AggregateExec::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        let input = self.input.final_round_evaluate(builder, alloc, table_map);
        // 1. selection
        let selection_column: Column<'a, S> =
            self.where_clause.prover_evaluate(builder, alloc, &input);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.prover_evaluate(builder, alloc, &input))
            .collect::<Vec<_>>();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, &input))
            .collect::<Vec<_>>();
        // 3. Compute filtered_columns
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
            ..
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, &[], &[], selection)
            .expect("columns should be aggregatable");

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        // 4. Tally results
        let sum_result_columns_iter = sum_result_columns.iter().map(|col| Column::Scalar(col));
        let columns = group_by_result_columns
            .clone()
            .into_iter()
            .chain(sum_result_columns_iter)
            .chain(iter::once(Column::BigInt(count_column)));
        let res = Table::<'a, S>::try_from_iter(
            self.get_column_result_fields()
                .into_iter()
                .map(|field| field.name())
                .zip(columns.clone()),
        )
        .expect("Failed to create table from column references");
        // 5. Produce MLEs
        for column in columns {
            builder.produce_intermediate_mle(column);
        }
        // 6. Prove group by
        prove_group_by(
            builder,
            alloc,
            alpha,
            beta,
            (&group_by_columns, &sum_columns, selection),
            (&group_by_result_columns, &sum_result_columns, count_column),
            input.num_rows(),
        );

        log::log_memory_usage("End");

        res
    }
}

fn verify_group_by<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    input_chi_eval: S,
    output_chi_eval: S,
    (g_in_evals, sum_in_evals, sel_in_eval): (Vec<S>, Vec<S>, S),
    (g_out_evals, sum_out_evals, count_out_eval): (Vec<S>, Vec<S>, S),
) -> Result<(), ProofError> {
    // g_in_fold = alpha * sum beta^j * g_in[j]
    let g_in_fold_eval = alpha * fold_vals(beta, &g_in_evals);
    // g_out_fold = alpha * sum beta^j * g_out[j]
    let g_out_fold_eval = alpha * fold_vals(beta, &g_out_evals);
    // sum_in_fold = chi_n + sum beta^(j+1) * sum_in[j]
    let sum_in_fold_eval = input_chi_eval + beta * fold_vals(beta, &sum_in_evals);
    // sum_out_fold = count_out + sum beta^(j+1) * sum_out[j]
    let sum_out_fold_eval = count_out_eval + beta * fold_vals(beta, &sum_out_evals);

    let g_in_star_eval = builder.try_consume_final_round_mle_evaluation()?;
    let g_out_star_eval = builder.try_consume_final_round_mle_evaluation()?;

    // sum g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_fold = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        g_in_star_eval * sel_in_eval * sum_in_fold_eval - g_out_star_eval * sum_out_fold_eval,
        3,
    )?;

    // g_in_star + g_in_star * g_in_fold - chi_n = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        g_in_star_eval + g_in_star_eval * g_in_fold_eval - input_chi_eval,
        2,
    )?;

    // g_out_star + g_out_star * g_out_fold - chi_m = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        g_out_star_eval + g_out_star_eval * g_out_fold_eval - output_chi_eval,
        2,
    )?;

    Ok(())
}

pub fn prove_group_by<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    (g_in, sum_in, sel_in): (&[Column<S>], &[Column<S>], &'a [bool]),
    (g_out, sum_out, count_out): (&[Column<S>], &[&'a [S]], &'a [i64]),
    n: usize,
) {
    let m = count_out.len();
    let chi_n = alloc.alloc_slice_fill_copy(n, true);
    let chi_m = alloc.alloc_slice_fill_copy(m, true);

    // g_in_fold = alpha * sum beta^j * g_in[j]
    let g_in_fold = alloc.alloc_slice_fill_copy(n, Zero::zero());
    fold_columns(g_in_fold, alpha, beta, g_in);

    // g_out_fold = alpha * sum beta^j * g_out[j]
    let g_out_fold = alloc.alloc_slice_fill_copy(m, Zero::zero());
    fold_columns(g_out_fold, alpha, beta, g_out);

    // sum_in_fold = 1 + sum beta^(j+1) * sum_in[j]
    let sum_in_fold = alloc.alloc_slice_fill_copy(n, One::one());
    fold_columns(sum_in_fold, beta, beta, sum_in);

    // sum_out_fold = count_out + sum beta^(j+1) * sum_out[j]
    let sum_out_fold = alloc.alloc_slice_fill_default(m);
    slice_ops::slice_cast_mut(count_out, sum_out_fold);
    fold_columns(sum_out_fold, beta, beta, sum_out);

    // g_in_star = (1 + g_in_fold)^(-1)
    let g_in_star = alloc.alloc_slice_copy(g_in_fold);
    slice_ops::add_const::<S, S>(g_in_star, One::one());
    slice_ops::batch_inversion(g_in_star);

    // g_out_star = (1 + g_out_fold)^(-1)
    let g_out_star = alloc.alloc_slice_copy(g_out_fold);
    slice_ops::add_const::<S, S>(g_out_star, One::one());
    slice_ops::batch_inversion(g_out_star);

    builder.produce_intermediate_mle(g_in_star as &[_]);
    builder.produce_intermediate_mle(g_out_star as &[_]);

    // sum g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_fold = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (
                S::one(),
                vec![
                    Box::new(g_in_star as &[_]),
                    Box::new(sel_in),
                    Box::new(sum_in_fold as &[_]),
                ],
            ),
            (
                -S::one(),
                vec![Box::new(g_out_star as &[_]), Box::new(sum_out_fold as &[_])],
            ),
        ],
    );

    // g_in_star + g_in_star * g_in_fold - chi_n = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(g_in_star as &[_])]),
            (
                S::one(),
                vec![Box::new(g_in_star as &[_]), Box::new(g_in_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_n as &[_])]),
        ],
    );

    // g_out_star + g_out_star * g_out_fold - chi_m = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(g_out_star as &[_])]),
            (
                S::one(),
                vec![Box::new(g_out_star as &[_]), Box::new(g_out_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_m as &[_])]),
        ],
    );
}
