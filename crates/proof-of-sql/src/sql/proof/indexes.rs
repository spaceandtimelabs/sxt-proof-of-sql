use crate::base::{polynomial::compute_truncated_lagrange_basis_sum, scalar::Scalar};
use alloc::vec::Vec;
use core::{ops::Range, slice};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Indexes of a table for use in the [`ProvableQueryResult`](crate::sql::proof::ProvableQueryResult)
pub enum Indexes {
    /// Sparse indexes. (i.e. explicitly specified indexes)
    Sparse(Vec<u64>),
    /// Dense indexes. (i.e. all indexes in a range, which means the indexes do not need to be sent to the verifier)
    Dense(Range<u64>),
}

impl Default for Indexes {
    fn default() -> Self {
        Self::Sparse(Vec::default())
    }
}

impl Indexes {
    /// Check if the indexes are valid for a table with n rows
    pub fn valid(&self, n: usize) -> bool {
        let n = n as u64;
        match &self {
            Self::Sparse(ix) => {
                if ix.is_empty() {
                    return true;
                }
                let index = ix[0];
                if index >= n {
                    return false;
                }
                let mut prev_index = index;
                for index in ix.iter().skip(1) {
                    if *index <= prev_index || *index >= n {
                        return false;
                    }
                    prev_index = *index;
                }
                true
            }
            Self::Dense(range) => range.end <= n && (range.start < range.end || range.start == 0),
        }
    }
    /// Get an iterator over the indexes
    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        enum Iter<'a> {
            Sparse(slice::Iter<'a, u64>),
            Dense(Range<u64>),
        }
        impl<'a> Iterator for Iter<'a> {
            type Item = u64;
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Iter::Sparse(iter) => iter.next().copied(),
                    Iter::Dense(iter) => iter.next(),
                }
            }
        }
        match self {
            Self::Sparse(vec) => Iter::Sparse(vec.iter()),
            Self::Dense(range) => Iter::Dense(range.clone()),
        }
    }
    /// Get the number of indexes
    pub fn len(&self) -> usize {
        match self {
            Self::Sparse(vec) => vec.len(),
            Self::Dense(range) => {
                if range.end <= range.start {
                    0
                } else {
                    (range.end - range.start) as usize
                }
            }
        }
    }
    /// Check if the number of indexes is zero.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Sparse(vec) => vec.is_empty(),
            Self::Dense(range) => range.end <= range.start,
        }
    }

    /// Evaluates the mle that is 1 at the indexes and 0 elsewhere at the given evaluation point.
    /// This returne None for Sparse indexes and the actual value for Dense indexes.
    pub fn evaluate_at_point<S: Scalar>(&self, evaluation_point: &[S]) -> Option<S> {
        match self {
            Indexes::Sparse(_) => None,
            Indexes::Dense(range) => {
                if range.is_empty() {
                    Some(Zero::zero())
                } else if range.end as usize > 2usize.pow(evaluation_point.len() as u32) {
                    // This only happens when the indexes are tampered with.
                    None
                } else {
                    Some(
                        compute_truncated_lagrange_basis_sum(range.end as usize, evaluation_point)
                            - compute_truncated_lagrange_basis_sum(
                                range.start as usize,
                                evaluation_point,
                            ),
                    )
                }
            }
        }
    }
}
