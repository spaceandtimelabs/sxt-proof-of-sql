use super::{PostprocessingError, PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{
        order_by_util::{
            compare_indexes_by_owned_columns_with_direction, OrderIndexDirectionPairs,
        },
        OwnedTable,
    },
    math::permutation::Permutation,
    scalar::Scalar,
};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

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
        // Define the ordering
        let permutation = Permutation::unchecked_new_from_cmp(owned_table.num_rows(), |&a, &b| {
            compare_indexes_by_owned_columns_with_direction(&column_direction_pairs, a, b)
        });
        // Apply the ordering
        Ok(
            OwnedTable::<S>::try_from_iter(owned_table.into_inner().into_iter().map(
                |(identifier, column)| {
                    (
                        identifier,
                        column
                            .try_permute(&permutation)
                            .expect("There should be no column length mismatch here"),
                    )
                },
            ))
            .expect("There should be no column length mismatch here"),
        )
    }
}
