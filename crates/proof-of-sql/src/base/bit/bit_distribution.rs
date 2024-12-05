use super::bit_mask_utils::make_sign_bit_mask;
use crate::base::scalar::Scalar;
use bit_iter::BitIter;
use core::convert::Into;
use serde::{Deserialize, Serialize};

/// Describe the distribution of bit values in a table column
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BitDistribution {
    /// Identifies all columns that are identical to the leading column (the sign column)
    pub sign_mask: [u64; 4],
    /// Identifies all columns that are the complement of the lead column
    pub inverse_sign_mask: [u64; 4],
}

impl BitDistribution {
    pub fn new<S: Scalar, T: Into<S> + Clone>(data: &[T]) -> Self {
        if data.is_empty() {
            return Self {
                sign_mask: [0; 4],
                inverse_sign_mask: [0; 4],
            };
        }
        let mut sign_mask = make_sign_bit_mask(data[0].clone().into());
        let mut inverse_sign_mask = sign_mask.map(|u| u ^ u64::MAX);

        for x in data.iter().skip(1) {
            let mask = make_sign_bit_mask((*x).clone().into());
            for i in 0..4 {
                sign_mask[i] &= mask[i];
                inverse_sign_mask[i] &= mask[i] ^ u64::MAX;
            }
        }
        Self {
            sign_mask,
            inverse_sign_mask,
        }
    }

    pub fn num_varying_bits(&self) -> usize {
        (0..4).fold(0, |acc, i| {
            acc + (self.sign_mask[i] | self.inverse_sign_mask[i]).count_zeros() as usize
        })
    }

    /// Check if this instance represents a valid bit distribution. `is_valid`
    /// can be used after deserializing a [`BitDistribution`] from an untrusted
    /// source.
    pub fn is_valid(&self) -> bool {
        for (s, i) in self.sign_mask.iter().zip(self.inverse_sign_mask) {
            if s & i != 0 {
                return false;
            }
        }
        true
    }
    // Value  = Sum of varying bits | or mask
    // Varying bits = vary mask & value

    /// Iterate over each varying bit
    pub fn for_each_varying_bit<F>(&self, mut f: F)
    where
        F: FnMut(usize, usize),
    {
        for i in 0..4 {
            let bitset = (self.sign_mask[i] | self.inverse_sign_mask[i]) ^ u64::MAX;
            for pos in BitIter::from(bitset) {
                f(i, pos);
            }
        }
    }
}
