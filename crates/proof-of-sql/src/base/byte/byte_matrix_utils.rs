use super::ByteDistribution;
use crate::base::{bit::bit_mask_utils::make_bit_mask, scalar::Scalar};
use bnum::types::U256;
use core::ops::Shr;
use itertools::Itertools;

/// Let `x1, ..., xn` denote the values of a data column. Let
/// `b1, ..., bk` denote the bit positions of `abs(x1), ..., abs(xn)`
/// that vary.
///
/// `compute_varying_bit_matrix` returns the matrix M where
///   `M_ij = abs(xi) & (1 << bj) == 1`
/// The last column of M corresponds to the sign bit if it varies.
pub fn compute_varying_byte_matrix(
    bit_masks: impl Iterator<Item = U256> + Clone,
    dist: &ByteDistribution,
) -> Vec<Vec<u8>> {
    dist.varying_byte_indices()
        .map(|start_index| {
            bit_masks.clone().map(move |bit_mask| {
                let shifted_byte: u8 = bit_mask.shr(start_index).try_into().unwrap();
                shifted_byte
            })
        })
        .fold(
            vec![Vec::new(); dist.varying_byte_indices().count()],
            |acc: Vec<Vec<u8>>, shifted_bytes| {
                acc.into_iter()
                    .zip(shifted_bytes)
                    .map(|(v, b)| v.into_iter().chain([b].into_iter()).collect_vec())
                    .collect_vec()
            },
        )
}
