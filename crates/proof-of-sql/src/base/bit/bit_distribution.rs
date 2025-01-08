use crate::base::{bit::make_abs_bit_mask, scalar::Scalar};
use bit_iter::BitIter;
use core::convert::Into;
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
    pub vary_mask: [u64; 4],
}

impl BitDistribution {
    pub fn new<S: Scalar, T: Into<S> + Clone>(data: &[T]) -> Self {
        if data.is_empty() {
            return Self {
                or_all: [0; 4],
                vary_mask: [0; 4],
            };
        }
        let mut or_all = make_abs_bit_mask(data[0].clone().into());
        let mut vary_mask = [0; 4];
        for x in data.iter().skip(1) {
            let mask = make_abs_bit_mask((*x).clone().into());
            for i in 0..4 {
                vary_mask[i] |= or_all[i] ^ mask[i];
                or_all[i] |= mask[i];
            }
        }
        Self { or_all, vary_mask }
    }

    pub fn vary_mask(&self) -> U256 {
        U256::from(self.vary_mask)
    }

    /// # Panics
    ///
    /// Panics if conversion from `ExpType` to `usize` fails
    pub fn num_varying_bits(&self) -> usize {
        self.vary_mask().count_ones() as usize
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
        for (m, o) in self.vary_mask.iter().zip(self.or_all) {
            if m & !o != 0 {
                return false;
            }
        }
        true
    }

    /// In order to avoid cases with large numbers where there can be both a positive and negative
    /// representation, we restrict the range of bit distributions that we accept.
    ///
    /// Currently this is set to be the minimal value that will include the sum of two signed 128-bit
    /// integers. The range will likely be expanded in the future as we support additional expressions.
    pub fn is_within_acceptable_range(&self) -> bool {
        // handle the case of everything zero
        if self.num_varying_bits() == 0 && self.constant_part() == [0; 4] {
            return true;
        }

        // signed 128 bit numbers range from
        //      -2^127 to 2^127-1
        // the maximum absolute value of the sum of two signed 128-integers is
        // then
        //       2 * (2^127) = 2^128
        self.most_significant_abs_bit() <= 128
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
