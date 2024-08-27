use super::{PostprocessingError, PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{compare_indexes_by_owned_columns_with_direction, OwnedColumn, OwnedTable},
    math::permutation::Permutation,
    scalar::Scalar,
};
use proof_of_sql_parser::intermediate_ast::{OrderBy, OrderByDirection};
use rayon::prelude::ParallelSliceMut;
use serde::{Deserialize, Serialize};

/// A node representing a list of `OrderBy` expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderByPostprocessing {
    by_exprs: Vec<OrderBy>,
}

impl OrderByPostprocessing {
    /// Create a new `OrderByPostprocessing` node.
    pub fn new(by_exprs: Vec<OrderBy>) -> Self {
        Self { by_exprs }
    }
}

impl<S: Scalar> PostprocessingStep<S> for OrderByPostprocessing {
    /// Apply the slice transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let mut indexes = (0..owned_table.num_rows()).collect::<Vec<_>>();
        // Evaluate the columns by which we order
        // Once we allow OrderBy for general aggregation-free expressions here we will need to call eval()
        let order_by_pairs: Vec<(OwnedColumn<S>, OrderByDirection)> = self
            .by_exprs
            .iter()
            .map(
                |order_by| -> PostprocessingResult<(OwnedColumn<S>, OrderByDirection)> {
                    Ok((
                        owned_table
                            .inner_table()
                            .get(&order_by.expr)
                            .ok_or(PostprocessingError::ColumnNotFound(
                                order_by.expr.to_string(),
                            ))?
                            .clone(),
                        order_by.direction,
                    ))
                },
            )
            .collect::<PostprocessingResult<Vec<(OwnedColumn<S>, OrderByDirection)>>>()?;
        // Define the ordering
        indexes.par_sort_unstable_by(|&a, &b| {
            compare_indexes_by_owned_columns_with_direction(&order_by_pairs, a, b)
        });
        let permutation = Permutation::unchecked_new(indexes);
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
