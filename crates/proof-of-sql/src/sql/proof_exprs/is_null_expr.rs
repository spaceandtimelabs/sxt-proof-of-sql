use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable IS NULL expression, evaluates to TRUE if the expression is NULL
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsNullExpr {
    expr: Box<DynProofExpr>,
}

impl IsNullExpr {
    /// Create a new IS NULL expression
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        Self { expr }
    }
}

impl ProofExpr for IsNullExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        // Evaluate the inner expression (for potential side-effects)
        let _ = self.expr.result_evaluate(alloc, table);
        // Get the presence slice directly for the expression
        let presence = table.presence_for_expr(&*self.expr);
        if presence.is_none() {
            // If no nulls, IS NULL is false for all rows
            return Column::Boolean(alloc.alloc_slice_fill_copy(table.num_rows(), false));
        }
        let presence_slice = presence.unwrap();
        // IS NULL is true where the presence indicator is true
        Column::Boolean(alloc.alloc_slice_fill_with(presence_slice.len(), |i| presence_slice[i]))
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let inner_column = self.expr.prover_evaluate(builder, alloc, table);
        // Obtain the presence slice directly
        let presence = table.presence_for_expr(&*self.expr);
        let result_slice = if presence.is_none() {
            // No nulls: IS NULL is false for all entries
            alloc.alloc_slice_fill_copy(table.num_rows(), false)
        } else {
            let presence_slice = presence.unwrap();
            // IS NULL is exactly the presence indicator
            alloc.alloc_slice_fill_with(presence_slice.len(), |i| presence_slice[i])
        };
        let nullable_column = match NullableColumn::with_presence(inner_column, presence) {
            Ok(col) => col,
            Err(err) => {
                tracing::warn!(
                    "IsNullExpr: Error creating NullableColumn: {:?}, assuming no NULLs",
                    err
                );
                NullableColumn::new(inner_column)
            }
        };
        // Record the IS NULL check in the proof
        builder.record_is_null_check(&nullable_column, alloc);
        if let Column::Boolean(inner_values) = inner_column {
            let is_not_null_slice = if presence.is_none() {
                // If no nulls, then every entry is not null
                alloc.alloc_slice_fill_copy(table.num_rows(), true)
            } else {
                let presence_slice = presence.unwrap();
                // NOT NULL is the negation of presence
                alloc.alloc_slice_fill_with(presence_slice.len(), |i| !presence_slice[i])
            };
            let invalid_state = alloc.alloc_slice_fill_with(table.num_rows(), |i| {
                is_not_null_slice[i] && inner_values[i]
            });
            // Add a constraint that if a value is not null, then the inner boolean must be false
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![(S::one(), vec![Box::new(&*invalid_state)])],
            );
        }
        Column::Boolean(result_slice)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        let inner_eval = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;
        let is_null_eval = builder.try_consume_final_round_mle_evaluation()?;
        let is_not_null_eval = chi_eval - is_null_eval;
        if self.expr.data_type() == ColumnType::Boolean {
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                is_not_null_eval * inner_eval,
                2,
            )?;
        }
        Ok(is_null_eval)
    }
}
