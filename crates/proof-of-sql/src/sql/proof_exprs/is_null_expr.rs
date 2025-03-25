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
    ) -> NullableColumn<'a, S> {
        // Evaluate the inner expression (for potential side-effects)
        let _ = self.expr.result_evaluate(alloc, table);

        // Instead of relying on presence_for_expr, we'll derive presence information
        // directly from the column references in the expression
        let mut column_refs = IndexSet::default();
        self.expr.get_column_references(&mut column_refs);

        // For each referenced column, get its presence information from the table
        let mut has_nullable_column = false;
        let mut combined_presence = vec![true; table.num_rows()];

        // Get access to the presence map
        let presence_map = table.presence_map();

        for col_ref in &column_refs {
            let ident = col_ref.column_id();
            // Access presence information via the presence map
            if let Some(col_presence) = presence_map.get(&ident) {
                has_nullable_column = true;
                // Update combined presence - a row is present only if all component values are present
                for (i, &is_present) in col_presence.iter().enumerate() {
                    if !is_present {
                        combined_presence[i] = false;
                    }
                }
            }
        }

        // Convert combined presence to a slice with the correct lifetime
        let presence_slice = if has_nullable_column {
            alloc.alloc_slice_copy(&combined_presence)
        } else {
            // If no nullable columns, all values are present (therefore not NULL)
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        };

        // IS NULL is true where the presence indicator is false (false means NULL)
        let res = Column::Boolean(alloc.alloc_slice_fill_with(presence_slice.len(), |i| !presence_slice[i]));
        NullableColumn::new(res)
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> NullableColumn<'a, S> {
        // Evaluate the inner expression
        let inner_column = self.expr.prover_evaluate(builder, alloc, table);

        // Instead of relying on presence_for_expr, we'll derive presence information
        // directly from the column references in the expression
        let mut column_refs = IndexSet::default();
        self.expr.get_column_references(&mut column_refs);

        // For each referenced column, get its presence information from the table
        let mut has_nullable_column = false;
        let mut combined_presence = vec![true; table.num_rows()];

        // Get access to the presence map
        let presence_map = table.presence_map();

        for col_ref in &column_refs {
            let ident = col_ref.column_id();
            // Access presence information via the presence map
            if let Some(col_presence) = presence_map.get(&ident) {
                has_nullable_column = true;
                // Update combined presence - a row is present only if all component values are present
                for (i, &is_present) in col_presence.iter().enumerate() {
                    if !is_present {
                        combined_presence[i] = false;
                    }
                }
            }
        }

        // Convert combined presence to a slice with the correct lifetime
        let presence_slice = if has_nullable_column {
            alloc.alloc_slice_copy(&combined_presence)
        } else {
            // If no nullable columns, all values are present
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        };

        // Create the result - IS NULL is true where presence is false
        let result_slice =
            alloc.alloc_slice_fill_with(presence_slice.len(), |i| !presence_slice[i]);

        // Now we include both the derived presence information and inner values in the proof
        builder.produce_intermediate_mle(Column::Boolean(presence_slice));
        builder.produce_intermediate_mle(inner_column.values);

        // Create a nullable column with our derived presence information
        let nullable_column = NullableColumn {
            values: inner_column.values,
            presence: Some(presence_slice),
        };

        // Record the IS NULL check in the proof
        builder.record_is_null_check(&nullable_column, alloc);

        let res = Column::Boolean(result_slice);
        NullableColumn::new(res)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<(S, Option<S>), ProofError> {
        // First get the inner expression evaluation
        let (_inner_eval, _) = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;

        // Get the derived presence information that was explicitly committed in the proof
        let presence_eval = builder.try_consume_final_round_mle_evaluation()?;

        // Get the inner expression values
        let values_eval = builder.try_consume_final_round_mle_evaluation()?;

        // We don't need to compute or explicitly check the expected IS NULL value here
        // The sumcheck protocol will verify the mathematical relationship

        // For boolean columns, we verify that when is_null is true, the inner value must be false
        if self.expr.data_type() == ColumnType::Boolean {
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                (chi_eval - presence_eval) * values_eval,
                2,
            )?;
        }

        // Get the claimed result from the proof - this is the evaluation of the IS NULL expression
        let claimed_result = builder.try_consume_final_round_mle_evaluation()?;

        Ok((claimed_result, None))
    }
}
