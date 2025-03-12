use crate::base::if_rayon;
use alloc::{format, string::String, vec::Vec};
use core::cmp::Ordering;
use itertools::Itertools;
#[cfg(feature = "rayon")]
use rayon::prelude::ParallelSliceMut;
use snafu::Snafu;

/// An error that occurs when working with permutations
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum PermutationError {
    /// The permutation is invalid
    #[snafu(display("Permutation is invalid {error}"))]
    InvalidPermutation { error: String },
    /// Application of a permutation to a slice with an incorrect length
    #[snafu(display("Application of a permutation to a slice with a different length {permutation_size} != {slice_length}"))]
    PermutationSizeMismatch {
        permutation_size: usize,
        slice_length: usize,
    },
}

/// Permutation of [0, 1, 2, ..., n-1]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permutation {
    /// The permutation
    permutation: Vec<usize>,
}

impl Permutation {
    /// Create a new permutation from a comparison function with the given length
    pub(crate) fn unchecked_new_from_cmp<F>(length: usize, cmp: F) -> Self
    where
        F: Fn(&usize, &usize) -> Ordering + Sync,
    {
        let mut indexes = (0..length).collect_vec();
        if_rayon!(
            indexes.par_sort_unstable_by(cmp),
            indexes.sort_unstable_by(cmp)
        );
        Self {
            permutation: indexes,
        }
    }

    /// Create a new permutation. If the permutation is invalid, return an error.
    pub fn try_new(permutation: Vec<usize>) -> Result<Self, PermutationError> {
        let length = permutation.len();
        // Check for uniqueness
        let mut elements = permutation.clone();
        elements.sort_unstable();
        elements.dedup();
        if elements.len() < length {
            Err(PermutationError::InvalidPermutation {
                error: format!("Permutation can not have duplicate elements: {permutation:?}"),
            })
        }
        // Check that no element is out of bounds
        else if permutation.iter().any(|&i| i >= length) {
            Err(PermutationError::InvalidPermutation {
                error: format!("Permutation can not have elements out of bounds: {permutation:?}"),
            })
        } else {
            Ok(Self { permutation })
        }
    }

    /// Get the size of the permutation
    pub fn size(&self) -> usize {
        self.permutation.len()
    }

    /// Apply the permutation to the given slice
    pub fn try_apply<T>(&self, slice: &[T]) -> Result<Vec<T>, PermutationError>
    where
        T: Clone,
    {
        if slice.len() == self.size() {
            Ok(self.permutation.iter().map(|&i| slice[i].clone()).collect())
        } else {
            Err(PermutationError::PermutationSizeMismatch {
                permutation_size: self.size(),
                slice_length: slice.len(),
            })
        }
    }

    /// Apply the permutation to chunks of the given size within the slice
    pub fn try_chunked_apply<T>(
        &self,
        slice: &[T],
        chunk_size: usize,
    ) -> Result<Vec<T>, PermutationError>
    where
        T: Clone,
    {
        if slice.len() % chunk_size != 0 {
            return Err(PermutationError::PermutationSizeMismatch {
                permutation_size: self.size(),
                slice_length: slice.len(),
            });
        }

        let num_chunks = slice.len() / chunk_size;
        if self.size() != num_chunks {
            return Err(PermutationError::PermutationSizeMismatch {
                permutation_size: self.size(),
                slice_length: num_chunks,
            });
        }

        let mut result = Vec::with_capacity(slice.len());
        for &i in &self.permutation {
            let start = i * chunk_size;
            let end = start + chunk_size;
            result.extend_from_slice(&slice[start..end]);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::vec;

    #[test]
    fn we_cannot_chunked_apply_if_slice_length_not_multiple_of_chunk_size() {
        let permutation = Permutation::try_new(vec![1, 0]).unwrap();

        let slice = [10, 20, 30, 40, 50];
        let result = permutation.try_chunked_apply(&slice, 2);
        assert_eq!(
            result,
            Err(PermutationError::PermutationSizeMismatch {
                permutation_size: 2,
                slice_length: 5
            })
        );
    }

    #[test]
    fn we_cannot_chunked_apply_if_num_chunks_does_not_match_permutation_length() {
        let permutation = Permutation::try_new(vec![2, 0, 1]).unwrap();

        let slice_ok = [100, 101, 200, 201, 300, 301];
        let ok_result = permutation.try_chunked_apply(&slice_ok, 2).unwrap();

        assert_eq!(ok_result, vec![300, 301, 100, 101, 200, 201]);

        let slice_mismatch = [10, 11, 12, 13];
        let err_result = permutation.try_chunked_apply(&slice_mismatch, 2);
        assert_eq!(
            err_result,
            Err(PermutationError::PermutationSizeMismatch {
                permutation_size: 3,
                slice_length: 2
            })
        );
    }

    #[test]
    fn test_apply_permutation() {
        let permutation = Permutation::try_new(vec![1, 0, 2]).unwrap();
        assert_eq!(permutation.size(), 3);
        assert_eq!(
            permutation.try_apply(&["and", "Space", "Time"]).unwrap(),
            vec!["Space", "and", "Time"]
        );
    }

    #[test]
    fn test_invalid_permutation() {
        assert!(matches!(
            Permutation::try_new(vec![1, 0, 0]),
            Err(PermutationError::InvalidPermutation { .. })
        ));
        assert!(matches!(
            Permutation::try_new(vec![1, 0, 3]),
            Err(PermutationError::InvalidPermutation { .. })
        ));
    }

    #[test]
    fn test_permutation_size_mismatch() {
        let permutation = Permutation::try_new(vec![1, 0, 2]).unwrap();
        assert_eq!(
            permutation.try_apply(&["Space", "Time"]),
            Err(PermutationError::PermutationSizeMismatch {
                permutation_size: 3,
                slice_length: 2
            })
        );
    }
}
