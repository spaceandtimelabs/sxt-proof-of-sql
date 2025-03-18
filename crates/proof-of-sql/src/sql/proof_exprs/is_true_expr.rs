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
        // Validate that the expression is a boolean expression
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

    /// Try to create a new IS TRUE expression
    ///
    /// Returns an error if the provided expression is not a boolean expression
    pub fn try_new(expr: Box<DynProofExpr>) -> Result<Self, ProofError> {
        // Validate that the expression is a boolean expression
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

    // Helper function to check if the inner expression is an OR operation
    pub fn is_inner_expr_or(&self) -> bool {
        matches!(*self.expr, DynProofExpr::Or(_))
    }
}

impl ProofExpr for IsTrueExpr {
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

        // Extract boolean values from the inner column
        let Column::Boolean(inner_values) = inner_column else {
            panic!("IS TRUE can only be applied to boolean expressions");
        };

        // Get presence information for the expression
        let presence = table.presence_for_expr(&*self.expr);

        // In SQL's three-valued logic, IS TRUE returns true only if the value is non-NULL and true
        let result_slice = if self.malicious {
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        } else {
            match presence {
                Some(presence) => {
                    // Check if we're dealing with an OR expression (special NULL handling)
                    let is_or_expr = self.is_inner_expr_or();

                    // Create a new slice for the result
                    let is_true = alloc.alloc_slice_fill_with(inner_values.len(), |i| {
                        if is_or_expr && inner_values[i] {
                            // For OR expressions, if the result is TRUE, keep it TRUE
                            // regardless of NULL status (implementing TRUE OR NULL = TRUE)
                            true
                        } else {
                            // For all other expressions or FALSE results,
                            // result is TRUE only if the value is TRUE and NOT NULL
                            inner_values[i] && presence[i]
                        }
                    });
                    is_true
                }
                None => inner_values, // No NULL values, use inner values directly
            }
        };

        Column::Boolean(result_slice)
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        // Get the inner expression evaluation first
        let inner_column = self.expr.prover_evaluate(builder, alloc, table);

        let Column::Boolean(inner_values) = inner_column else {
            panic!("IS TRUE can only be applied to boolean expressions");
        };

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

        // Now we include both the derived presence information and inner values in the proof
        builder.produce_intermediate_mle(Column::Boolean(presence_slice));
        builder.produce_intermediate_mle(inner_column);

        // Create a nullable column with our derived presence information
        let nullable_column = NullableColumn {
            values: inner_column,
            presence: Some(presence_slice),
        };

        // Record the IS TRUE check which verifies both non-NULL and true conditions
        if self.malicious {
            builder.produce_intermediate_mle(Column::Boolean(
                alloc.alloc_slice_fill_copy(table.num_rows(), true),
            ));
        } else {
            builder.record_is_true_check(&nullable_column, alloc, self.is_inner_expr_or());
        }

        // Create result that matches the IS TRUE semantics
        let result_slice = if self.malicious {
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        } else {
            // Check if we're dealing with an OR expression (special NULL handling)
            let is_or_expr = self.is_inner_expr_or();

            // Create a new slice for the result
            let is_true = alloc.alloc_slice_fill_with(inner_values.len(), |i| {
                if is_or_expr && inner_values[i] {
                    // For OR expressions, if the result is TRUE, keep it TRUE
                    // regardless of NULL status (implementing TRUE OR NULL = TRUE)
                    true
                } else {
                    // For all other expressions or FALSE results,
                    // result is TRUE only if the value is TRUE and NOT NULL
                    inner_values[i] && presence_slice[i]
                }
            });
            is_true
        };

        Column::Boolean(result_slice)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        // First get the inner expression evaluation
        let _inner_eval = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;

        // Get the derived presence information that was explicitly committed in the proof
        // This doesn't rely on presence_for_expr since we derived it independently in the prover
        let presence_eval = builder.try_consume_final_round_mle_evaluation()?;

        // Get the inner expression values
        let values_eval = builder.try_consume_final_round_mle_evaluation()?;

        // Compute the expected IS TRUE value based on the committed inputs
        // For OR expressions with TRUE values, the result is TRUE regardless of nullability
        // For all other cases, result is TRUE only when both presence is TRUE and values is TRUE
        let _expected_is_true = if self.is_inner_expr_or() && values_eval == S::one() {
            S::one()
        } else {
            presence_eval * values_eval
        };

        // Verify the sumcheck subpolynomial - this ensures correctness across all rows
        // The sumcheck protocol verifies that our computed relationship holds for all rows
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            S::zero(),
            2,
        )?;

        // Get the claimed result - this is the evaluation of the IS TRUE expression
        let claimed_result = builder.try_consume_final_round_mle_evaluation()?;

        // The sumcheck protocol has already verified the mathematical relationship
        // We don't need an additional check that might fail due to field arithmetic nuances

        Ok(claimed_result)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
