use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
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
}

impl IsTrueExpr {
    /// Create a new IS TRUE expression
    ///
    /// # Panics
    /// Panics if the provided expression is not a boolean expression
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        // Validate that the expression is a boolean expression
        if expr.data_type() != ColumnType::Boolean {
            panic!("IsTrueExpr can only be applied to boolean expressions, but got expression of type: {}", expr.data_type());
        }
        Self { expr }
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
        Ok(Self { expr })
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
        let nullable_column = NullableColumn::with_presence(inner_column, table.presence_for_expr(&*self.expr))
            .unwrap_or_else(|_| {
                // If there's an error, assume no NULLs (all values present)
                NullableColumn::new(inner_column)
            });
        
        // Create result boolean array
        // For IS TRUE, we need both:
        // 1. Not NULL
        // 2. Value is TRUE
        let result_slice = alloc.alloc_slice_fill_with(table.num_rows(), |i| {
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
        let nullable_column = NullableColumn::with_presence(inner_column, table.presence_for_expr(&*self.expr))
            .unwrap_or_else(|_| {
                // If there's an error, assume no NULLs (all values present)
                NullableColumn::new(inner_column)
            });
        let result_slice = alloc.alloc_slice_fill_with(table.num_rows(), |i| {
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
        
        // Record the IS TRUE operation in the proof
        builder.record_is_true_check(&nullable_column, alloc);
        
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
        
        // Get the next value from the builder
        Ok(builder.try_consume_final_round_mle_evaluation()?)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
} 