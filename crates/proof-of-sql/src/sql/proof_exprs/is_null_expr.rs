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
use alloc::vec;
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

    /// Helper function to create a nullable column and evaluate IS NULL condition
    /// This reduces code duplication between `result_evaluate` and `prover_evaluate`
    fn create_is_null_column<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        inner_column: Column<'a, S>,
    ) -> (Column<'a, S>, NullableColumn<'a, S>) {
        // Create a nullable column with the presence slice
        let nullable_column =
            NullableColumn::with_presence(inner_column, table.presence_for_expr(&*self.expr))
                .unwrap_or_else(|_| {
                    // If there's an error, assume no NULLs (all values present)
                    NullableColumn::new(inner_column)
                });

        // Create result boolean array - true if null, false if not null
        let result_slice =
            alloc.alloc_slice_fill_with(table.num_rows(), |i| nullable_column.is_null(i));

        (Column::Boolean(result_slice), nullable_column)
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
        // Evaluate the inner expression
        let inner_column = self.expr.result_evaluate(alloc, table);

        // Use the helper function to create the result column and discard the nullable column
        self.create_is_null_column(alloc, table, inner_column).0
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let inner_column = self.expr.prover_evaluate(builder, alloc, table);
        let (result, nullable_column) = self.create_is_null_column(alloc, table, inner_column);

        // Record the IS NULL operation in the proof
        builder.record_is_null_check(&nullable_column, alloc);

        // For boolean columns, we can add a constraint that when is_null is true, the inner value must be false
        if let Column::Boolean(inner_values) = inner_column {
            // Get the is_null slice (presence slice)
            let is_null_slice = if let Some(presence) = &nullable_column.presence {
                presence
            } else {
                // If presence is None, all values are non-null, so is_null is all false
                // Convert the mutable slice to an immutable reference to match the presence type
                &*alloc.alloc_slice_fill_copy(table.num_rows(), false)
            };

            // Create a slice that is true when is_null is true and inner_value is true (which should never happen)
            let invalid_state = alloc
                .alloc_slice_fill_with(table.num_rows(), |i| is_null_slice[i] && inner_values[i]);

            // Add a constraint that invalid_state must be all false
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![(S::one(), vec![Box::new(&*invalid_state)])],
            );
        }

        result
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
        // Get the evaluation of the inner expression
        let inner_eval = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;

        // Get the is_null evaluation from the builder
        let is_null_eval = builder.try_consume_final_round_mle_evaluation()?;

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

        // Return the is_null evaluation
        Ok(is_null_eval)
    }
}
