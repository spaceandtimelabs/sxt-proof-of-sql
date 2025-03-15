use super::{PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{OwnedNullableColumn, OwnedColumn, OwnedTable},
    map::IndexMap,
    scalar::Scalar,
};
use alloc::vec::Vec;
use proof_of_sql_parser::intermediate_ast::AliasedResultExpr;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// The select expression used to select, reorder, and apply alias transformations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectPostprocessing {
    /// The aliased result expressions we select
    aliased_result_exprs: Vec<AliasedResultExpr>,
}

impl SelectPostprocessing {
    /// Create a new `SelectPostprocessing` node.
    #[must_use]
    pub fn new(aliased_result_exprs: Vec<AliasedResultExpr>) -> Self {
        Self {
            aliased_result_exprs,
        }
    }
}

impl<S: Scalar> PostprocessingStep<S> for SelectPostprocessing {
    /// Apply the select transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        // Collect all columns and their presence information
        let mut cols: IndexMap<Ident, OwnedColumn<S>> = IndexMap::default();
        let mut presence_info: IndexMap<Ident, Vec<bool>> = IndexMap::default();
        
        // Process each expression
        for aliased_expr in &self.aliased_result_exprs {
            // Evaluate the expression to get a nullable column
            let nullable_result = owned_table.evaluate_nullable(&aliased_expr.expr)?;
            let alias: Ident = aliased_expr.alias.into();
            
            // Get the values and presence information
            let mut values = nullable_result.values;
            
            // Check if this is a simple column reference or an expression
            let is_simple_column_ref = match &*aliased_expr.expr {
                proof_of_sql_parser::intermediate_ast::Expression::Column(_) => true,
                _ => false,
            };
            
            // For arithmetic expressions, zero out NULL values
            // For direct column references, preserve original values
            if !is_simple_column_ref && nullable_result.presence.is_some() {
                if let Some(presence_vec) = nullable_result.presence.clone() {
                    // Zero out values where NULL is present (presence = false) for numeric columns in expressions
                    match &mut values {
                        OwnedColumn::BigInt(vals) => {
                            for (i, present) in presence_vec.iter().enumerate() {
                                if !present {
                                    vals[i] = 0;
                                }
                            }
                        },
                        OwnedColumn::Int(vals) => {
                            for (i, present) in presence_vec.iter().enumerate() {
                                if !present {
                                    vals[i] = 0;
                                }
                            }
                        },
                        OwnedColumn::Int128(vals) => {
                            for (i, present) in presence_vec.iter().enumerate() {
                                if !present {
                                    vals[i] = 0;
                                }
                            }
                        },
                        OwnedColumn::Decimal75(_, _, vals) => {
                            for (i, present) in presence_vec.iter().enumerate() {
                                if !present {
                                    vals[i] = S::ZERO;
                                }
                            }
                        },
                        OwnedColumn::Scalar(vals) => {
                            for (i, present) in presence_vec.iter().enumerate() {
                                if !present {
                                    vals[i] = S::ZERO;
                                }
                            }
                        },
                        // Don't modify non-numeric columns
                        _ => {}
                    }
                }
            }
            
            // Store the column values
            cols.insert(alias.clone(), values);
            
            // If the expression result includes NULL values, store the presence information
            if let Some(presence_vec) = nullable_result.presence {
                presence_info.insert(alias, presence_vec);
            }
        }
        
        // Create a new table with the columns
        let mut result_table = OwnedTable::try_new(cols)?;
        
        // Add the presence information to the table
        for (ident, presence) in presence_info {
            result_table.set_presence(ident, presence);
        }
        
        Ok(result_table)
    }
}
