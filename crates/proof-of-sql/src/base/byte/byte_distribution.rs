use crate::base::{bit::bit_mask_utils::make_bit_mask, scalar::Scalar};
use ark_std::iterable::Iterable;
use bit_iter::BitIter;
use bnum::types::U256;
use core::{convert::Into, ops::Shl};
use serde::{Deserialize, Serialize};

/// Describes the distribution of byte values in a table column.
///
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ByteDistribution {
    /// Identifies any byte columns that vary
    vary_mask: u32,
    /// Identifies the values of the constant bytes (the ones not identified by `vary_mask`).
    /// The varying bytes are all set to `0`.
    constant_mask: [u64; 4],
}

impl ByteDistribution {
    /// Creates the `ByteDistribution` for a column of data.
    pub fn new<S: Scalar, T: Into<S> + Copy>(data: &[T]) -> Self {
        let bit_masks = data.iter().copied().map(Into::<S>::into).map(make_bit_mask);
        let (vary_mask, constant_mask) = (0u8..32)
            .map(|u| {
                let shifted_max_byte = U256::from(255u8).shl(u * 8);
                let mut shifted_byte_column = bit_masks
                    .clone()
                    .map(|bit_mask| bit_mask & shifted_max_byte);
                let (is_const, shifted_byte) = match shifted_byte_column.next() {
                    None => (true, U256::ZERO),
                    Some(first) => (shifted_byte_column.all(|byte| first == byte), first),
                };
                if is_const {
                    (0u32, shifted_byte)
                } else {
                    (1u32 << u, U256::ZERO)
                }
            })
            .fold(
                (0u32, U256::ZERO),
                |(vary_mask, constant_mask), (vary_bit, shifted_byte)| {
                    (vary_mask | vary_bit, constant_mask | shifted_byte)
                },
            );
        Self {
            vary_mask,
            constant_mask: constant_mask.into(),
        }
    }

    /// Returns the starting indices (`0, 8, ..., 248` are the possible values) of all varying byte columns.
    #[expect(clippy::missing_panics_doc)]
    pub fn varying_byte_indices(&self) -> impl Iterator<Item = u8> + '_ {
        BitIter::from(self.vary_mask)
            .iter()
            .map(|u| u8::try_from(u * 8).unwrap())
    }

    /// Returns the number of byte columns that vary.
    #[expect(clippy::missing_panics_doc)]
    pub fn varying_byte_count(&self) -> u8 {
        self.vary_mask.count_ones().try_into().unwrap()
    }

    /// Exposes `constant_mask` as `U256`
    pub fn constant_mask(&self) -> U256 {
        U256::from(self.constant_mask)
    }
}

#[cfg(test)]
mod tests {
    use super::ByteDistribution;
    use crate::base::scalar::{test_scalar::TestScalar, ScalarExt};
    use bnum::types::U256;
    use core::ops::{Neg, Shl, Shr};
    use itertools::Itertools;

    #[test]
    fn we_can_get_byte_distribution_from_empty_column() {
        let byte_distribution = ByteDistribution::new::<TestScalar, TestScalar>(&[]);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(byte_distribution.constant_mask(), U256::ZERO);
        assert_eq!(byte_distribution.varying_byte_count(), 0);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_single_positive_value_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let column = [value].map(TestScalar::from_wrapping);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            byte_distribution.constant_mask(),
            value | U256::ONE.shl(255)
        );
        assert_eq!(byte_distribution.varying_byte_count(), 0);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_single_negative_value_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let const_scalar = -TestScalar::from_wrapping(value);
        let column = [const_scalar];
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            byte_distribution.constant_mask(),
            U256::ONE.shl(255) - value
        );
        assert_eq!(byte_distribution.varying_byte_count(), 0);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_constant_positive_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let column = [value; 3].map(TestScalar::from_wrapping);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            byte_distribution.constant_mask(),
            value | U256::ONE.shl(255)
        );
        assert_eq!(byte_distribution.varying_byte_count(), 0);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_constant_negative_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let const_scalar = -TestScalar::from_wrapping(value);
        let column = [const_scalar; 3];
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            byte_distribution.constant_mask(),
            U256::ONE.shl(255) - value
        );
        assert_eq!(byte_distribution.varying_byte_count(), 0);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_bitwise_inverse_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        // We add one to the absolute value of the negative value so so that the scalars will be the bitwise inverses of each other.
        let negative_scalar = -TestScalar::from_wrapping(value + U256::ONE);
        let positive_scalar = TestScalar::from_wrapping(value);
        let column = [positive_scalar, negative_scalar];
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, u32::MAX);
        assert_eq!(byte_distribution.constant_mask(), U256::ZERO);
        assert_eq!(byte_distribution.varying_byte_count(), 32);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            (0u8..32).map(|i| i * 8).collect_vec()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_positive_column() {
        let column = [
            1_974_179_072u32,
            2_518_259_060,
            1_394_578_845,
            1_000_510_769,
            1_675_728_301,
        ]
        .map(TestScalar::from);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 13);
        assert_eq!(
            byte_distribution.constant_mask(),
            U256::from(149u8).shl(8) | U256::ONE.shl(255)
        );
        assert_eq!(byte_distribution.varying_byte_count(), 3);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            [0u8, 16, 24].into_iter().collect_vec()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_negative_column() {
        let column = [
            1_974_179_073u32,
            2_518_259_061,
            1_394_578_846,
            1_000_510_770,
            1_675_728_302,
        ]
        .map(TestScalar::from)
        .map(Neg::neg);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 13);
        assert_eq!(
            byte_distribution.constant_mask(),
            U256::from(106u8).shl(8) | U256::MAX.shr(33u8).shl(32)
        );
        assert_eq!(byte_distribution.varying_byte_count(), 3);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            [0u8, 16, 24].into_iter().collect_vec()
        );
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_column() {
        let negative_column = [
            1_974_179_073u32,
            2_518_259_061,
            1_394_578_846,
            1_000_510_770,
            1_675_728_302,
        ]
        .map(TestScalar::from)
        .map(Neg::neg);
        let positive_column = [
            1_974_179_072u32,
            2_518_259_060,
            1_394_578_845,
            1_000_510_769,
            1_675_728_301,
        ]
        .map(TestScalar::from);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(
            &negative_column
                .iter()
                .chain(positive_column.iter())
                .collect_vec(),
        );
        assert_eq!(byte_distribution.vary_mask, u32::MAX);
        assert_eq!(U256::from(byte_distribution.constant_mask), U256::ZERO);
        assert_eq!(byte_distribution.varying_byte_count(), 32);
        assert_eq!(
            byte_distribution.varying_byte_indices().collect_vec(),
            (0u8..32).map(|i| i * 8).collect_vec()
        );
    }
}
