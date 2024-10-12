<<<<<<< Updated upstream
use alloc::{format, string::String, vec::Vec};
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
    /// Create a new permutation without checks
    ///
    /// Warning: This function does not check if the permutation is valid.
    /// Only use this function if you are sure that the permutation is valid.
    pub(crate) fn unchecked_new(permutation: Vec<usize>) -> Self {
        Self { permutation }
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
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::vec;

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
=======
use alloc::{format, string::String, vec::Vec};
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
    /// Create a new permutation without checks
    ///
    /// Warning: This function does not check if the permutation is valid.
    /// Only use this function if you are sure that the permutation is valid.
    pub(crate) fn unchecked_new(permutation: Vec<usize>) -> Self {
        Self { permutation }
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
        if slice.len() != self.size() {
            Err(PermutationError::PermutationSizeMismatch {
                permutation_size: self.size(),
                slice_length: slice.len(),
            })
        } else {
            Ok(self.permutation.iter().map(|&i| slice[i].clone()).collect())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::vec;

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
>>>>>>> Stashed changes
