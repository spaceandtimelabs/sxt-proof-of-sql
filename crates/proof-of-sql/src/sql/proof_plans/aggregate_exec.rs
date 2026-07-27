use super::{fold_columns, fold_vals, DynProofPlan};
use crate::{
    base::{
        database::{
            group_by_util::{aggregate_columns, AggregatedColumns},
            Column, ColumnField, ColumnRef, ColumnType, LiteralValue, Table, TableEvaluation,
            TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, ProofExpr},
        proof_gadgets::{
            final_round_evaluate_monotonic, first_round_evaluate_monotonic,
            fold_log_expr::FoldLogExpr, verify_monotonic,
        },
    },
    utils::log,
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::iter;
use num_traits::One;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use sqlparser::ast::Ident;
use tracing::{span, Level};

/// Errors returned when an aggregate proof plan cannot be constructed.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Snafu)]
pub enum AggregateExecError {
    /// Grouping and deduplication support at most one expression.
    #[snafu(display("grouping and deduplication support at most one expression, found {count}"))]
    UnsupportedGroupByExpressionCount {
        /// Number of grouping expressions found.
        count: usize,
    },
    /// The grouping expression has a datatype for which uniqueness cannot be proven.
    #[snafu(display("grouping and deduplication do not support expressions of type {data_type}"))]
    UnsupportedGroupByExpressionType {
        /// Unsupported grouping datatype.
        data_type: ColumnType,
    },
}

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <group_by_expr1>.expr as <group_by_expr1>.alias, ..., <group_by_exprM>.expr as <group_by_exprM>.alias,
///         SUM(<sum_expr1>.expr) as <sum_expr1>.alias, ..., SUM(<sum_exprN>.expr) as <sum_exprN>.alias,
///         COUNT(*) as <count_alias>
///     FROM <input>
///     WHERE <where_clause>
///     GROUP BY <group_by_expr1>.expr, ..., <group_by_exprM>.expr
/// ```
///
/// Note: if `group_by_exprs` is empty, then the query is equivalent to removing the `GROUP BY` clause.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AggregateExec {
    group_by_exprs: Vec<AliasedDynProofExpr>,
    sum_expr: Vec<AliasedDynProofExpr>,
    count_alias: Ident,
    input: Box<DynProofPlan>,
    where_clause: DynProofExpr,
}

impl AggregateExec {
    /// Creates a new aggregate proof plan.
    pub fn try_new(
        group_by_exprs: Vec<AliasedDynProofExpr>,
        sum_expr: Vec<AliasedDynProofExpr>,
        count_alias: Ident,
        input: Box<DynProofPlan>,
        where_clause: DynProofExpr,
    ) -> Result<Self, AggregateExecError> {
        let group_by = Self {
            group_by_exprs,
            sum_expr,
            count_alias,
            input,
            where_clause,
        };
        group_by.try_get_is_uniqueness_provable()?;
        Ok(group_by)
    }

    /// Get a reference to the input plan
    pub fn input(&self) -> &DynProofPlan {
        &self.input
    }

    /// Get a reference to the where clause
    pub fn where_clause(&self) -> &DynProofExpr {
        &self.where_clause
    }

    /// Get a reference to the group by expressions
    pub fn group_by_exprs(&self) -> &[AliasedDynProofExpr] {
        &self.group_by_exprs
    }

    /// Get a reference to the sum expressions
    pub fn sum_expr(&self) -> &[AliasedDynProofExpr] {
        &self.sum_expr
    }

    /// Get a reference to the count alias
    pub fn count_alias(&self) -> &Ident {
        &self.count_alias
    }

    /// Checks if the group by expression can prove uniqueness
    /// This is true if there is only one group by column and its type is not `VarChar` and not `VarBinary`
    pub fn try_get_is_uniqueness_provable(&self) -> Result<bool, AggregateExecError> {
        if let [group_by_expr] = self.group_by_exprs.as_slice() {
            let data_type = group_by_expr.expr.data_type();
            if matches!(data_type, ColumnType::VarChar | ColumnType::VarBinary) {
                Err(AggregateExecError::UnsupportedGroupByExpressionType { data_type })
            } else {
                Ok(true)
            }
        } else if self.group_by_exprs.is_empty() {
            Ok(false)
        } else {
            Err(AggregateExecError::UnsupportedGroupByExpressionCount {
                count: self.group_by_exprs.len(),
            })
        }
    }
}

impl ProofPlan for AggregateExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let input_eval = self
            .input
            .verifier_evaluate(builder, accessor, chi_eval_map, params)?;
        let input_chi_eval = input_eval.chi_eval();
        // Build new accessors
        let input_schema = self.input.get_column_result_fields();
        // Check for tables - this is just error handling, we don't need the table ref
        let accessor = input_schema
            .iter()
            .zip(input_eval.column_evals())
            .map(|(field, eval)| (field.name().clone(), *eval))
            .collect::<IndexMap<_, _>>();

        // Compute g_in_star
        let fold_gadget = FoldLogExpr::new(alpha, beta);
        let group_by_evals = self
            .group_by_exprs
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .verifier_evaluate(builder, &accessor, input_chi_eval, params)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let g_in_star_eval = fold_gadget
            .verify_evaluate(builder, &group_by_evals, input_chi_eval)?
            .0;
        // End compute g_in_star

        let where_eval =
            self.where_clause
                .verifier_evaluate(builder, &accessor, input_chi_eval, params)?;

        // Compute sum_in_fold
        let aggregate_evals = self
            .sum_expr
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .verifier_evaluate(builder, &accessor, input_chi_eval, params)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sum_in_fold_eval = input_chi_eval + beta * fold_vals(beta, &aggregate_evals);
        // End compute sum_in_fold

        let output_chi_eval = builder.try_consume_chi_evaluation()?;

        // 3. filtered_columns
        let group_by_result_columns_evals =
            builder.try_consume_first_round_mle_evaluations(self.group_by_exprs.len())?;
        let g_out_star_eval = fold_gadget
            .verify_evaluate(builder, &group_by_result_columns_evals, output_chi_eval.0)?
            .0;

        match self.try_get_is_uniqueness_provable() {
            Ok(true) => {
                verify_monotonic::<S, true, true>(
                    builder,
                    alpha,
                    beta,
                    group_by_result_columns_evals[0],
                    output_chi_eval.0,
                )?;
            }
            Ok(false) => (),
            Err(_) => {
                Err(ProofError::UnsupportedQueryPlan {
                error: "AggregateExec with nonzero grouping columns and without provable uniqueness check not supported.",
            })?;
            }
        }

        let sum_result_columns_evals =
            builder.try_consume_first_round_mle_evaluations(self.sum_expr.len() + 1)?;

        let sum_out_fold_eval = fold_vals(beta, &sum_result_columns_evals);

        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            g_in_star_eval * where_eval * sum_in_fold_eval - g_out_star_eval * sum_out_fold_eval,
            3,
        )?;

        let column_evals = group_by_result_columns_evals
            .into_iter()
            .chain(sum_result_columns_evals)
            .collect::<Vec<_>>();
        Ok(TableEvaluation::new(column_evals, output_chi_eval))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.group_by_exprs
            .iter()
            .map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
            })
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
        self.input.get_column_references()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

impl ProverEvaluate for AggregateExec {
    #[tracing::instrument(
        name = "AggregateExec::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        builder.request_post_result_challenges(2);

        let input = self
            .input
            .first_round_evaluate(builder, alloc, table_map, params)?;

        // Compute g_in_star
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .first_round_evaluate(alloc, &input, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // End compute g_in_star

        let selection_column: Column<'a, S> = self
            .where_clause
            .first_round_evaluate(alloc, &input, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // Compute sum_in_fold
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .first_round_evaluate(alloc, &input, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // End compute sum_in_fold

        // Compute filtered_columns
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
            ..
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, &[], &[], selection)
            .expect("columns should be aggregatable");
        for column in &group_by_result_columns {
            builder.produce_intermediate_mle(*column);
        }

        builder.produce_chi_evaluation_length(count_column.len());

        let sum_result_columns_iter = sum_result_columns
            .iter()
            .map(|col| Column::Scalar(col))
            .chain(iter::once(Column::BigInt(count_column)));
        let res = Table::<'a, S>::try_from_iter(
            self.get_column_result_fields()
                .into_iter()
                .map(|field| field.name())
                .zip(
                    group_by_result_columns
                        .iter()
                        .copied()
                        .chain(sum_result_columns_iter.clone()),
                ),
        )
        .expect("Failed to create table from column references");
        // Prove result uniqueness if possible
        if self
            .try_get_is_uniqueness_provable()
            .expect("Group by must be provable")
        {
            first_round_evaluate_monotonic(
                builder,
                alloc,
                alloc.alloc_slice_copy(&group_by_result_columns[0].to_scalar()),
            );
        }
        // Produce MLEs
        for column in sum_result_columns_iter {
            builder.produce_intermediate_mle(column);
        }

        log::log_memory_usage("End");

        Ok(res)
    }

    #[expect(clippy::too_many_lines)]
    #[tracing::instrument(
        name = "AggregateExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        let input = self
            .input
            .final_round_evaluate(builder, alloc, table_map, params)?;

        let n = input.num_rows();

        // Compute g_in_star
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, &input, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let fold_gadget = FoldLogExpr::new(alpha, beta);
        let g_in_star = fold_gadget
            .final_round_evaluate(builder, alloc, &group_by_columns, n)
            .0;
        // End compute g_in_star

        let selection_column: Column<'a, S> = self
            .where_clause
            .final_round_evaluate(builder, alloc, &input, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // Compute sum_in_fold
        let span = span!(
            Level::DEBUG,
            "AggregateExec::final_round_evaluate sum_columns"
        )
        .entered();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, &input, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        span.exit();

        let span = span!(
            Level::DEBUG,
            "AggregateExec::final_round_evaluate allocate sum_in_fold"
        )
        .entered();
        let sum_in_fold = alloc.alloc_slice_fill_copy(n, One::one());
        span.exit();

        fold_columns(sum_in_fold, beta, beta, &sum_columns);
        // End compute sum_in_fold

        // 3. Compute filtered_columns
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
            ..
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, &[], &[], selection)
            .expect("columns should be aggregatable");

        let m = count_column.len();

        let g_out_star = fold_gadget
            .final_round_evaluate(builder, alloc, &group_by_result_columns, m)
            .0;

        if self
            .try_get_is_uniqueness_provable()
            .expect("Group by must be provable")
        {
            let g_out_scalars = group_by_result_columns[0].to_scalar();
            let alloc_g_out_scalars = alloc.alloc_slice_copy(&g_out_scalars);
            final_round_evaluate_monotonic::<S, true, true>(
                builder,
                alloc,
                alpha,
                beta,
                alloc_g_out_scalars,
            );
        }

        // 4. Tally results
        let sum_result_columns_iter = sum_result_columns.iter().map(|col| Column::Scalar(col));
        let columns = group_by_result_columns
            .clone()
            .into_iter()
            .chain(sum_result_columns_iter.clone())
            .chain(iter::once(Column::BigInt(count_column)));
        let res = Table::<'a, S>::try_from_iter(
            self.get_column_result_fields()
                .into_iter()
                .map(|field| field.name())
                .zip(columns.clone()),
        )
        .expect("Failed to create table from column references");
        // 5. Prove group by

        let sum_out_fold = alloc.alloc_slice_fill_default(m);
        slice_ops::slice_cast_mut(count_column, sum_out_fold);
        fold_columns(sum_out_fold, beta, beta, &sum_result_columns);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::ZeroSum,
            vec![
                (
                    S::one(),
                    vec![
                        Box::new(g_in_star as &[_]),
                        Box::new(selection),
                        Box::new(sum_in_fold as &[_]),
                    ],
                ),
                (
                    -S::one(),
                    vec![Box::new(g_out_star as &[_]), Box::new(sum_out_fold as &[_])],
                ),
            ],
        );

        log::log_memory_usage("End");

        Ok(res)
    }
}
