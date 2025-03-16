use super::{PostprocessingError, PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{
        order_by_util::OrderIndexDirectionPairs,
        OwnedTable, OwnedColumn,
    },
    math::permutation::Permutation,
    scalar::{Scalar, ScalarExt},
};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::base::map::IndexMap;
use core::cmp::Ordering;

/// A node representing a list of `OrderBy` expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderByPostprocessing {
    index_direction_pairs: OrderIndexDirectionPairs,
}

impl OrderByPostprocessing {
    /// Create a new `OrderByPostprocessing` node.
    #[must_use]
    pub fn new(index_direction_pairs: OrderIndexDirectionPairs) -> Self {
        Self {
            index_direction_pairs,
        }
    }
}

impl<S: Scalar> PostprocessingStep<S> for OrderByPostprocessing {
    /// Apply the slice transformation to the given `OwnedTable`.
    #[allow(clippy::too_many_lines)]
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let opt_max_index = self
            .index_direction_pairs
            .iter()
            .map(|(index, _)| index)
            .max();
        if let Some(max_index) = opt_max_index {
            if *max_index >= owned_table.num_columns() {
                return Err(PostprocessingError::IndexOutOfBounds { index: *max_index });
            }
        }
        
        // Extract column names for the order-by columns to retrieve presence vectors later
        let order_by_column_names: Vec<_> = self.index_direction_pairs
            .iter()
            .map(|(index, _)| {
                owned_table.column_names().nth(*index).unwrap().clone()
            })
            .collect();
        
        // Create pairs of (Column, Direction) as before
        let column_direction_pairs = self
            .index_direction_pairs
            .iter()
            .map(|(index, direction)| {
                (
                    owned_table
                        .column_by_index(*index)
                        .expect("The index should be valid here")
                        .clone(),
                    *direction,
                )
            })
            .collect::<Vec<_>>();
        
        // Create a map of presence vectors for the columns we're ordering by
        let presence_for_order_by: IndexMap<_, _> = order_by_column_names
            .iter()
            .enumerate()
            .filter_map(|(i, ident)| {
                // Only include columns that have NULL values
                owned_table.get_presence(ident).map(|presence| {
                    (i, presence.clone())
                })
            })
            .collect();
        
        // Define a custom comparison function that handles NULL values according to SQL standard
        let compare_with_nulls = |a: &usize, b: &usize| {
            for (idx, (col, is_asc)) in column_direction_pairs.iter().enumerate() {
                // Check if either value is NULL
                let a_is_null = presence_for_order_by.get(&idx)
                    .is_some_and(|presence| !presence[*a]);
                
                let b_is_null = presence_for_order_by.get(&idx)
                    .is_some_and(|presence| !presence[*b]);
                
                match (a_is_null, b_is_null) {
                    // Both NULL - continue to next column
                    (true, true) => continue,
                    
                    // a is NULL, b is not NULL
                    // In ASC: NULL first (a < b) -> Less
                    // In DESC: NULL last (a > b) -> Greater
                    (true, false) => return if *is_asc { Ordering::Less } else { Ordering::Greater },
                    
                    // a is not NULL, b is NULL
                    // In ASC: NULL first (a > b) -> Greater
                    // In DESC: NULL last (a < b) -> Less
                    (false, true) => return if *is_asc { Ordering::Greater } else { Ordering::Less },
                    
                    // Neither is NULL, compare the values normally
                    (false, false) => {
                        let ordering = match col {
                            OwnedColumn::Boolean(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::Uint8(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::TinyInt(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::SmallInt(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::Int(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => {
                                col[*a].cmp(&col[*b])
                            }
                            OwnedColumn::Int128(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::Decimal75(_, _, col) => col[*a].signed_cmp(&col[*b]),
                            OwnedColumn::Scalar(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::VarChar(col) => col[*a].cmp(&col[*b]),
                            OwnedColumn::VarBinary(col) => col[*a].cmp(&col[*b]),
                        };
                        
                        // Apply direction to ordering
                        let dir_ordering = if *is_asc { ordering } else { ordering.reverse() };
                        
                        // If not equal, return the result
                        match dir_ordering {
                            Ordering::Equal => {},  // Continue to next column
                            _ => return dir_ordering
                        }
                    }
                }
            }
            
            // If all columns are equal (or all NULL), maintain original order
            Ordering::Equal
        };
        
        // Define the ordering using our custom comparator
        let permutation = Permutation::unchecked_new_from_cmp(owned_table.num_rows(), compare_with_nulls);
        
        // Extract the table and presence information
        let table_map = owned_table.inner_table().clone();
        let mut presence_map = IndexMap::default();
        
        // Collect all presence information
        for (ident, _) in &table_map {
            if let Some(presence) = owned_table.get_presence(ident) {
                // Clone the presence info for each column that has it
                presence_map.insert(ident.clone(), presence.clone());
            }
        }
        
        // Apply the permutation to both the columns and presence info
        let columns_iter = table_map.into_iter().map(|(identifier, column)| {
            (
                identifier.clone(),
                column
                    .try_permute(&permutation)
                    .expect("There should be no column length mismatch here"),
            )
        });
        
        // Create a new table with the permuted columns
        let mut new_table = OwnedTable::<S>::try_from_iter(columns_iter)
            .expect("There should be no columns with differing lengths here");
        
        // Apply the same permutation to presence vectors and set them in the new table
        for (ident, presence) in presence_map {
            // Apply the permutation to the entire presence vector
            match permutation.try_apply(&presence) {
                Ok(new_presence) => {
                    // Update the new table with the permuted presence info
                    new_table.set_presence(ident, new_presence);
                },
                Err(err) => {
                    // This shouldn't happen since we're using the same permutation that was used for columns
                    // But in case it does, we can just skip this presence vector
                    eprintln!("Failed to apply permutation to presence vector: {err}");
                }
            }
        }
        
        Ok(new_table)
    }
}
