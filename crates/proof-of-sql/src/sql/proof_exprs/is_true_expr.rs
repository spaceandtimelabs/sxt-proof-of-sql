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

        // Compute the not-null indicator: if no presence then all entries are not null, else invert the presence slice
        let not_null = if presence.is_none() {
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        } else {
            let presence_slice = presence.unwrap();
            alloc.alloc_slice_fill_with(presence_slice.len(), |i| !presence_slice[i])
        };

        // Compute result as the product (logical AND) of not_null and the inner boolean value
        let result_slice = if let Column::Boolean(inner_values) = inner_column {
            alloc.alloc_slice_fill_with(table.num_rows(), |i| {
                if self.malicious {
                    true
                } else {
                    not_null[i] && inner_values[i]
                }
            })
        } else {
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

        // Record the IS TRUE check in the proof
        if self.malicious {
            builder.produce_intermediate_mle(Column::Boolean(
                alloc.alloc_slice_fill_copy(table.num_rows(), true),
            ));
        } else {
            builder.record_is_true_check(&nullable_column, alloc);
        }

        // For boolean expressions, enforce algebraically that the result equals (not_null * inner_value)
        if let Column::Boolean(inner_values) = inner_column {
            let expected =
                alloc.alloc_slice_fill_with(table.num_rows(), |i| not_null[i] && inner_values[i]);
            let mismatch = alloc.alloc_slice_fill_with(table.num_rows(), |i| {
                if self.malicious {
                    false
                } else {
                    result_slice[i] != expected[i]
                }
            });
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![(S::one(), vec![Box::new(&*mismatch)])],
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
        let _inner_eval = self.expr.verifier_evaluate(builder, accessor, chi_eval)?;

        // Enforce final consistency check by producing the sumcheck subpolynomial evaluation which should yield zero
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            S::zero(),
            2,
        )?;

        // Now, consume the final round MLE evaluation for IS TRUE
        Ok(builder.try_consume_final_round_mle_evaluation()?)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
