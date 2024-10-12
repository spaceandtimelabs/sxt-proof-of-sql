<<<<<<< Updated upstream
use super::{fold_columns, fold_vals};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            group_by_util::{
                aggregate_columns, compare_indexes_by_owned_columns, AggregatedColumns,
            },
            Column, ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor,
            MetadataAccessor, OwnedTable,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, ProofExpr, TableExpr},
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::{iter, iter::repeat_with};
use num_traits::One;
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
///         SUM(<sum_expr1>.expr) as <sum_expr1>.alias, ..., SUM(<sum_exprN>.expr) as <sum_exprN>.alias,
///         COUNT(*) as count_alias
///     FROM <table>
///     WHERE <where_clause>
///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
/// ```
///
/// Note: if `group_by_exprs` is empty, then the query is equivalent to removing the `GROUP BY` clause.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupByExec<C: Commitment> {
    pub(super) group_by_exprs: Vec<ColumnExpr<C>>,
    pub(super) sum_expr: Vec<AliasedDynProofExpr<C>>,
    pub(super) count_alias: Identifier,
    pub(super) table: TableExpr,
    pub(super) where_clause: DynProofExpr<C>,
}

impl<C: Commitment> GroupByExec<C> {
    /// Creates a new `group_by` expression.
    pub fn new(
        group_by_exprs: Vec<ColumnExpr<C>>,
        sum_expr: Vec<AliasedDynProofExpr<C>>,
        count_alias: Identifier,
        table: TableExpr,
        where_clause: DynProofExpr<C>,
    ) -> Self {
        Self {
            group_by_exprs,
            sum_expr,
            count_alias,
            table,
            where_clause,
        }
    }
}

impl<C: Commitment> ProofPlan<C> for GroupByExec<C> {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in &self.group_by_exprs {
            expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        for aliased_expr in &self.sum_expr {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        // For the count col
        builder.count_intermediate_mles(1);
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
        result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        // 1. selection
        let where_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let group_by_evals = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        let aggregate_evals = self
            .sum_expr
            .iter()
            .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        // 3. filtered_columns

        let group_by_result_columns_evals: Vec<_> =
            repeat_with(|| builder.consume_intermediate_mle())
                .take(self.group_by_exprs.len())
                .collect();
        let sum_result_columns_evals: Vec<_> = repeat_with(|| builder.consume_intermediate_mle())
            .take(self.sum_expr.len())
            .collect();
        let count_column_eval = builder.consume_intermediate_mle();

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        verify_group_by(
            builder,
            alpha,
            beta,
            (group_by_evals, aggregate_evals, where_eval),
            (
                group_by_result_columns_evals.clone(),
                sum_result_columns_evals.clone(),
                count_column_eval,
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
            None => todo!("GroupByExec currently only supported at top level of query plan."),
        }

        Ok(group_by_result_columns_evals
            .into_iter()
            .chain(sum_result_columns_evals)
            .chain(iter::once(count_column_eval))
            .collect::<Vec<_>>())
    }

    #[allow(clippy::redundant_closure_for_method_calls)]
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.group_by_exprs
            .iter()
            .map(|col| col.get_column_field())
            .chain(self.sum_expr.iter().map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type())
            }))
            .chain(iter::once(ColumnField::new(
                self.count_alias,
                ColumnType::BigInt,
            )))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();

        for col in &self.group_by_exprs {
            columns.insert(col.get_column_reference());
        }
        for aliased_expr in &self.sum_expr {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for GroupByExec<C> {
    #[tracing::instrument(name = "GroupByExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        input_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause
                .result_evaluate(input_length, alloc, accessor);

        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.result_evaluate(input_length, alloc, accessor))
            .collect::<Vec<_>>();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .result_evaluate(input_length, alloc, accessor)
            })
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
        group_by_result_columns
            .into_iter()
            .chain(sum_result_columns_iter)
            .chain(iter::once(Column::BigInt(count_column)))
            .collect::<Vec<_>>()
    }

    fn first_round_evaluate(&self, builder: &mut FirstRoundBuilder) {
        builder.request_post_result_challenges(2);
    }

    #[tracing::instrument(name = "GroupByExec::final_round_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.prover_evaluate(builder, alloc, accessor))
            .collect::<Vec<_>>();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor))
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
        let res = group_by_result_columns
            .clone()
            .into_iter()
            .chain(sum_result_columns_iter)
            .chain(core::iter::once(Column::BigInt(count_column)))
            .collect::<Vec<_>>();
        // 5. Produce MLEs
        res.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        // 6. Prove group by
        prove_group_by(
            builder,
            alloc,
            alpha,
            beta,
            (&group_by_columns, &sum_columns, selection),
            (&group_by_result_columns, &sum_result_columns, count_column),
        );
        res
    }
}

fn verify_group_by<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    alpha: C::Scalar,
    beta: C::Scalar,
    (g_in_evals, sum_in_evals, sel_in_eval): (Vec<C::Scalar>, Vec<C::Scalar>, C::Scalar),
    (g_out_evals, sum_out_evals, count_out_eval): (Vec<C::Scalar>, Vec<C::Scalar>, C::Scalar),
) -> Result<(), ProofError> {
    let one_eval = builder.mle_evaluations.input_one_evaluation;

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
        SumcheckSubpolynomialType::ZeroSum,
        g_in_star_eval * sel_in_eval * sum_in_fold_eval - g_out_star_eval * sum_out_bar_fold_eval,
    );

    // g_in_star * g_in_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        g_in_star_eval * g_in_fold_eval - one_eval,
    );

    // g_out_star * g_out_bar_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        g_out_star_eval * g_out_bar_fold_eval - one_eval,
    );

    Ok(())
}

#[allow(
    clippy::missing_panics_doc,
    reason = "alpha is guaranteed to not be zero in this context"
)]
pub fn prove_group_by<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
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
=======
use super::{fold_columns, fold_vals};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            group_by_util::{
                aggregate_columns, compare_indexes_by_owned_columns, AggregatedColumns,
            },
            Column, ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor,
            MetadataAccessor, OwnedTable,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            CountBuilder, ProofBuilder, ProofPlan, ProverEvaluate, ResultBuilder,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, ProofExpr, TableExpr},
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::{iter, iter::repeat_with};
use num_traits::One;
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
///         SUM(<sum_expr1>.expr) as <sum_expr1>.alias, ..., SUM(<sum_exprN>.expr) as <sum_exprN>.alias,
///         COUNT(*) as count_alias
///     FROM <table>
///     WHERE <where_clause>
///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
/// ```
///
/// Note: if `group_by_exprs` is empty, then the query is equivalent to removing the `GROUP BY` clause.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupByExec<C: Commitment> {
    pub(super) group_by_exprs: Vec<ColumnExpr<C>>,
    pub(super) sum_expr: Vec<AliasedDynProofExpr<C>>,
    pub(super) count_alias: Identifier,
    pub(super) table: TableExpr,
    pub(super) where_clause: DynProofExpr<C>,
}

impl<C: Commitment> GroupByExec<C> {
    /// Creates a new `group_by` expression.
    pub fn new(
        group_by_exprs: Vec<ColumnExpr<C>>,
        sum_expr: Vec<AliasedDynProofExpr<C>>,
        count_alias: Identifier,
        table: TableExpr,
        where_clause: DynProofExpr<C>,
    ) -> Self {
        Self {
            group_by_exprs,
            sum_expr,
            count_alias,
            table,
            where_clause,
        }
    }
}

impl<C: Commitment> ProofPlan<C> for GroupByExec<C> {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in &self.group_by_exprs {
            expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        for aliased_expr in &self.sum_expr {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        // For the count col
        builder.count_intermediate_mles(1);
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
        result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        // 1. selection
        let where_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let group_by_evals = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        let aggregate_evals = self
            .sum_expr
            .iter()
            .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        // 3. filtered_columns

        let group_by_result_columns_evals: Vec<_> =
            repeat_with(|| builder.consume_intermediate_mle())
                .take(self.group_by_exprs.len())
                .collect();
        let sum_result_columns_evals: Vec<_> = repeat_with(|| builder.consume_intermediate_mle())
            .take(self.sum_expr.len())
            .collect();
        let count_column_eval = builder.consume_intermediate_mle();

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        verify_group_by(
            builder,
            alpha,
            beta,
            (group_by_evals, aggregate_evals, where_eval),
            (
                group_by_result_columns_evals.clone(),
                sum_result_columns_evals.clone(),
                count_column_eval,
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
            None => todo!("GroupByExec currently only supported at top level of query plan."),
        }

        Ok(group_by_result_columns_evals
            .into_iter()
            .chain(sum_result_columns_evals)
            .chain(iter::once(count_column_eval))
            .collect::<Vec<_>>())
    }

    #[allow(clippy::redundant_closure_for_method_calls)]
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.group_by_exprs
            .iter()
            .map(|col| col.get_column_field())
            .chain(self.sum_expr.iter().map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type())
            }))
            .chain(iter::once(ColumnField::new(
                self.count_alias,
                ColumnType::BigInt,
            )))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();

        for col in &self.group_by_exprs {
            columns.insert(col.get_column_reference());
        }
        for aliased_expr in &self.sum_expr {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for GroupByExec<C> {
    #[tracing::instrument(name = "GroupByExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        let input_length = accessor.get_length(self.table.table_ref);
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause
                .result_evaluate(input_length, alloc, accessor);

        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.result_evaluate(input_length, alloc, accessor))
            .collect::<Vec<_>>();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .result_evaluate(input_length, alloc, accessor)
            })
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
        builder.set_result_table_length(count_column.len());
        builder.request_post_result_challenges(2);
        group_by_result_columns
            .into_iter()
            .chain(sum_result_columns_iter)
            .chain(iter::once(Column::BigInt(count_column)))
            .collect::<Vec<_>>()
    }

    #[tracing::instrument(name = "GroupByExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.prover_evaluate(builder, alloc, accessor))
            .collect::<Vec<_>>();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor))
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
        let res = group_by_result_columns
            .clone()
            .into_iter()
            .chain(sum_result_columns_iter)
            .chain(core::iter::once(Column::BigInt(count_column)))
            .collect::<Vec<_>>();
        // 5. Produce MLEs
        res.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        // 6. Prove group by
        prove_group_by(
            builder,
            alloc,
            alpha,
            beta,
            (&group_by_columns, &sum_columns, selection),
            (&group_by_result_columns, &sum_result_columns, count_column),
        );
        res
    }
}

fn verify_group_by<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    alpha: C::Scalar,
    beta: C::Scalar,
    (g_in_evals, sum_in_evals, sel_in_eval): (Vec<C::Scalar>, Vec<C::Scalar>, C::Scalar),
    (g_out_evals, sum_out_evals, count_out_eval): (Vec<C::Scalar>, Vec<C::Scalar>, C::Scalar),
) -> Result<(), ProofError> {
    let one_eval = builder.mle_evaluations.input_one_evaluation;

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
        SumcheckSubpolynomialType::ZeroSum,
        g_in_star_eval * sel_in_eval * sum_in_fold_eval - g_out_star_eval * sum_out_bar_fold_eval,
    );

    // g_in_star * g_in_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        g_in_star_eval * g_in_fold_eval - one_eval,
    );

    // g_out_star * g_out_bar_fold - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        g_out_star_eval * g_out_bar_fold_eval - one_eval,
    );

    Ok(())
}

#[allow(
    clippy::missing_panics_doc,
    reason = "alpha is guaranteed to not be zero in this context"
)]
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
>>>>>>> Stashed changes
