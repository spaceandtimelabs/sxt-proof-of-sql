use super::{PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{OwnedColumn, OwnedTable},
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
            let values = nullable_result.values;

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
