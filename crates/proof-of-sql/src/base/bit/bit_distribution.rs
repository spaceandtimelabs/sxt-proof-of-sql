use super::bit_mask_utils::{is_bit_mask_negative_representation, make_bit_mask};
use crate::base::scalar::{Scalar, ScalarExt};
use ark_std::iterable::Iterable;
use bit_iter::BitIter;
use bnum::types::U256;
use core::{convert::Into, option::Iter};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// Describe the distribution of bit values in a table column
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BitDistribution {
    /// Identifies all columns that are identical to the leading column (the sign column). The lead bit indicates if the sign column is constant
    pub(crate) vary_mask: [u64; 4],
    /// Identifies all columns that are the identcal to the lead column. The lead bit indicates the sign of the last row of data (only relevant if the sign is constant)
    pub(crate) leading_bit_mask: [u64; 4],
}

impl BitDistribution {
    pub fn new<S: Scalar, T: Into<S> + Clone>(data: &[T]) -> Self {
        let bit_masks = data.iter().cloned().map(Into::<S>::into).map(make_bit_mask);
        let (sign_mask, inverse_sign_mask) =
            bit_masks
                .clone()
                .fold((U256::MAX, U256::MAX), |acc, bit_mask| {
                    let bit_mask = if is_bit_mask_negative_representation(bit_mask) {
                        bit_mask ^ (U256::MAX >> 1)
                    } else {
                        bit_mask
                    };
                    (acc.0 & bit_mask, acc.1 & !bit_mask)
                });
        let vary_mask_bit = U256::from(
            !bit_masks
                .map(is_bit_mask_negative_representation)
                .all_equal(),
        ) << 255;
        let vary_mask: U256 = !(sign_mask | inverse_sign_mask) | vary_mask_bit;

        Self {
            leading_bit_mask: sign_mask.into(),
            vary_mask: vary_mask.into(),
        }
    }

    pub fn vary_mask(&self) -> U256 {
        U256::from(self.vary_mask)
    }

    pub fn leading_bit_mask(&self) -> U256 {
        U256::from(self.leading_bit_mask) | (U256::ONE << 255)
    }

    pub fn leading_bit_inverse_mask(&self) -> U256 {
        (!self.vary_mask() ^ self.leading_bit_mask()) & (U256::MAX >> 1)
    }

    pub fn num_varying_bits(&self) -> usize {
        self.vary_mask().count_ones().try_into().unwrap()
    }

    pub fn leading_bit_eval<S: ScalarExt>(&self, bit_evals: &[S], one_eval: S) -> S {
        if U256::from(self.vary_mask) & (U256::ONE << 255) != U256::ZERO {
            *bit_evals.last().expect("bit_evals should be non-empty")
        } else if U256::from(self.leading_bit_mask) & (U256::ONE << 255) == U256::ZERO {
            S::ZERO
        } else {
            one_eval
        }
    }

    /// Check if this instance represents a valid bit distribution. `is_valid`
    /// can be used after deserializing a [`BitDistribution`] from an untrusted
    /// source.
    pub fn is_valid(&self) -> bool {
        (self.vary_mask() & self.leading_bit_mask()) & (U256::MAX >> 1) == U256::ZERO
    }
    // Value  = Sum of varying bits | or mask
    // Varying bits = vary mask & value

    /// Iterate over each varying bit
    pub fn vary_mask_iter(&self) -> impl Iterator<Item = u8> + '_ {
        (0..4).flat_map(|i| {
            BitIter::from(self.vary_mask[i])
                .iter()
                .map(move |pos| (i * 64 + pos) as u8)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::base::bit::BitDistribution;

    #[test]
    fn we_can_detect_invalid_bit_distributions() {
        let dist = BitDistribution {
            leading_bit_mask: [1, 0, 0, 0],
            vary_mask: [1, 0, 0, 0],
        };
        assert!(!dist.is_valid());
    }
}
