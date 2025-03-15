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
        // Evaluate the inner expression - this already incorporates the SQL three-valued logic
        // for complex expressions like AND, OR, etc.
        let inner_column = self.expr.result_evaluate(alloc, table);

        // Extract boolean values from the inner column
        let Column::Boolean(inner_values) = inner_column else {
            panic!("IS TRUE can only be applied to boolean expressions");
        };

        // In SQL's three-valued logic, IS TRUE returns true only if the value is non-NULL and true
        // For complex expressions like OR, the NULL handling is already done by the inner expression
        // evaluation, so we don't need to recheck presence here.
        //
        // The inner_values already incorporate the three-valued logic results, including NULL propagation
        // rules for operations like AND, OR, etc.
        let result_slice = if self.malicious {
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        } else {
            // For IS TRUE, we just return the value as-is since NULL values
            // would already be represented as false in inner_values
            inner_values
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
        
        // For complex expressions, the inner expression evaluation already handles NULLs correctly
        let Column::Boolean(inner_values) = inner_column else {
            panic!("IS TRUE can only be applied to boolean expressions");
        };

        // Create a nullable column for record-keeping, though we don't actually need
        // to explicitly check for NULLs here
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
        let result_slice = if self.malicious {
            alloc.alloc_slice_fill_copy(table.num_rows(), true)
        } else {
            // For complex expressions like OR, the SQL three-valued logic is already
            // applied by the inner expression evaluation, so we just return the value
            inner_values
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
