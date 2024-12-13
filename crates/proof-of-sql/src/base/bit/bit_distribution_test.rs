use super::*;
use crate::base::scalar::{test_scalar::TestScalar, ScalarExt};
use bnum::types::U256;

#[test]
fn we_can_compute_the_bit_distribution_of_an_empty_slice() {
    let data: Vec<i64> = vec![];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping(U256::MAX)
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.inverse_sign_mask()),
        TestScalar::from(0)
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.vary_mask()),
        TestScalar::from(0)
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_i, _j| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_a_single_element() {
    let val = (1 << 2) | (1 << 10);
    let data: Vec<i64> = vec![val];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping((U256::ONE << 2) | (U256::ONE << 10) | (U256::ONE << 255))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.vary_mask()),
        TestScalar::from(0)
    );
    assert_eq!(
        dist.inverse_sign_mask(),
        ((U256::ONE << 2) | (U256::ONE << 10) | (U256::ONE << 255)) ^ U256::MAX
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_i, _j| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_one_varying_bits() {
    let data: Vec<i64> = vec![(1 << 2) | (1 << 10), (1 << 2) | (1 << 10) | (1 << 21)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 1);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping((U256::ONE << 2) | (U256::ONE << 10) | (U256::ONE << 255))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.inverse_sign_mask()),
        TestScalar::from_wrapping(
            (U256::FOUR | (U256::ONE << 10) | (U256::ONE << 21) | (U256::ONE << 255)) ^ U256::MAX
        )
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_index, i: u8| {
        assert_eq!(i, 21);
        cnt += 1;
    });
    assert_eq!(cnt, 1);
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
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping((U256::ONE << 10) | (U256::ONE << 255))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.inverse_sign_mask()),
        TestScalar::from_wrapping(
            (U256::FOUR
                | U256::EIGHT
                | (U256::ONE << 10)
                | (U256::ONE << 21)
                | (U256::ONE << 50)
                | (U256::ONE << 255))
                ^ U256::MAX
        )
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_index, i: u8| {
        assert!(i == 2 || i == 3 || i == 21 || i == 50);
        cnt += 1;
    });
    assert_eq!(cnt, 4);
}

#[test]
fn we_can_compute_the_bit_distribution_of_negative_values() {
    let data: Vec<i64> = vec![-1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping(U256::ONE << 255)
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.inverse_sign_mask()),
        TestScalar::from_wrapping(U256::MAX ^ (U256::ONE << 255))
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_i, _j| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs() {
    let data: Vec<i64> = vec![-1, 1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 2);
    assert_eq!(
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping(U256::ONE << 255)
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.inverse_sign_mask()),
        TestScalar::from_wrapping(U256::MAX ^ (U256::ONE | (U256::ONE << 255)))
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_index, i: u8| {
        assert!(i == 0 || i == 255);
        cnt += 1;
    });
    assert_eq!(cnt, 2);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs_and_values() {
    let data: Vec<i64> = vec![4, -1, 1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 3);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping(U256::ONE << 255)
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.inverse_sign_mask()),
        TestScalar::from_wrapping(U256::MAX ^ (U256::FIVE | (U256::ONE << 255)))
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_index, i: u8| {
        assert!(i == 0 || i == 2 || i == 255);
        cnt += 1;
    });
    assert_eq!(cnt, 3);
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
        TestScalar::from_wrapping(dist.sign_mask()),
        TestScalar::from_wrapping((U256::ONE << 203) | (U256::ONE << 255))
    );
    assert_eq!(
        TestScalar::from_wrapping(dist.inverse_sign_mask()),
        TestScalar::from_wrapping(U256::MAX ^ ((U256::ONE << 203) | (U256::ONE << 255)))
    );

    let mut cnt = 0;
    dist.for_enumerated_vary_mask(|_i, _j| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}
