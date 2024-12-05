use super::*;
use crate::base::scalar::test_scalar::TestScalar;
use num_traits::Zero;

#[test]
fn we_can_compute_the_bit_distribution_of_an_empty_slice() {
    let data: Vec<i64> = vec![];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 256);
    assert!(dist.is_valid());
    assert_eq!(TestScalar::from(dist.sign_mask), TestScalar::zero());
    assert_eq!(TestScalar::from(dist.inverse_sign_mask), TestScalar::zero());

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 256);
}

#[test]
fn we_can_compute_the_bit_distribution_of_a_slice_with_a_single_element() {
    let val = (1 << 2) | (1 << 10);
    let data: Vec<i64> = vec![val];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from(dist.sign_mask),
        TestScalar::from(val) + TestScalar::from([0u64, 0, 0, 1 << 63])
    );
    assert_eq!(
        TestScalar::from(dist.inverse_sign_mask),
        TestScalar::from([
            u64::MAX ^ ((1 << 2) | (1 << 10)),
            u64::MAX,
            u64::MAX,
            u64::MAX ^ 1 << 63
        ])
    );

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
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
        TestScalar::from(dist.sign_mask),
        TestScalar::from((1 << 10) | (1 << 2)) + TestScalar::from([0u64, 0, 0, 1 << 63])
    );
    assert_eq!(
        TestScalar::from(dist.inverse_sign_mask),
        TestScalar::from([
            u64::MAX ^ ((1 << 2) | (1 << 10) | (1 << 21)),
            u64::MAX,
            u64::MAX,
            u64::MAX ^ 1 << 63
        ])
    );

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
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 4);
    assert!(dist.is_valid());

    assert_eq!(
        TestScalar::from(dist.sign_mask),
        TestScalar::from(1 << 10) + TestScalar::from([0u64, 0, 0, 1 << 63])
    );
    assert_eq!(
        TestScalar::from(dist.inverse_sign_mask),
        TestScalar::from([
            u64::MAX ^ ((1 << 2) | (1 << 3) | (1 << 10) | (1 << 21) | (1 << 50)),
            u64::MAX,
            u64::MAX,
            u64::MAX ^ 1 << 63
        ])
    );

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
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 0);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from(dist.sign_mask),
        TestScalar::from([0, 0, 0, 1 << 63])
    );
    assert_eq!(
        TestScalar::from(dist.inverse_sign_mask),
        TestScalar::from([u64::MAX, u64::MAX, u64::MAX, u64::MAX ^ 1 << 63])
    );

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs() {
    let data: Vec<i64> = vec![-1, 1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 1);
    assert_eq!(
        TestScalar::from(dist.sign_mask),
        TestScalar::from([0, 0, 0, 1 << 63])
    );
    assert_eq!(
        TestScalar::from(dist.inverse_sign_mask),
        TestScalar::from([u64::MAX ^ 1, u64::MAX, u64::MAX, u64::MAX ^ 1 << 63])
    );

    let mut cnt = 0;
    dist.for_each_varying_bit(|i: usize, pos: usize| {
        assert_eq!(i, 0);
        assert_eq!(pos, 0);
        cnt += 1;
    });
    assert_eq!(cnt, 1);
}

#[test]
fn we_can_compute_the_bit_distribution_of_values_with_different_signs_and_values() {
    let data: Vec<i64> = vec![4, -1, 1];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert_eq!(dist.num_varying_bits(), 2);
    assert!(dist.is_valid());
    assert_eq!(
        TestScalar::from(dist.sign_mask),
        TestScalar::from([0, 0, 0, 1 << 63])
    );
    assert_eq!(
        TestScalar::from(dist.inverse_sign_mask),
        TestScalar::from([u64::MAX ^ 5, u64::MAX, u64::MAX, u64::MAX ^ 1 << 63])
    );

    let mut cnt = 0;
    dist.for_each_varying_bit(|i: usize, pos: usize| {
        assert!((i == 0 && (pos == 0 || pos == 2)));
        cnt += 1;
    });
    assert_eq!(cnt, 2);
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
        TestScalar::from(dist.sign_mask),
        TestScalar::from([0, 0, 0, (1 << 11) | (1 << 63)])
    );
    assert_eq!(
        TestScalar::from(dist.inverse_sign_mask),
        TestScalar::from([
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX ^ ((1 << 11) | (1 << 63))
        ])
    );

    let mut cnt = 0;
    dist.for_each_varying_bit(|_i: usize, _pos: usize| {
        cnt += 1;
    });
    assert_eq!(cnt, 0);
}

#[test]
fn we_can_detect_invalid_bit_distributions() {
    let dist = BitDistribution {
        sign_mask: [1, 0, 0, 0],
        inverse_sign_mask: [1, 0, 0, 0],
    };
    assert!(!dist.is_valid());
}
