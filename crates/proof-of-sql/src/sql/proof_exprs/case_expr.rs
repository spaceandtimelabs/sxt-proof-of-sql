use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{select_column, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString, vec, vec::Vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable `CASE WHEN c_1 THEN v_1 ... WHEN c_n THEN v_n ELSE v_else END` expression
/// with first-match-wins semantics.
///
/// For each arm we build a "guard" `g_i` that is 1 exactly on the rows where arm `i`
/// is the first true condition - its condition AND-ed with "no earlier arm won":
///
/// ```text
/// g_i = c_i AND NOT (g_1 OR ... OR g_{i-1})   which, since the guards are exclusive,
///     = c_i * (1 - g_1 - ... - g_{i-1})
/// ```
///
/// Exactly one guard is 1 per row (or none, meaning the else branch), so the result
/// is a plain selection:
///
/// ```text
/// r = g_1*v_1 + ... + g_n*v_n + (1 - g_1 - ... - g_n)*v_else
/// ```
///
/// The prover commits the guards and the result, and proves the two equations above.
/// A CASE with no WHEN arms is allowed and simply evaluates to the else branch.
///
/// Since `r` is always one branch's value, it keeps the branch type - no widening,
/// no overflow - and any type works, varchar included, because the prover picks
/// real values instead of doing arithmetic on them. Guards are just 0/1 columns and
/// cheap to commit, so only the single result is a full data column, however many
/// arms there are.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseExpr {
    when_thens: Vec<(DynProofExpr, DynProofExpr)>,
    else_expr: Box<DynProofExpr>,
}

/// The source index selected for `row`: the first arm whose condition is true, or
/// `else_source` if none match. Computed lazily per row so no per-row index column
/// is materialized.
fn winning_source(conditions: &[&[bool]], else_source: usize, row: usize) -> usize {
    conditions
        .iter()
        .position(|condition| condition[row])
        .unwrap_or(else_source)
}

impl CaseExpr {
    /// Create a CASE expression.
    ///
    /// Every condition must be boolean and every branch value (including ELSE) must
    /// have exactly the same type. The WHEN arms may be empty, in which case the
    /// expression evaluates to the else branch (whether to allow that is the
    /// planner's choice, not the plan's).
    pub fn try_new(
        when_thens: Vec<(DynProofExpr, DynProofExpr)>,
        else_expr: Box<DynProofExpr>,
    ) -> AnalyzeResult<Self> {
        let result_type = else_expr.data_type();
        for (condition, value) in &when_thens {
            if condition.data_type() != ColumnType::Boolean {
                return Err(AnalyzeError::DataTypeMismatch {
                    left_type: condition.data_type().to_string(),
                    right_type: ColumnType::Boolean.to_string(),
                });
            }
            if value.data_type() != result_type {
                return Err(AnalyzeError::DataTypeMismatch {
                    left_type: value.data_type().to_string(),
                    right_type: result_type.to_string(),
                });
            }
        }
        Ok(Self {
            when_thens,
            else_expr,
        })
    }
}

impl ProofExpr for CaseExpr {
    fn data_type(&self) -> ColumnType {
        self.else_expr.data_type()
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        // Evaluate conditions and their values together to keep the builder
        // operation order consistent with the verifier.
        let (conditions, mut sources): (Vec<&[bool]>, Vec<Column<'a, S>>) = self
            .when_thens
            .iter()
            .map(|(condition, value)| -> PlaceholderResult<_> {
                Ok((
                    condition
                        .first_round_evaluate(alloc, table, params)?
                        .as_boolean()
                        .expect("condition is not boolean"),
                    value.first_round_evaluate(alloc, table, params)?,
                ))
            })
            .collect::<PlaceholderResult<Vec<_>>>()?
            .into_iter()
            .unzip();
        // The else branch is the last source.
        sources.push(self.else_expr.first_round_evaluate(alloc, table, params)?);
        let else_source = self.when_thens.len();
        Ok(select_column(alloc, &sources, table.num_rows(), |row| {
            winning_source(&conditions, else_source, row)
        }))
    }

    #[tracing::instrument(name = "CaseExpr::final_round_evaluate", level = "info", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        // Evaluate conditions and their values together to keep the builder
        // operation order consistent with the verifier.
        let (condition_columns, mut sources): (Vec<Column<'a, S>>, Vec<Column<'a, S>>) = self
            .when_thens
            .iter()
            .map(|(condition, value)| -> PlaceholderResult<_> {
                Ok((
                    condition.final_round_evaluate(builder, alloc, table, params)?,
                    value.final_round_evaluate(builder, alloc, table, params)?,
                ))
            })
            .collect::<PlaceholderResult<Vec<_>>>()?
            .into_iter()
            .unzip();
        // The else branch is the last source.
        sources.push(
            self.else_expr
                .final_round_evaluate(builder, alloc, table, params)?,
        );

        let conditions: Vec<&[bool]> = condition_columns
            .iter()
            .map(|column| column.as_boolean().expect("condition is not boolean"))
            .collect();
        let num_rows = table.num_rows();
        let else_source = self.when_thens.len();

        // Guards. Each is committed and pinned by g_i = c_i * (1 - g_0 - ... - g_{i-1}),
        // which expands to g_i - c_i + c_i*g_0 + ... + c_i*g_{i-1} = 0.
        //
        // This produces O(arms^2) sumcheck terms (each guard references all earlier
        // ones), which is fine for the small arm counts CASE has in practice. A
        // prefix-product form (g_i = g_{i-1} * (1 - c_i)) would be O(arms).
        let mut guards: Vec<&'a [bool]> = Vec::with_capacity(condition_columns.len());
        for (arm, condition_column) in condition_columns.iter().enumerate() {
            let guard = alloc.alloc_slice_fill_with(num_rows, |row| {
                winning_source(&conditions, else_source, row) == arm
            }) as &[_];
            builder.produce_intermediate_mle(guard);
            let mut terms: Vec<(S, Vec<Box<_>>)> = vec![
                (S::one(), vec![Box::new(guard) as Box<_>]),
                (-S::one(), vec![Box::new(*condition_column) as Box<_>]),
            ];
            for prior in &guards {
                terms.push((
                    S::one(),
                    vec![
                        Box::new(*condition_column) as Box<_>,
                        Box::new(*prior) as Box<_>,
                    ],
                ));
            }
            builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomialType::Identity, terms);
            guards.push(guard);
        }

        // Result, pinned by r = sum g_i*v_i + (1 - sum g_i)*v_else, which expands
        // to r - v_else - sum(g_i*v_i) + sum(g_i*v_else) = 0.
        let result = select_column(alloc, &sources, num_rows, |row| {
            winning_source(&conditions, else_source, row)
        });
        let (else_column, value_columns) = sources
            .split_last()
            .expect("sources always has the else column");
        builder.produce_intermediate_mle(result);
        let mut terms: Vec<(S, Vec<Box<_>>)> = vec![
            (S::one(), vec![Box::new(result) as Box<_>]),
            (-S::one(), vec![Box::new(*else_column) as Box<_>]),
        ];
        for (guard, value_column) in guards.iter().zip(value_columns) {
            terms.push((
                -S::one(),
                vec![
                    Box::new(*guard) as Box<_>,
                    Box::new(*value_column) as Box<_>,
                ],
            ));
            terms.push((
                S::one(),
                vec![Box::new(*guard) as Box<_>, Box::new(*else_column) as Box<_>],
            ));
        }
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomialType::Identity, terms);

        log::log_memory_usage("End");

        Ok(result)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        // Evaluate conditions and their values together to mirror the prover's order.
        let (condition_evals, value_evals): (Vec<S>, Vec<S>) = self
            .when_thens
            .iter()
            .map(|(condition, value)| -> Result<_, ProofError> {
                Ok((
                    condition.verifier_evaluate(builder, accessor, chi_eval, params)?,
                    value.verifier_evaluate(builder, accessor, chi_eval, params)?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        let else_eval = self
            .else_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)?;

        // Mirror the prover's guard identities: each guard is read from the proof and
        // checked against g_i - c_i + c_i*(sum earlier) = 0. The bare -c_i term needs
        // no chi factor: every column is already 0 outside the table rows. `guard_sum`
        // accumulates g_0 + ... + g_{i-1} as we go.
        let mut guard_evals = Vec::with_capacity(condition_evals.len());
        let mut guard_sum = S::ZERO;
        for condition_eval in &condition_evals {
            let guard_eval = builder.try_consume_final_round_mle_evaluation()?;
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                guard_eval - *condition_eval + *condition_eval * guard_sum,
                2,
            )?;
            guard_evals.push(guard_eval);
            guard_sum += guard_eval;
        }

        // Mirror the result identity: r - v_else - sum(g_i*v_i) + (sum g_i)*v_else = 0.
        // With guards this is degree 2 (the g_i*v_i products); with no arms it is just
        // r - v_else, which is degree 1.
        let result_eval = builder.try_consume_final_round_mle_evaluation()?;
        let selected_sum: S = guard_evals
            .iter()
            .zip(&value_evals)
            .map(|(guard, value)| *guard * *value)
            .sum();
        let result_degree = if guard_evals.is_empty() { 1 } else { 2 };
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            result_eval - selected_sum - else_eval + guard_sum * else_eval,
            result_degree,
        )?;

        Ok(result_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        for (condition, value) in &self.when_thens {
            condition.get_column_references(columns);
            value.get_column_references(columns);
        }
        self.else_expr.get_column_references(columns);
    }
}
