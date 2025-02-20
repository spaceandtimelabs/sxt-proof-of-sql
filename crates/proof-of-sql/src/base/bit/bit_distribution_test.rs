use super::*;
use crate::base::scalar::{Scalar, ScalarExt, test_scalar::TestScalar};
use bnum::types::U256;
use core::ops::Shl;

// vary_mask function start

#[test]
fn we_can_get_u256_version_of_vary_mask_for_zero() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [0; 4],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let u256_vary_mask = bit_distribution.vary_mask();

    // ASSERT
    assert_eq!(u256_vary_mask, U256::ZERO);
}

#[test]
fn we_can_get_u256_version_of_vary_mask_for_one() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [1, 0, 0, 0],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let u256_vary_mask = bit_distribution.vary_mask();

    // ASSERT
    assert_eq!(u256_vary_mask, U256::ONE);
}

#[test]
fn we_can_get_u256_version_of_vary_mask_for_large_number() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [256, 0, 0, 256],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let u256_vary_mask = bit_distribution.vary_mask();

    // ASSERT
    assert_eq!(u256_vary_mask, U256::ONE.shl(8) + (U256::ONE.shl(200)));
}

// vary_mask function end

// leading_bit_mask function start

#[test]
fn we_can_get_u256_version_of_leading_bit_mask_for_zero() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [0; 4],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let u256_leading_bit_mask = bit_distribution.leading_bit_mask();

    // ASSERT
    assert_eq!(u256_leading_bit_mask, U256::ONE.shl(255));
}

#[test]
fn we_can_get_u256_version_of_leading_bit_mask_for_one() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [0; 4],
        leading_bit_mask: [1, 0, 0, 0],
    };

    // ACT
    let u256_leading_bit_mask = bit_distribution.leading_bit_mask();

    // ASSERT
    assert_eq!(u256_leading_bit_mask, U256::ONE | (U256::ONE.shl(255)));
}

#[test]
fn we_can_get_u256_version_of_leading_bit_mask_for_large_number() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [0; 4],
        leading_bit_mask: [256, 0, 0, 256],
    };

    // ACT
    let u256_leading_bit_mask = bit_distribution.leading_bit_mask();

    // ASSERT
    assert_eq!(
        u256_leading_bit_mask,
        (U256::ONE.shl(8) + U256::ONE.shl(200)) | U256::ONE.shl(255)
    );
}

// leading_bit_mask function end

// leading_bit_inverse_mask function start

#[test]
fn we_can_get_u256_version_of_leading_bit_inverse_mask_for_zero() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [u64::MAX; 4],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let u256_leading_bit_inverse_mask = bit_distribution.leading_bit_inverse_mask();

    // ASSERT
    assert_eq!(u256_leading_bit_inverse_mask, U256::ZERO);
}

#[test]
fn we_can_get_u256_version_of_leading_bit_inverse_mask_for_one() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [u64::MAX - 1, u64::MAX, u64::MAX, u64::MAX],
        leading_bit_mask: [0, 0, 0, 0],
    };

    // ACT
    let u256_leading_bit_inverse_mask = bit_distribution.leading_bit_inverse_mask();

    // ASSERT
    assert_eq!(u256_leading_bit_inverse_mask, U256::ONE);
}

#[test]
fn we_can_get_u256_version_of_leading_bit_inverse_mask_for_large_number() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [u64::MAX - 255, u64::MAX, u64::MAX, u64::MAX - 255],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let u256_leading_bit_inverse_mask = bit_distribution.leading_bit_inverse_mask();

    // ASSERT
    assert_eq!(
        u256_leading_bit_inverse_mask,
        (U256::ONE.shl(8) - U256::ONE) + ((U256::ONE.shl(8) - U256::ONE) << 192)
    );
}

// leading_bit_inverse_mask function end

#[test]
fn we_can_get_leading_bit_eval_while_varying() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [0, 0, 0, 1 << 63],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let bit_eval = bit_distribution
        .leading_bit_eval(&[TestScalar::ONE], TestScalar::TWO)
        .unwrap();

    // ASSERT
    assert_eq!(bit_eval, TestScalar::ONE);
}

#[test]
fn we_can_get_leading_bit_eval_while_constant_and_zero() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [0, 0, 0, 0],
        leading_bit_mask: [0; 4],
    };

    // ACT
    let bit_eval = bit_distribution
        .leading_bit_eval(&[TestScalar::ONE], TestScalar::TWO)
        .unwrap();

    // ASSERT
    assert_eq!(bit_eval, TestScalar::ZERO);
}

#[test]
fn we_can_get_leading_bit_eval_while_constant_and_non_zero() {
    // ARRANGE
    let bit_distribution = BitDistribution {
        vary_mask: [0, 0, 0, 0],
        leading_bit_mask: [0, 0, 0, 1 << 63],
    };

    // ACT
    let bit_eval = bit_distribution
        .leading_bit_eval(&[TestScalar::ONE], TestScalar::TWO)
        .unwrap();

    // ASSERT
    assert_eq!(bit_eval, TestScalar::TWO);
}

// leading_bit_eval functions start

// leading_bit_eval functions end

#[test]
fn we_can_compute_the_bit_distribution_of_an_empty_slice() {
    let data: Vec<i64> = vec![];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::MAX)
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_inverse_mask()),
        TestScalar::from(0)
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.vary_mask()),
        TestScalar::from(0)
    );

    assert_eq!(dist.vary_mask_iter().count(), 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_a_single_element() {
    let val = (1 << 2) | (1 << 10);
    let data: Vec<i64> = vec![val];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::ONE.shl(2) | U256::ONE.shl(10) | (U256::ONE.shl(255)))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.vary_mask()),
        TestScalar::from(0)
    );
    assert_eq!(
        dist.leading_bit_inverse_mask(),
        (U256::ONE.shl(2) | U256::ONE.shl(10) | (U256::ONE.shl(255))) ^ U256::MAX
    );

    assert_eq!(dist.vary_mask_iter().count(), 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_one_varying_bits() {
    let data: Vec<i64> = vec![(1 << 2) | (1 << 10), (1 << 2) | (1 << 10) | (1 << 21)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 1);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::ONE.shl(2) | U256::ONE.shl(10) | (U256::ONE.shl(255)))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_inverse_mask()),
        TestScalar::from_wrapping(
            (U256::FOUR | U256::ONE.shl(10) | U256::ONE.shl(21) | (U256::ONE.shl(255))) ^ U256::MAX
        )
    );

    assert_eq!(dist.vary_mask_iter().count(), 1);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_multiple_varying_bits() {
    let data: Vec<i64> = vec![
        (1 << 2) | (1 << 10),
        (1 << 3) | (1 << 10) | (1 << 21),
        (1 << 10) | (1 << 21) | (1 << 50),
    ];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 4);
    assert!(dist.is_valid());

    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::ONE.shl(10) | (U256::ONE.shl(255)))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_inverse_mask()),
        TestScalar::from_wrapping(
            (U256::FOUR
                | U256::EIGHT
                | U256::ONE.shl(10)
                | U256::ONE.shl(21)
                | U256::ONE.shl(50)
                | (U256::ONE.shl(255)))
                ^ U256::MAX
        )
    );

    for i in dist.vary_mask_iter() {
        assert!(i == 2 || i == 3 || i == 21 || i == 50);
    }
    assert_eq!(dist.vary_mask_iter().count(), 4);
}

#[test]
fn we_can_compute_the_bit_distribution_of_negative_values() {
    let data: Vec<i64> = vec![-1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::ONE.shl(255))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_inverse_mask()),
        TestScalar::from_wrapping(U256::MAX ^ (U256::ONE.shl(255)))
    );

    assert_eq!(dist.vary_mask_iter().count(), 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs() {
    let data: Vec<i64> = vec![-1, 1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 2);
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::ONE.shl(255))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_inverse_mask()),
        TestScalar::from_wrapping(U256::MAX ^ (U256::ONE | (U256::ONE.shl(255))))
    );

    assert_eq!(dist.vary_mask_iter().count(), 2);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs_and_values() {
    let data: Vec<i64> = vec![4, -1, 1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 3);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::ONE.shl(255))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_inverse_mask()),
        TestScalar::from_wrapping(U256::MAX ^ (U256::FIVE | (U256::ONE.shl(255))))
    );

    assert_eq!(dist.vary_mask_iter().count(), 3);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_larger_than_64_bit_integers() {
    let mut val = [0; 4];
    val[3] = 1 << 11;
    let data: Vec<TestScalar> = vec![TestScalar::from_bigint(val)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_mask()),
        TestScalar::from_wrapping(U256::ONE.shl(203) | (U256::ONE.shl(255)))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.leading_bit_inverse_mask()),
        TestScalar::from_wrapping(U256::MAX ^ (U256::ONE.shl(203) | (U256::ONE.shl(255))))
    );

    assert_eq!(dist.vary_mask_iter().count(), 0);
}

#[test]
fn we_can_detect_invalid_bit_distributions() {
    let dist = BitDistribution {
        vary_mask: [1, 0, 0, 0],
        leading_bit_mask: [1, 0, 0, 0],
    };
    assert!(!dist.is_valid());
}

#[test]
fn zero_is_within_range() {
    let data: Vec<TestScalar> = vec![TestScalar::from(0)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert!(dist.is_within_acceptable_range());
}

#[test]
fn the_sum_of_two_signed_128_bit_numbers_is_within_range() {
    let data: Vec<TestScalar> = vec![TestScalar::from(i128::MIN) + TestScalar::from(i128::MIN)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert!(dist.is_within_acceptable_range());
}

#[test]
fn we_reject_distributions_that_are_outside_of_maximum_range() {
    let data: Vec<TestScalar> = vec![
        TestScalar::from(u128::MAX) + TestScalar::from(u128::MAX) + TestScalar::from(u128::MAX),
    ];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert!(!dist.is_within_acceptable_range());
}
