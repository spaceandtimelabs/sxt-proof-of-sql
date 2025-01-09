use super::bit_mask_utils::{is_bit_mask_negative_representation, make_bit_mask};
use crate::base::scalar::{Scalar, ScalarExt};
use ark_std::iterable::Iterable;
use bit_iter::BitIter;
use bnum::types::U256;
use core::convert::Into;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// Describe the distribution of bit values in a table column
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BitDistribution {
    /// We use two arrays to track which bits vary
    /// and the constant bit values. If
    /// `{x_1, ..., x_n}` represents the values [`BitDistribution`] describes, then:
    ///   `or_all = abs(x_1) | abs(x_2) | ... | abs(x_n)`
    ///   `vary_mask & (1 << i) =`
    ///              `1` if `x_s & (1 << i) != x_t & (1 << i)` for some `s != t`
    ///              0 otherwise
    pub or_all: [u64; 4],
    /// Identifies all columns that are identical to the leading column (the sign column). The lead bit indicates if the sign column is constant
    pub(crate) vary_mask: [u64; 4],
    /// Identifies all columns that are the identical to the lead column. The lead bit indicates the sign of the last row of data (only relevant if the sign is constant)
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

    /// Identifies all columns that are the identical to the lead column.
    pub fn leading_bit_mask(&self) -> U256 {
        U256::from(self.leading_bit_mask) | (U256::ONE << 255)
    }

    /// Identifies all columns that are the identical to the inverse of the lead column.
    pub fn leading_bit_inverse_mask(&self) -> U256 {
        (!self.vary_mask() ^ self.leading_bit_mask()) & (U256::MAX >> 1)
    }

    /// # Panics
    ///
    /// Panics if conversion from `ExpType` to `usize` fails
    pub fn num_varying_bits(&self) -> usize {
        self.vary_mask().count_ones() as usize
    }

    /// # Panics
    ///
    /// Panics if lead bit varies but `bit_evals` is empty
    pub fn leading_bit_eval<S: ScalarExt>(&self, bit_evals: &[S], one_eval: S) -> S {
        if U256::from(self.vary_mask) & (U256::ONE << 255) != U256::ZERO {
            *bit_evals.last().expect("bit_evals should be non-empty")
        } else if U256::from(self.leading_bit_mask) & (U256::ONE << 255) == U256::ZERO {
            S::ZERO
        } else {
            one_eval
        }
    }

    pub fn has_varying_sign_bit(&self) -> bool {
        self.vary_mask[3] & (1 << 63) != 0
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn sign_bit(&self) -> bool {
        assert!(!self.has_varying_sign_bit());
        self.or_all[3] & (1 << 63) != 0
    }

    /// Check if this instance represents a valid bit distribution. `is_valid`
    /// can be used after deserializing a [`BitDistribution`] from an untrusted
    /// source.
    pub fn is_valid(&self) -> bool {
        (self.vary_mask() & self.leading_bit_mask()) & (U256::MAX >> 1) == U256::ZERO
    }

    /// In order to avoid cases with large numbers where there can be both a positive and negative
    /// representation, we restrict the range of bit distributions that we accept.
    ///
    /// Currently this is set to be the minimal value that will include the sum of two signed 128-bit
    /// integers. The range will likely be expanded in the future as we support additional expressions.
    pub fn is_within_acceptable_range(&self) -> bool {
        // signed 128 bit numbers range from
        //      -2^127 to 2^127-1
        // the maximum absolute value of the sum of two signed 128-integers is
        // then
        //       2 * (2^127) = 2^128
        (self.leading_bit_inverse_mask() >> 128) == (U256::MAX >> 129)
    }

    /// Iterate over each varying bit
    ///
    /// # Panics
    ///
    /// The panic shouldn't be mathematically possible
    pub fn vary_mask_iter(&self) -> impl Iterator<Item = u8> + '_ {
        (0..4).flat_map(|i| {
            BitIter::from(self.vary_mask[i])
                .iter()
                .map(move |pos| u8::try_from(i * 64 + pos).expect("index greater than 255"))
        })
    }

    /// If `{b_i}` represents the non-varying 1-bits of the absolute values, return the value
    ///    `sum_i b_i 2 ^ i`
    pub fn constant_part(&self) -> [u64; 4] {
        let mut val = [0; 4];
        self.for_each_abs_constant_bit(|i: usize, bit: usize| {
            val[i] |= 1u64 << bit;
        });
        val
    }

    /// Iterate over each constant 1-bit for the absolute values
    pub fn for_each_abs_constant_bit<F>(&self, mut f: F)
    where
        F: FnMut(usize, usize),
    {
        for i in 0..4 {
            let bitset = if i == 3 {
                !(self.vary_mask[i] | (1 << 63))
            } else {
                !self.vary_mask[i]
            };
            let bitset = bitset & self.or_all[i];
            for pos in BitIter::from(bitset) {
                f(i, pos);
            }
        }
    }

    /// Iterate over each varying bit for the absolute values
    pub fn for_each_abs_varying_bit<F>(&self, mut f: F)
    where
        F: FnMut(usize, usize),
    {
        for i in 0..4 {
            let bitset = if i == 3 {
                self.vary_mask[i] & !(1 << 63)
            } else {
                self.vary_mask[i]
            };
            for pos in BitIter::from(bitset) {
                f(i, pos);
            }
        }
    }

    /// Iterate over each varying bit for the absolute values and the sign bit
    /// if it varies
    pub fn for_each_varying_bit<F>(&self, mut f: F)
    where
        F: FnMut(usize, usize),
    {
        for i in 0..4 {
            let bitset = self.vary_mask[i];
            for pos in BitIter::from(bitset) {
                f(i, pos);
            }
        }
    }

    /// Return the position of the most significant bit of the absolute values
    /// # Panics
    /// Panics if no bits are set to 1 in the bit representation of `or_all`.
    pub fn most_significant_abs_bit(&self) -> usize {
        let mask = self.or_all[3] & !(1 << 63);
        if mask != 0 {
            return 64 - (mask.leading_zeros() as usize) - 1 + 3 * 64;
        }
        for i in (0..3).rev() {
            let mask = self.or_all[i];
            if mask != 0 {
                return 64 - (mask.leading_zeros() as usize) - 1 + 64 * i;
            }
        }
        panic!("no bits are set");
    }
}
