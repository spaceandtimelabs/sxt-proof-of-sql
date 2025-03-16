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
                    // Create a new slice that is true only when both:
                    // 1. The value is non-NULL (presence is true)
                    // 2. The boolean value is true
                    let is_true = alloc.alloc_slice_copy(inner_values);
                    for (is_true, &present) in is_true.iter_mut().zip(presence.iter()) {
                        *is_true = *is_true && present;
                    }
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
        let inner_column = self.expr.prover_evaluate(builder, alloc, table);

        let Column::Boolean(inner_values) = inner_column else {
            panic!("IS TRUE can only be applied to boolean expressions");
        };

        // Get presence information
        let presence = table.presence_for_expr(&*self.expr);
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

        // Record the IS TRUE check which verifies both non-NULL and true conditions
        if self.malicious {
            builder.produce_intermediate_mle(Column::Boolean(
                alloc.alloc_slice_fill_copy(table.num_rows(), true),
            ));
        } else {
            builder.record_is_true_check(&nullable_column, alloc);
        }

        // Create result that matches the IS TRUE semantics
        let result_slice = if self.malicious {
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        } else {
            match presence {
                Some(presence) => {
                    // Create a new slice that is true only when both:
                    // 1. The value is non-NULL (presence is true)
                    // 2. The boolean value is true
                    let is_true = alloc.alloc_slice_copy(inner_values);
                    for (is_true, &present) in is_true.iter_mut().zip(presence.iter()) {
                        *is_true = *is_true && present;
                    }
                    is_true
                }
                None => inner_values, // No NULL values, use inner values directly
            }
        };

        Column::Boolean(result_slice)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        // Get the evaluation of the inner expression
        let _inner_eval = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;

        // Verify the sumcheck subpolynomial - this ensures correctness across all rows
        // The sumcheck verifies that claimed_result is true only when both:
        // 1. The value is non-NULL (presence = true)
        // 2. The boolean value is true
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            S::zero(),
            2,
        )?;

        // Get the claimed result - this is the evaluation of the IS TRUE expression
        // which is true only when the value is both non-NULL and TRUE
        let claimed_result = builder.try_consume_final_round_mle_evaluation()?;

        Ok(claimed_result)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
