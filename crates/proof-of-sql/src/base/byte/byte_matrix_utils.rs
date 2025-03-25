use super::ByteDistribution;
use crate::base::{bit::bit_mask_utils::make_bit_mask, scalar::Scalar};
use bnum::types::U256;
use core::ops::Shr;

/// Let `x1, ..., xn` denote the values of a data column. Let
/// `b1, ..., bk` denote the bit positions of `abs(x1), ..., abs(xn)`
/// that vary.
///
/// `compute_varying_bit_matrix` returns the matrix M where
///   `M_ij = abs(xi) & (1 << bj) == 1`
/// The last column of M corresponds to the sign bit if it varies.
#[expect(clippy::missing_panics_doc)]
pub fn compute_varying_byte_matrix<S: Scalar>(
    column_data: &[impl Copy + Into<S>],
    dist: &ByteDistribution,
) -> impl Iterator<Item = Vec<u8>> + Clone {
    dist.varying_byte_indices()
        .map(|start_index| {
            column_data
                .iter()
                .map(move |row| {
                    let bit_mask = make_bit_mask((*row).into());
                    let shifted_byte: u8 = (bit_mask.shr(start_index) & U256::from(255u8))
                        .try_into()
                        .unwrap();
                    shifted_byte
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter()
}
