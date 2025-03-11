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
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use tracing;

/// Provable IS NOT NULL expression, evaluates to TRUE if the expression is not NULL
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsNotNullExpr {
    expr: Box<DynProofExpr>,
}

impl IsNotNullExpr {
    /// Create a new IS NOT NULL expression
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        Self { expr }
    }
}

impl ProofExpr for IsNotNullExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        // Evaluate the inner expression
        let _inner_column = self.expr.result_evaluate(alloc, table);

        // Get the presence slice directly - this avoids creating a temporary NullableColumn
        // if we only need to check for nulls
        let presence = table.presence_for_expr(&*self.expr);

        // Create result boolean array - false if null, true if not null
        // Performance optimization: If presence is None, all values are non-null,
        // so we can just return a slice of all true values
        if presence.is_none() {
            // No nulls in the column, return all true values
            tracing::trace!("IsNotNullExpr: No nulls in column, returning all true values");
            return Column::Boolean(alloc.alloc_slice_fill_copy(table.num_rows(), true));
        }

        // We have a presence slice, so we need to check each value
        let presence_slice = presence.unwrap();

        // Create a new slice with negated values since presence[i]=true means NULL
        let result_slice =
            alloc.alloc_slice_fill_with(presence_slice.len(), |i| !presence_slice[i]);

        Column::Boolean(result_slice)
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        tracing::trace!("IsNotNullExpr: Starting prover evaluation");

        // Evaluate the inner expression
        let inner_column = self.expr.prover_evaluate(builder, alloc, table);

        // Get the presence slice directly - this avoids creating a temporary NullableColumn
        // if we only need to check for nulls
        let presence = table.presence_for_expr(&*self.expr);

        // Create result boolean array - false if null, true if not null
        // Performance optimization: If presence is None, all values are non-null,
        // so we can just return a slice of all true values
        let result_slice = if presence.is_none() {
            tracing::trace!("IsNotNullExpr: No nulls in column, returning all true values");
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        } else {
            let presence_slice = presence.unwrap();
            // Create a new slice with negated values since presence[i]=true means NULL
            alloc.alloc_slice_fill_with(presence_slice.len(), |i| !presence_slice[i])
        };

        // We still need to create a NullableColumn for the record_is_not_null_check operation
        let nullable_column = match NullableColumn::with_presence(inner_column, presence) {
            Ok(col) => col,
            Err(err) => {
                tracing::warn!(
                    "IsNotNullExpr: Error creating NullableColumn: {:?}, assuming no NULLs",
                    err
                );
                NullableColumn::new(inner_column)
            }
        };

        // Calculate is_null (the complement of is_not_null)
        let is_null_slice = if let Some(presence_slice) = presence {
            // is_null is the same as presence
            presence_slice
        } else {
            // No nulls, so is_null is all false
            alloc.alloc_slice_fill_copy(table.num_rows(), false)
        };

        // Record the IS NOT NULL operation in the proof
        builder.record_is_not_null_check(&nullable_column, alloc);

        // For boolean columns, we can add a constraint that when is_null is true, the inner value must be false
        if let Column::Boolean(inner_values) = inner_column {
            // Create a slice that is true when is_null is true and inner_value is true (which should never happen)
            let invalid_state = alloc
                .alloc_slice_fill_with(table.num_rows(), |i| is_null_slice[i] && inner_values[i]);

            // Add a constraint that invalid_state must be all false
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![(S::one(), vec![Box::new(&*invalid_state)])],
            );
        }

        Column::Boolean(result_slice)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        // Get the evaluation of the inner expression
        let inner_eval = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;

        // Get the is_not_null evaluation from the builder
        let is_not_null_eval = builder.try_consume_final_round_mle_evaluation()?;

        // Calculate the complement of is_not_null_eval (which represents is_null)
        let is_null_eval = chi_eval - is_not_null_eval;

        // For boolean columns, we verify that when is_null is true, the inner value must be false
        // This means is_null * inner_eval must be 0
        if self.expr.data_type() == ColumnType::Boolean {
            // Constraint: is_null_eval * inner_eval = 0
            // This ensures that if a value is null (is_null_eval = 1), then inner_eval must be 0
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                is_null_eval * inner_eval,
                2,
            )?;
        }

        // Return the is_not_null evaluation
        Ok(is_not_null_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
