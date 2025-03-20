use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
    utils::log,
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable IS TRUE expression, evaluates to TRUE if the expression is both not NULL and TRUE
/// This is particularly useful for WHERE clauses in SQL that require boolean expressions to be TRUE
/// (not NULL and not FALSE)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsTrueExpr {
    expr: Box<DynProofExpr>,
    pub(crate) malicious: bool,
}

impl IsTrueExpr {
    /// Create a new IS TRUE expression
    ///
    /// # Panics
    /// Panics if the provided expression is not a boolean expression
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        assert!(
            expr.data_type() == ColumnType::Boolean,
            "IsTrueExpr can only be applied to boolean expressions, but got expression of type: {}",
            expr.data_type()
        );
        Self {
            expr,
            malicious: false,
        }
    }

    pub fn try_new(expr: Box<DynProofExpr>) -> Result<Self, ProofError> {
        if expr.data_type() != ColumnType::Boolean {
            return Err(ProofError::UnsupportedQueryPlan {
                error: "IsTrueExpr can only be applied to boolean expressions",
            });
        }
        Ok(Self {
            expr,
            malicious: false,
        })
    }

    pub fn is_inner_expr_or(&self) -> bool {
        let type_name = std::any::type_name_of_val(&*self.expr);
        type_name.contains("::Or")
    }
}

impl ProofExpr for IsTrueExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "IsTrueExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let inner_column = self.expr.result_evaluate(alloc, table);
        let inner_values = inner_column
            .as_boolean()
            .expect("Expression is not boolean");
        let mut column_refs = IndexSet::default();
        self.expr.get_column_references(&mut column_refs);

        if self.malicious {
            let result_slice = alloc.alloc_slice_fill_copy(table.num_rows(), true);
            let res = Column::Boolean(result_slice);
            log::log_memory_usage("End");
            return res;
        }

        let mut has_nullable_column = false;
        let mut combined_presence = vec![true; table.num_rows()];
        let presence_map = table.presence_map();

        for col_ref in &column_refs {
            let ident = col_ref.column_id();
            if let Some(col_presence) = presence_map.get(&ident) {
                has_nullable_column = true;
                for (i, &is_present) in col_presence.iter().enumerate() {
                    if !is_present {
                        combined_presence[i] = false;
                    }
                }
            }
        }

        let presence_slice = if has_nullable_column {
            alloc.alloc_slice_copy(&combined_presence)
        } else {
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        };

        let is_or_expr = self.is_inner_expr_or();
        let result_slice = alloc.alloc_slice_fill_with(inner_values.len(), |i| {
            if is_or_expr && inner_values[i] {
                true
            } else {
                inner_values[i] && presence_slice[i]
            }
        });

        let res = Column::Boolean(result_slice);
        log::log_memory_usage("End");
        res
    }

    #[tracing::instrument(name = "IsTrueExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let inner_column = self.expr.prover_evaluate(builder, alloc, table);
        let inner_values = inner_column
            .as_boolean()
            .expect("Expression is not boolean");
        let n = table.num_rows();

        if self.malicious {
            let result_slice = alloc.alloc_slice_fill_copy(n, true);
            builder.produce_intermediate_mle(Column::Boolean(result_slice));
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![(
                    S::one(),
                    vec![Box::new(alloc.alloc_slice_fill_copy(n, false) as &[_])],
                )],
            );

            let res = Column::Boolean(result_slice);
            log::log_memory_usage("End");
            return res;
        }

        let mut column_refs = IndexSet::default();
        self.expr.get_column_references(&mut column_refs);

        let mut has_nullable_column = false;
        let mut combined_presence = vec![true; n];
        let presence_map = table.presence_map();

        for col_ref in &column_refs {
            let ident = col_ref.column_id();
            if let Some(col_presence) = presence_map.get(&ident) {
                has_nullable_column = true;
                for (i, &is_present) in col_presence.iter().enumerate() {
                    if !is_present {
                        combined_presence[i] = false;
                    }
                }
            }
        }

        let presence_slice: &[bool] = if has_nullable_column {
            alloc.alloc_slice_copy(&combined_presence)
        } else {
            alloc.alloc_slice_fill_copy(n, true)
        };

        builder.produce_intermediate_mle(presence_slice);
        builder.produce_intermediate_mle(inner_values);

        let is_or_expr = self.is_inner_expr_or();
        let is_true_result: &[bool] = alloc.alloc_slice_fill_with(n, |i| {
            if is_or_expr && inner_values[i] {
                true
            } else {
                inner_values[i] && presence_slice[i]
            }
        });

        builder.produce_intermediate_mle(is_true_result);

        if is_or_expr {
            let or_logic_slice: &[bool] = alloc.alloc_slice_fill_with(n, |i| inner_values[i]);

            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![
                    (S::one(), vec![Box::new(is_true_result)]),
                    (-S::one(), vec![Box::new(or_logic_slice)]),
                ],
            );
        } else {
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![
                    (S::one(), vec![Box::new(is_true_result)]),
                    (
                        -S::one(),
                        vec![Box::new(presence_slice), Box::new(inner_values)],
                    ),
                ],
            );
        }

        let res = Column::Boolean(is_true_result);
        log::log_memory_usage("End");
        res
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        let _inner_eval = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;
        let presence_eval = builder.try_consume_final_round_mle_evaluation()?;
        let values_eval = builder.try_consume_final_round_mle_evaluation()?;
        let is_true_eval = builder.try_consume_final_round_mle_evaluation()?;

        let is_or_expr = self.is_inner_expr_or();
        if is_or_expr {
            let or_result = values_eval + (presence_eval * values_eval)
                - (values_eval * presence_eval * values_eval);
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                is_true_eval - or_result,
                1,
            )?;
        } else {
            let and_result = presence_eval * values_eval;
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                is_true_eval - and_result,
                2,
            )?;
        };

        Ok(is_true_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
