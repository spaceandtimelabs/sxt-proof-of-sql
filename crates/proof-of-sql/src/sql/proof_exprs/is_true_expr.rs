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

        // Since we need to create a nullable column to check for null values,
        // we need to create a temporary NullableColumn with the presence slice
        let nullable_column =
            NullableColumn::with_presence(inner_column, table.presence_for_expr(&*self.expr))
                .unwrap_or_else(|_| {
                    // If there's an error, assume no NULLs (all values present)
                    NullableColumn::new(inner_column)
                });

        // Create result boolean array
        // For IS TRUE, we need both:
        // 1. Not NULL
        // 2. Value is TRUE
        let result_slice = alloc.alloc_slice_fill_with(table.num_rows(), |i| {
            if self.malicious {
                return true;
            }
            if nullable_column.is_null(i) {
                false // NULL values are never TRUE
            } else {
                // Check if the value is true
                match nullable_column.values {
                    Column::Boolean(values) => values[i],
                    _ => panic!("IS TRUE can only be applied to boolean expressions"),
                }
            }
        });

        Column::Boolean(result_slice)
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let inner_column = self.expr.prover_evaluate(builder, alloc, table);
        let presence = table.presence_for_expr(&*self.expr);

        let Column::Boolean(inner_values) = inner_column else {
            panic!("IS TRUE can only be applied to boolean expressions");
        };

        // Create a nullable column using the original presence
        let nullable_column = match NullableColumn::with_presence(inner_column, presence) {
            Ok(col) => col,
            Err(err) => {
                tracing::warn!(
                    "IsTrueExpr: Error creating NullableColumn: {:?}, assuming no NULLs",
                    err
                );
                NullableColumn::new(inner_column)
            }
        };

        // When malicious, produce fake MLE with all true values
        // Otherwise, record the proper IS TRUE check
        if self.malicious {
            builder.produce_intermediate_mle(Column::Boolean(
                alloc.alloc_slice_fill_copy(table.num_rows(), true),
            ));
        } else {
            builder.record_is_true_check(&nullable_column, alloc);
        }

        // Compute result (which may be incorrect if malicious=true)
        let result_slice = alloc.alloc_slice_fill_with(table.num_rows(), |i| {
            if self.malicious {
                true
            } else {
                // presence[i] = false means NULL, so !presence[i] means IS NULL
                // presence[i] = true means NOT NULL, so presence[i] means IS NOT NULL
                // For a value to be TRUE, it must be NOT NULL and have a true value
                let not_null = presence.is_none_or(|p| p[i]);
                not_null && inner_values[i]
            }
        });

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

        // Verify the sumcheck subpolynomial - this ensures correctness across all rows
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            S::zero(),
            2,
        )?;

        // Get the claimed result and verify it matches inner evaluation
        let claimed_result = builder.try_consume_final_round_mle_evaluation()?;
        if claimed_result != inner_eval {
            return Err(ProofError::VerificationError {
                error: "IS TRUE verification failed: result does not match inner evaluation",
            });
        }

        Ok(inner_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
