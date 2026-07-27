use super::{fold_columns, fold_vals};
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
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, ProofExpr, TableExpr},
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
use sqlparser::ast::Ident;
use tracing::{span, Level};

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
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GroupByExec {
    pub(super) group_by_exprs: Vec<ColumnExpr>,
    pub(super) sum_expr: Vec<AliasedDynProofExpr>,
    pub(super) count_alias: Ident,
    pub(super) table: TableExpr,
    pub(super) where_clause: DynProofExpr,
}

impl GroupByExec {
    /// Creates a new `group_by` expression.
    pub fn try_new(
        group_by_exprs: Vec<ColumnExpr>,
        sum_expr: Vec<AliasedDynProofExpr>,
        count_alias: Ident,
        table: TableExpr,
        where_clause: DynProofExpr,
    ) -> Option<Self> {
        let group_by = Self {
            group_by_exprs,
            sum_expr,
            count_alias,
            table,
            where_clause,
        };
        group_by.try_get_is_uniqueness_provable().map(|_| group_by)
    }

    /// Get a reference to the table expression
    pub fn table(&self) -> &TableExpr {
        &self.table
    }

    /// Get a reference to the where clause
    pub fn where_clause(&self) -> &DynProofExpr {
        &self.where_clause
    }

    /// Get a reference to the group by expressions
    pub fn group_by_exprs(&self) -> &[ColumnExpr] {
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
    pub fn try_get_is_uniqueness_provable(&self) -> Option<bool> {
        match (
            self.group_by_exprs.len(),
            self.group_by_exprs.first().map(ColumnExpr::data_type),
        ) {
            (0, _) => Some(false),
            (1, Some(data_type))
                if !matches!(data_type, ColumnType::VarChar | ColumnType::VarBinary) =>
            {
                Some(true)
            }
            _ => None,
        }
    }
}

impl ProofPlan for GroupByExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let input_chi_eval = chi_eval_map
            .get(&self.table.table_ref)
            .expect("Chi eval not found")
            .0;
        let accessor = accessor
            .get(&self.table.table_ref)
            .cloned()
            .unwrap_or_else(|| [].into_iter().collect());

        // Compute g_in_star
        let fold_gadget = FoldLogExpr::new(alpha, beta);
        let group_by_evals = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, &accessor, input_chi_eval, params))
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
            Some(true) => {
                verify_monotonic::<S, true, true>(
                    builder,
                    alpha,
                    beta,
                    group_by_result_columns_evals[0],
                    output_chi_eval.0,
                )?;
            }
            Some(false) => (),
            None => {
                Err(ProofError::UnsupportedQueryPlan {
                error: "GroupByExec with nonzero grouping columns and without provable uniqueness check not supported.",
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

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref.clone()])
    }
}

impl ProverEvaluate for GroupByExec {
    #[tracing::instrument(name = "GroupByExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        builder.request_post_result_challenges(2);

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");

        // Compute g_in_star
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| -> PlaceholderResult<Column<'a, S>> {
                expr.first_round_evaluate(alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // End compute g_in_star

        let selection_column: Column<'a, S> = self
            .where_clause
            .first_round_evaluate(alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // Compute sum_in_fold
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr.expr.first_round_evaluate(alloc, table, params)
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

    #[tracing::instrument(name = "GroupByExec::final_round_evaluate", level = "debug", skip_all)]
    #[expect(clippy::too_many_lines)]
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

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");

        let n = table.num_rows();

        // Compute g_in_star
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| -> PlaceholderResult<Column<'a, S>> {
                expr.final_round_evaluate(builder, alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let fold_gadget = FoldLogExpr::new(alpha, beta);
        let g_in_star = fold_gadget
            .final_round_evaluate(builder, alloc, &group_by_columns, n)
            .0;
        // End compute g_in_star

        let selection_column: Column<'a, S> = self
            .where_clause
            .final_round_evaluate(builder, alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // Compute sum_in_fold
        let span = span!(
            Level::DEBUG,
            "GroupByExec::final_round_evaluate sum_columns"
        )
        .entered();
        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        span.exit();

        let span = span!(
            Level::DEBUG,
            "GroupByExec::final_round_evaluate allocate sum_in_fold"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::database::{ColumnRef, ColumnType, LiteralValue, TableRef},
        sql::{
            proof::ProofPlan,
            proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
        },
    };
    use alloc::{vec, vec::Vec};
    use sqlparser::ast::Ident;

    fn column_ref(table_ref: &TableRef, name: &str, column_type: ColumnType) -> ColumnRef {
        ColumnRef::new(table_ref.clone(), Ident::new(name), column_type)
    }

    fn column_expr(table_ref: &TableRef, name: &str, column_type: ColumnType) -> ColumnExpr {
        ColumnExpr::new(column_ref(table_ref, name, column_type))
    }

    fn dyn_column(table_ref: &TableRef, name: &str, column_type: ColumnType) -> DynProofExpr {
        DynProofExpr::new_column(column_ref(table_ref, name, column_type))
    }

    fn sum_expr(table_ref: &TableRef) -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: dyn_column(table_ref, "amount", ColumnType::BigInt),
            alias: Ident::new("total_amount"),
        }
    }

    fn table_expr(table_ref: &TableRef) -> TableExpr {
        TableExpr {
            table_ref: table_ref.clone(),
        }
    }

    fn group_by_plan(table_ref: &TableRef) -> GroupByExec {
        GroupByExec::try_new(
            vec![column_expr(table_ref, "customer_id", ColumnType::BigInt)],
            vec![sum_expr(table_ref)],
            Ident::new("row_count"),
            table_expr(table_ref),
            dyn_column(table_ref, "is_billable", ColumnType::Boolean),
        )
        .unwrap()
    }

    #[test]
    fn group_by_accessors_and_references_work_without_blitzar() {
        let table_ref = TableRef::new("sxt", "orders");
        let plan = group_by_plan(&table_ref);

        assert_eq!(plan.table(), &table_expr(&table_ref));
        assert_eq!(plan.where_clause().data_type(), ColumnType::Boolean);
        assert_eq!(plan.group_by_exprs().len(), 1);
        assert_eq!(plan.sum_expr().len(), 1);
        assert_eq!(plan.count_alias(), &Ident::new("row_count"));

        let result_fields = plan.get_column_result_fields();
        let field_names = result_fields
            .iter()
            .map(ColumnField::name)
            .collect::<Vec<_>>();
        assert_eq!(
            field_names,
            vec![
                Ident::new("customer_id"),
                Ident::new("total_amount"),
                Ident::new("row_count"),
            ]
        );
        assert_eq!(result_fields[0].data_type(), ColumnType::BigInt);
        assert_eq!(result_fields[2].data_type(), ColumnType::BigInt);

        let column_refs = plan.get_column_references();
        assert!(column_refs.contains(&column_ref(&table_ref, "customer_id", ColumnType::BigInt)));
        assert!(column_refs.contains(&column_ref(&table_ref, "amount", ColumnType::BigInt)));
        assert!(column_refs.contains(&column_ref(&table_ref, "is_billable", ColumnType::Boolean)));

        let table_refs = plan.get_table_references();
        assert_eq!(table_refs.len(), 1);
        assert!(table_refs.contains(&table_ref));
    }

    #[test]
    fn group_by_uniqueness_gating_matches_supported_group_shapes() {
        let table_ref = TableRef::new("sxt", "orders");

        let ungrouped = GroupByExec::try_new(
            vec![],
            vec![sum_expr(&table_ref)],
            Ident::new("row_count"),
            table_expr(&table_ref),
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        )
        .unwrap();
        assert_eq!(ungrouped.try_get_is_uniqueness_provable(), Some(false));

        let numeric_group = group_by_plan(&table_ref);
        assert_eq!(numeric_group.try_get_is_uniqueness_provable(), Some(true));

        let varchar_group = GroupByExec::try_new(
            vec![column_expr(
                &table_ref,
                "customer_name",
                ColumnType::VarChar,
            )],
            vec![],
            Ident::new("row_count"),
            table_expr(&table_ref),
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        );
        assert!(varchar_group.is_none());

        let multi_group = GroupByExec::try_new(
            vec![
                column_expr(&table_ref, "customer_id", ColumnType::BigInt),
                column_expr(&table_ref, "amount", ColumnType::BigInt),
            ],
            vec![],
            Ident::new("row_count"),
            table_expr(&table_ref),
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        );
        assert!(multi_group.is_none());
    }
}
