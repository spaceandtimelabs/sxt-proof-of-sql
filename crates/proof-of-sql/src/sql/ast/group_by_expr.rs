use super::{
    aggregate_columns, fold_columns, fold_vals,
    group_by_util::{compare_indexes_by_owned_columns, AggregatedColumns},
    AliasedProvableExprPlan, ColumnExpr, ProvableExpr, ProvableExprPlan, TableExpr,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor,
            MetadataAccessor, OwnedTable,
        },
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{
        CountBuilder, Indexes, ProofBuilder, ProofExpr, ProverEvaluate, ResultBuilder,
        SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use bumpalo::Bump;
use core::iter::repeat_with;
use num_traits::One;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_exprs[1].expr> as <result_expr[1].alias>, ..., <result_exprs[K].expr> as <result_exprs[K].alias>,
///         SUM(<sum_exprs[1].expr>) as <sum_exprs[1].alias>, ..., SUM(<sum_exprs[M].expr>) as <sum_exprs[M].alias>,
///         COUNT(<count_exprs[1].expr>) as <count_exprs[1].alias>, ..., COUNT(<count_exprs[N].expr>) as <count_exprs[N].alias>
///     FROM <table>
///     WHERE <where_clause>
///     GROUP BY <group_by_exprs[1]>, ..., <group_by_exprs[L]>
/// ```
///
/// Note:
/// 1. If `group_by_exprs` is empty, then the query is equivalent to removing the `GROUP BY` clause.
/// 2. Result expressions must only contain columns that are in the `group_by_exprs`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupByExpr<C: Commitment> {
    pub(super) group_by_exprs: Vec<ColumnExpr<C>>,
    pub(super) result_exprs: Vec<AliasedProvableExprPlan<C>>,
    pub(super) sum_exprs: Vec<AliasedProvableExprPlan<C>>,
    pub(super) count_exprs: Vec<AliasedProvableExprPlan<C>>,
    pub(super) table: TableExpr,
    pub(super) where_clause: ProvableExprPlan<C>,
}

impl<C: Commitment> GroupByExpr<C> {
    /// Creates a new group_by expression.
    pub fn new(
        group_by_exprs: Vec<ColumnExpr<C>>,
        result_exprs: Vec<AliasedProvableExprPlan<C>>,
        sum_exprs: Vec<AliasedProvableExprPlan<C>>,
        count_exprs: Vec<AliasedProvableExprPlan<C>>,
        table: TableExpr,
        where_clause: ProvableExprPlan<C>,
    ) -> Self {
        Self {
            group_by_exprs,
            result_exprs,
            sum_exprs,
            table,
            count_exprs,
            where_clause,
        }
    }
}

impl<C: Commitment> ProofExpr<C> for GroupByExpr<C> {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in self.group_by_exprs.iter() {
            expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        for aliased_expr in self.sum_exprs.iter() {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        // We need at least one count column for the proof
        // so if none is in result we need to temporarily add one
        let count_expr_count = self.count_exprs.len().max(1);
        builder.count_intermediate_mles(count_expr_count);
        builder.count_intermediate_mles(2);
        builder.count_subpolynomials(3);
        builder.count_degree(3);
        builder.count_post_result_challenges(2);
        for aliased_expr in self.result_exprs.iter() {
            aliased_expr.expr.count(builder)?;
            builder.count_result_columns(1);
        }
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
        result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<(), ProofError> {
        // 1. selection
        let where_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let group_by_evals = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        let aggregate_evals = self
            .sum_exprs
            .iter()
            .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        let result_evals = self
            .result_exprs
            .iter()
            .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        // 3. indexes
        let indexes_eval = builder
            .mle_evaluations
            .result_indexes_evaluation
            .ok_or(ProofError::VerificationError("invalid indexes"))?;
        // 4. filtered_columns
        let group_by_result_columns_evals = Vec::from_iter(
            repeat_with(|| builder.consume_intermediate_mle()).take(self.group_by_exprs.len()),
        );
        let sum_result_columns_evals = Vec::from_iter(
            repeat_with(|| builder.consume_intermediate_mle()).take(self.sum_exprs.len()),
        );
        let count_column_evals = Vec::from_iter(
            repeat_with(|| builder.consume_intermediate_mle()).take(self.count_exprs.len()),
        );
        let count_column_eval = if self.count_exprs.is_empty() {
            builder.consume_intermediate_mle()
        } else {
            count_column_evals[0]
        };

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        verify_group_by(
            builder,
            alpha,
            beta,
            (group_by_evals, aggregate_evals, where_eval),
            (
                group_by_result_columns_evals,
                sum_result_columns_evals,
                count_column_eval,
            ),
        )?;
        match result {
            // Check for uniqueness
            Some(table) => {
                let cols = self
                    .group_by_exprs
                    .iter()
                    .map(|col| table.inner_table().get(&col.column_id()))
                    .collect::<Option<Vec<_>>>()
                    .ok_or(ProofError::VerificationError(
                        "Result does not all correct group by columns.",
                    ))?;
                if (0..table.num_rows() - 1)
                    .any(|i| compare_indexes_by_owned_columns(&cols, i, i + 1).is_ge())
                {
                    Err(ProofError::VerificationError(
                        "Result of group by not ordered as expected.",
                    ))?;
                }
            }
            None => todo!("GroupByExpr currently only supported at top level of query plan."),
        }
        // 5. set result columns
        let result_columns_evals = Vec::from_iter(
            repeat_with(|| builder.consume_result_mle()).take(self.result_exprs.len()),
        );
        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.result_exprs
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> HashSet<ColumnRef> {
        let mut columns = HashSet::new();

        // No need to add columns from result_exprs since they are already in group_by_exprs
        for col in self.group_by_exprs.iter() {
            columns.insert(col.get_column_reference());
        }
        for aliased_expr in self.sum_exprs.iter() {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for GroupByExpr<C> {
    #[tracing::instrument(name = "GroupByExpr::result_evaluate", level = "debug", skip_all)]
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
        let group_by_columns = Vec::from_iter(
            self.group_by_exprs
                .iter()
                .map(|expr| expr.result_evaluate(builder.table_length(), alloc, accessor)),
        );
        let sum_columns = Vec::from_iter(self.sum_exprs.iter().map(|aliased_expr| {
            aliased_expr
                .expr
                .result_evaluate(builder.table_length(), alloc, accessor)
        }));
        // 3. set result columns
        let result_columns = Vec::from_iter(self.result_exprs.iter().map(|aliased_expr| {
            aliased_expr
                .expr
                .result_evaluate(builder.table_length(), alloc, accessor)
        }));
        // Compute filtered_columns and indexes
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, selection)
            .expect("columns should be aggregatable");
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(count_column.len() as u64)));
        // 4. set filtered_columns
        // for col in sum_result_columns {
        //     builder.produce_result_column(col);
        // }
        // for _ in self.count_exprs.iter() {
        //     builder.produce_result_column(count_column);
        // }

        for col in result_columns {
            builder.produce_result_column(col);
        }
        builder.request_post_result_challenges(2);
    }

    #[tracing::instrument(name = "GroupByExpr::prover_evaluate", level = "debug", skip_all)]
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
        let group_by_columns = Vec::from_iter(
            self.group_by_exprs
                .iter()
                .map(|expr| expr.prover_evaluate(builder, alloc, accessor)),
        );
        let sum_columns = Vec::from_iter(
            self.sum_exprs
                .iter()
                .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor)),
        );
        // Compute filtered_columns and indexes
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            result_columns: result_result_columns,
            sum_columns: sum_result_columns,
            count_column,
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, selection)
            .expect("columns should be aggregatable");

        for raw_col in &group_by_result_columns {
            let scalars = raw_col.to_scalar_with_scaling(0);
            let col = alloc.alloc_slice_copy(&scalars);
            builder.produce_intermediate_mle(col as &[_]);
        }
        for col in &sum_result_columns {
            builder.produce_intermediate_mle(col as &[_]);
        }
        for _ in self.count_exprs.iter() {
            builder.produce_intermediate_mle(count_column as &[_]);
        }
        // Add additional MLEs for the count column if none is in result
        if self.count_exprs.is_empty() {
            builder.produce_intermediate_mle(count_column as &[_]);
        }

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        prove_group_by(
            builder,
            alloc,
            alpha,
            beta,
            (&group_by_columns, &sum_columns, selection),
            (&group_by_result_columns, &sum_result_columns, count_column),
        );
        // 3. set result columns
        let result_columns = Vec::from_iter(
            self.result_exprs
                .iter()
                .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor)),
        );
    }
}

fn verify_group_by<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    alpha: C::Scalar,
    beta: C::Scalar,
    (g_in_evals, sum_in_evals, sel_in_eval): (Vec<C::Scalar>, Vec<C::Scalar>, C::Scalar),
    (g_out_evals, sum_out_evals, count_out_eval): (Vec<C::Scalar>, Vec<C::Scalar>, C::Scalar),
) -> Result<(), ProofError> {
    let one_eval = builder.mle_evaluations.one_evaluation;
    let rand_eval = builder.mle_evaluations.random_evaluation;

    // g_in_fold = alpha + sum beta^j * g_in[j]
    let g_in_fold_eval = alpha * one_eval + fold_vals(beta, &g_in_evals);
    // g_out_bar_fold = alpha + sum beta^j * g_out_bar[j]
    let g_out_bar_fold_eval = alpha * one_eval + fold_vals(beta, &g_out_evals);
    // sum_in_fold = 1 + sum beta^(j+1) * sum_in[j]
    let sum_in_fold_eval = one_eval + beta * fold_vals(beta, &sum_in_evals);
    // sum_out_bar_fold = count_out_bar + sum beta^(j+1) * sum_out_bar[j]
    let sum_out_bar_fold_eval = count_out_eval + beta * fold_vals(beta, &sum_out_evals);

    let g_in_star_eval = builder.consume_intermediate_mle();
    let g_out_star_eval = builder.consume_intermediate_mle();

    // sum g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_bar_fold = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &(g_in_star_eval * sel_in_eval * sum_in_fold_eval
            - g_out_star_eval * sum_out_bar_fold_eval),
    );

    // g_in_star * g_in_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &(rand_eval * (g_in_star_eval * g_in_fold_eval - one_eval)),
    );

    // g_out_star * g_out_bar_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &(rand_eval * (g_out_star_eval * g_out_bar_fold_eval - one_eval)),
    );

    Ok(())
}

pub fn prove_group_by<'a, S: Scalar>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    (g_in, sum_in, sel_in): (&[Column<S>], &[Column<S>], &'a [bool]),
    (g_out, sum_out, count_out): (&[Column<S>], &[&'a [S]], &'a [i64]),
) {
    let n = builder.table_length();
    let m_out = count_out.len();

    // g_in_fold = alpha + sum beta^j * g_in[j]
    let g_in_fold = alloc.alloc_slice_fill_copy(n, alpha);
    fold_columns(g_in_fold, One::one(), beta, g_in);

    // g_out_bar_fold = alpha + sum beta^j * g_out_bar[j]
    let g_out_bar_fold = alloc.alloc_slice_fill_copy(n, alpha);
    fold_columns(g_out_bar_fold, One::one(), beta, g_out);

    // sum_in_fold = 1 + sum beta^(j+1) * sum_in[j]
    let sum_in_fold = alloc.alloc_slice_fill_copy(n, One::one());
    fold_columns(sum_in_fold, beta, beta, sum_in);

    // sum_out_bar_fold = count_out_bar + sum beta^(j+1) * sum_out_bar[j]
    let sum_out_bar_fold = alloc.alloc_slice_fill_default(n);
    slice_ops::slice_cast_mut(count_out, sum_out_bar_fold);
    fold_columns(sum_out_bar_fold, beta, beta, sum_out);

    // g_in_star = g_in_fold^(-1)
    let g_in_star = alloc.alloc_slice_copy(g_in_fold);
    slice_ops::batch_inversion(g_in_star);

    // g_out_star = g_out_bar_fold^(-1), which is simply alpha^(-1) when beyond the output length
    let g_out_star = alloc.alloc_slice_copy(g_out_bar_fold);
    g_out_star[m_out..].fill(alpha.inv().expect("alpha should never be 0"));
    slice_ops::batch_inversion(&mut g_out_star[..m_out]);

    builder.produce_intermediate_mle(g_in_star as &[_]);
    builder.produce_intermediate_mle(g_out_star as &[_]);

    // sum g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_bar_fold = 0
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
                vec![
                    Box::new(g_out_star as &[_]),
                    Box::new(sum_out_bar_fold as &[_]),
                ],
            ),
        ],
    );

    // g_in_star * g_in_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(g_in_star as &[_]), Box::new(g_in_fold as &[_])],
            ),
            (-S::one(), vec![]),
        ],
    );

    // g_out_star * g_out_bar_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![
                    Box::new(g_out_star as &[_]),
                    Box::new(g_out_bar_fold as &[_]),
                ],
            ),
            (-S::one(), vec![]),
        ],
    );
}
