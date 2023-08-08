use super::*;

use crate::base::scalar::ArkScalar;
use num_traits::{One, Zero};

#[test]
fn we_can_compute_the_bit_distribution_of_an_empty_slice() {
    let data: Vec<i64> = vec![];
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(!dist.has_varying_sign_bit());
    assert!(!dist.sign_bit());
    assert!(dist.is_valid());
    assert_eq!(dist.constant_part(), ArkScalar::zero());

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_a_single_element() {
    let val = (1 << 2) | (1 << 10);
    let data: Vec<i64> = vec![val];
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(!dist.has_varying_sign_bit());
    assert!(!dist.sign_bit());
    assert!(dist.is_valid());
    assert_eq!(dist.constant_part(), ArkScalar::from(val));
    assert_eq!(dist.most_significant_abs_bit(), 10);

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert!(pos == 2 || pos == 10);
        cnt += 1;
    });
    assert_eq!(cnt, 2);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_one_varying_bits() {
    let data: Vec<i64> = vec![(1 << 2) | (1 << 10), (1 << 2) | (1 << 10) | (1 << 21)];
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 1);
    assert!(!dist.has_varying_sign_bit());
    assert!(!dist.sign_bit());
    assert!(dist.is_valid());
    assert_eq!(dist.constant_part(), ArkScalar::from((1 << 10) | (1 << 2)));
    assert_eq!(dist.most_significant_abs_bit(), 21);

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert!(pos == 2 || pos == 10);
        cnt += 1;
    });
    assert_eq!(cnt, 2);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert_eq!(pos, 21);
        cnt += 1;
    });
    assert_eq!(cnt, 1);

    let mut cnt = 0;
    dist.for_each_varying_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert_eq!(pos, 21);
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
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 4);
    assert!(!dist.has_varying_sign_bit());
    assert!(!dist.sign_bit());
    assert!(dist.is_valid());
    assert_eq!(dist.constant_part(), ArkScalar::from(1 << 10));
    assert_eq!(dist.most_significant_abs_bit(), 50);

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert_eq!(pos, 10);
        cnt += 1;
    });
    assert_eq!(cnt, 1);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert!(pos == 2 || pos == 3 || pos == 21 || pos == 50);
        cnt += 1;
    });
    assert_eq!(cnt, 4);

    let mut cnt = 0;
    dist.for_each_varying_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert!(pos == 2 || pos == 3 || pos == 21 || pos == 50);
        cnt += 1;
    });
    assert_eq!(cnt, 4);
}

#[test]
fn we_can_compute_the_bit_distribution_of_negative_values() {
    let data: Vec<i64> = vec![-1];
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(!dist.has_varying_sign_bit());
    assert!(dist.sign_bit());
    assert!(dist.is_valid());
    assert_eq!(dist.constant_part(), ArkScalar::one());
    assert_eq!(dist.most_significant_abs_bit(), 0);

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert_eq!(pos, 0);
        cnt += 1;
    });
    assert_eq!(cnt, 1);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs() {
    let data: Vec<i64> = vec![-1, 1];
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 1);
    assert!(dist.has_varying_sign_bit());
    assert_eq!(dist.constant_part(), ArkScalar::one());
    assert_eq!(dist.most_significant_abs_bit(), 0);

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert_eq!(pos, 0);
        cnt += 1;
    });
    assert_eq!(cnt, 1);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);

    let mut cnt = 0;
    dist.for_each_varying_bit(|i: usize, pos: usize| {
        assert_eq!(i, 3);
        assert_eq!(pos, 63);
        cnt += 1;
    });
    assert_eq!(cnt, 1);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs_and_values() {
    let data: Vec<i64> = vec![4, -1, 1];
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 3);
    assert!(dist.has_varying_sign_bit());
    assert!(dist.is_valid());
    assert_eq!(dist.constant_part(), ArkScalar::zero());
    assert_eq!(dist.most_significant_abs_bit(), 2);

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert!(pos == 0 || pos == 2);
        cnt += 1;
    });
    assert_eq!(cnt, 2);

    let mut cnt = 0;
    dist.for_each_varying_bit(|i: usize, pos: usize| {
        assert!((i == 0 && (pos == 0 || pos == 2)) || (i == 3 && pos == 63));
        cnt += 1;
    });
    assert_eq!(cnt, 3);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_larger_than_64_bit_integers() {
    let mut val = [0; 4];
    val[3] = 1 << 11;
    let data: Vec<ArkScalar> = vec![ArkScalar::from_bigint(val)];
    let dist = BitDistribution::new(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(!dist.has_varying_sign_bit());
    assert!(dist.is_valid());
    assert_eq!(dist.constant_part(), ArkScalar::from_bigint(val));
    assert_eq!(dist.most_significant_abs_bit(), 64 * 3 + 11);

    let mut cnt = 0;
    dist.for_each_abs_constant_bit(|i: usize, pos: usize| {
        assert_eq!(i, 3);
        assert_eq!(pos, 11);
        cnt += 1;
    });
    assert_eq!(cnt, 1);

    let mut cnt = 0;
    dist.for_each_abs_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_detect_invalid_bit_distributions() {
    let dist = BitDistribution {
        or_all: [0, 0, 0, 0],
        vary_mask: [1, 0, 0, 0],
    };
    assert!(!dist.is_valid());
}
