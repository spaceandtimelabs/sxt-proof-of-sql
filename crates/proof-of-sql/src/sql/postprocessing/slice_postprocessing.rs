use super::{PostprocessingError, PostprocessingResult, PostprocessingStep};
use crate::base::{database::OwnedTable, scalar::Scalar};
use serde::{Deserialize, Serialize};

/// A `SlicePostprocessing` represents a slice of an `OwnedTable`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlicePostprocessing {
    /// number of rows to return
    ///
    /// - if None, specify all rows
    number_rows: Option<u64>,

    /// number of rows to skip
    ///
    /// - if None, specify the first row as starting point
    /// - if Some(nonnegative), specify the offset from the beginning
    /// - if Some(negative), specify the offset from the end
    ///   (e.g. -1 is the last row, -2 is the second to last row, etc.)
    offset_value: Option<i64>,
}

impl SlicePostprocessing {
    /// Create a new `SlicePostprocessing` with the given `number_rows` and `offset`.
    #[must_use]
    pub fn new(number_rows: Option<u64>, offset_value: Option<i64>) -> Self {
        Self {
            number_rows,
            offset_value,
        }
    }
}

impl<S: Scalar> PostprocessingStep<S> for SlicePostprocessing {
    /// Apply the slice transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        let num_rows = owned_table.num_rows();
        let limit = self.number_rows.unwrap_or(num_rows as u64);
        let offset = self.offset_value.unwrap_or(0);
        // Be permissive with data types at first so that computation can be done.
        // If the conversion fails, we will return None.
        let possible_starting_row = if offset < 0 {
            num_rows as i128 + offset as i128
        } else {
            offset as i128
        };
        // The `possible_ending_row` is NOT inclusive.
        let possible_ending_row = (possible_starting_row + limit as i128).min(num_rows as i128);
        let starting_row = usize::try_from(possible_starting_row).map_err(|_| {
            PostprocessingError::InvalidSliceIndex {
                index: possible_starting_row,
            }
        })?;
        let ending_row = usize::try_from(possible_ending_row).map_err(|_| {
            PostprocessingError::InvalidSliceIndex {
                index: possible_ending_row,
            }
        })?;
        Ok(OwnedTable::<S>::try_from_iter(
            owned_table
                .into_inner()
                .into_iter()
                .map(|(identifier, column)| (identifier, column.slice(starting_row, ending_row))),
        )
        .expect("Sliced columns of an existing table should have equal length"))
    }
}
