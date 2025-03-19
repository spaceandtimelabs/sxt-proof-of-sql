use crate::base::{bit::bit_mask_utils::make_bit_mask, scalar::Scalar};
use bnum::types::U256;
use core::{
    convert::Into,
    ops::{Shl, Shr},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// Describe the distribution of byte values in a table column
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct ByteDistribution {
    /// Identifies any byte columns that do not satisify the following conditions:
    /// 1. The set of all bytes in the column is contained by a set of two bytes which are inverses of each other.
    /// 2. The byte for each row is determined by the lead bit. In other words, the byte column and leading bit column are perfectly correlated.
    vary_mask: u32,
    /// The only relevant bits in this mask are the ones that belong to a varying byte (one that is not identified by the `vary_mask`).
    /// Each relevant byte is the one that shadows a lead bit of 1. The inverse of each relevant byte shadows a lead bit of 0.
    /// The lead bit of this mask is only relevant if `is_leading_bit_constant` is true. In that case, it indicates the value of the lead bit.
    leading_bit_shadow_mask: [u64; 4],
    /// Indicates if the lead bit is constant
    is_leading_bit_constant: bool,
}

impl ByteDistribution {
    #[cfg_attr(not(test), expect(dead_code))]
    fn new<S: Scalar, T: Into<S> + Clone>(data: &[T]) -> Self {
        let bit_masks = data.iter().cloned().map(Into::<S>::into).map(make_bit_mask);
        let mut leading_bit_column = bit_masks.clone().map(|u| u >= U256::ONE << 255);
        let (vary_mask, leading_bit_shadow_mask) = (0u8..32)
            .map(|u| (u, U256::from(255u8) << (u * 8)))
            .map(|(u, shifted_max_byte)| {
                let mut one_shadow_shifted_byte_column = bit_masks
                    .clone()
                    .map(|bit_mask| bit_mask & shifted_max_byte)
                    .zip(leading_bit_column.clone())
                    .map(|(shifted_byte, leading_bit)| {
                        if leading_bit {
                            shifted_byte
                        } else if u == 31 {
                            shifted_byte ^ (shifted_max_byte ^ U256::ONE.shl(255))
                        } else {
                            shifted_byte ^ shifted_max_byte
                        }
                    });
                let (is_const, shifted_byte) = match one_shadow_shifted_byte_column.next() {
                    None => (true, U256::ZERO),
                    Some(a) => (
                        one_shadow_shifted_byte_column
                            .all(|x| a & U256::MAX.shr(1) == x & U256::MAX.shr(1)),
                        a,
                    ),
                };
                (if is_const { 0u32 } else { 1u32 << u }, shifted_byte)
            })
            .fold(
                (0u32, U256::ZERO),
                |(vary_mask, leading_bit_shadow_mask), (vary_bit, shifted_byte)| {
                    (vary_mask | vary_bit, leading_bit_shadow_mask | shifted_byte)
                },
            );
        Self {
            vary_mask,
            leading_bit_shadow_mask: leading_bit_shadow_mask.into(),
            is_leading_bit_constant: leading_bit_column.all_equal(),
        }
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
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask),
            U256::ZERO
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_single_positive_value_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let column = [value].map(TestScalar::from_wrapping);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask),
            value | U256::ONE.shl(255)
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_single_negative_value_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let const_scalar = -TestScalar::from_wrapping(value);
        let column = [const_scalar];
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask),
            (U256::ONE.shl(255) - value) ^ U256::MAX.shr(1)
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_constant_positive_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let column = [value; 3].map(TestScalar::from_wrapping);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask),
            value | U256::ONE.shl(255)
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_constant_negative_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        let const_scalar = -TestScalar::from_wrapping(value);
        let column = [const_scalar; 3];
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask),
            (U256::ONE.shl(255) - value) ^ U256::MAX.shr(1)
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_constant_column() {
        let value = U256::from(18_446_744_073_709_551_615u64);
        // We add one to the absolute value of the negative value so so that the scalars will be the bitwise inverses of each other.
        let negative_scalar = -TestScalar::from_wrapping(value + U256::ONE);
        let positive_scalar = TestScalar::from_wrapping(value);
        let column = [positive_scalar, negative_scalar];
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(byte_distribution.vary_mask, 0);
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask) & U256::MAX.shr(1),
            value
        );
        assert!(!byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_positive_column() {
        let leading_bit_shadow_mask = U256::from(149u8).shl(8) | U256::ONE.shl(255);
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
            U256::from(byte_distribution.leading_bit_shadow_mask)
                & (U256::from(255u8).shl(8) | U256::ONE.shl(255)),
            leading_bit_shadow_mask
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_negative_column() {
        let leading_bit_shadow_mask = U256::from(149u8).shl(8);
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
            U256::from(byte_distribution.leading_bit_shadow_mask)
                & (U256::from(255u8).shl(8) | U256::ONE.shl(255)),
            leading_bit_shadow_mask
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_column() {
        let leading_bit_shadow_mask = U256::from(149u8).shl(8);
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
        assert_eq!(byte_distribution.vary_mask, 13);
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask) & (U256::from(255u8).shl(8)),
            leading_bit_shadow_mask
        );
        assert!(!byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_lead_byte_in_positive_column() {
        let leading_bit_shadow_mask = U256::ONE.shl(255);
        let column = [U256::ZERO, U256::ONE.shl(250)].map(TestScalar::from_wrapping);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(U256::from(byte_distribution.vary_mask), U256::ONE.shl(31));
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask)
                & (U256::MAX.shr(8) | U256::ONE.shl(255)),
            leading_bit_shadow_mask
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_lead_byte_in_negative_column() {
        let leading_bit_shadow_mask = U256::ZERO;
        let column = [U256::ONE, U256::ONE.shl(250) + U256::ONE]
            .map(TestScalar::from_wrapping)
            .map(Neg::neg);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(U256::from(byte_distribution.vary_mask), U256::ONE.shl(31));
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask)
                & (U256::MAX.shr(8) | U256::ONE.shl(255)),
            leading_bit_shadow_mask
        );
        assert!(byte_distribution.is_leading_bit_constant);
    }

    #[test]
    fn we_can_get_byte_distribution_from_variable_lead_byte_in_column() {
        let leading_bit_shadow_mask = U256::ZERO;
        let column = [U256::ZERO, U256::ONE.shl(250) + U256::ONE]
            .map(TestScalar::from_wrapping)
            .map(Neg::neg);
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(U256::from(byte_distribution.vary_mask), U256::ONE.shl(31));
        assert_eq!(
            U256::from(byte_distribution.leading_bit_shadow_mask) & U256::MAX.shr(8),
            leading_bit_shadow_mask
        );
        assert!(!byte_distribution.is_leading_bit_constant);
    }
}
