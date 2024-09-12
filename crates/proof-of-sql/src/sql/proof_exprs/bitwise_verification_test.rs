use super::{
    is_within_acceptable_range, verify_constant_abs_decomposition,
    verify_constant_sign_decomposition,
};
use crate::base::{
    bit::BitDistribution,
    scalar::test_scalar::TestScalar,
    slice_ops::{inner_product, slice_cast},
};
use ark_std::UniformRand;
use core::iter::repeat_with;

fn rand_eval_vec(len: usize) -> Vec<TestScalar> {
    let rng = &mut ark_std::test_rng();
    repeat_with(|| TestScalar::rand(rng)).take(len).collect()
}

#[test]
fn zero_is_within_range() {
    let data: Vec<TestScalar> = vec![TestScalar::from(0)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert!(is_within_acceptable_range(&dist));
}

#[test]
fn the_sum_of_two_signed_128_bit_numbers_is_within_range() {
    let data: Vec<TestScalar> = vec![TestScalar::from(i128::MIN) + TestScalar::from(i128::MIN)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert!(is_within_acceptable_range(&dist));
}

#[test]
fn we_reject_distributions_that_are_outside_of_maximum_range() {
    let data: Vec<TestScalar> = vec![
        TestScalar::from(u128::MAX) + TestScalar::from(u128::MAX) + TestScalar::from(u128::MAX),
    ];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    assert!(!is_within_acceptable_range(&dist));
}

#[test]
fn we_can_verify_the_decomposition_of_a_constant_column() {
    let data: Vec<TestScalar> = vec![TestScalar::from(1234), TestScalar::from(1234)];
    let eval_vec: Vec<TestScalar> = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    assert!(verify_constant_sign_decomposition(&dist, data_eval, one_eval, &[]).is_ok());
}

#[test]
fn we_can_verify_the_decomposition_of_a_column_with_constant_sign() {
    let data: Vec<TestScalar> = vec![TestScalar::from(123), TestScalar::from(122)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    let bits = [inner_product(&slice_cast(&[1, 0]), &eval_vec)];
    assert!(verify_constant_sign_decomposition(&dist, data_eval, one_eval, &bits).is_ok());
}

#[test]
fn we_can_verify_the_decomposition_of_a_constant_column_with_negative_values() {
    let data: Vec<TestScalar> = vec![TestScalar::from(-1234), TestScalar::from(-1234)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    assert!(verify_constant_sign_decomposition(&dist, data_eval, one_eval, &[]).is_ok());
}

#[test]
fn constant_verification_fails_if_the_commitment_doesnt_match() {
    let data: Vec<TestScalar> = vec![TestScalar::from(1234), TestScalar::from(1234)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data: Vec<TestScalar> = vec![TestScalar::from(1235), TestScalar::from(1234)];
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    assert!(verify_constant_sign_decomposition(&dist, data_eval, one_eval, &[]).is_err());
}

#[test]
fn constant_verification_fails_if_the_sign_bit_doesnt_match() {
    let data: Vec<TestScalar> = vec![TestScalar::from(1234), TestScalar::from(1234)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data: Vec<TestScalar> = vec![TestScalar::from(-1234), TestScalar::from(-1234)];
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    assert!(verify_constant_sign_decomposition(&dist, data_eval, one_eval, &[]).is_err());
}

#[test]
fn constant_verification_fails_if_a_varying_bit_doesnt_match() {
    let data: Vec<TestScalar> = vec![TestScalar::from(1234), TestScalar::from(1234)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data: Vec<TestScalar> = vec![TestScalar::from(234), TestScalar::from(1234)];
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    assert!(verify_constant_sign_decomposition(&dist, data_eval, one_eval, &[]).is_err());
}

#[test]
fn we_can_verify_a_decomposition_with_only_a_varying_sign() {
    let data: Vec<TestScalar> = vec![TestScalar::from(-1), TestScalar::from(1)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    let sign_eval = inner_product(&slice_cast(&[1, 0]), &eval_vec);
    assert!(verify_constant_abs_decomposition(&dist, data_eval, one_eval, sign_eval).is_ok());
}

#[test]
fn constant_abs_verification_fails_if_the_sign_and_data_dont_match() {
    let data: Vec<TestScalar> = vec![TestScalar::from(-1), TestScalar::from(1)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    let sign_eval = inner_product(&slice_cast(&[0, 1]), &eval_vec);
    assert!(verify_constant_abs_decomposition(&dist, data_eval, one_eval, sign_eval).is_err());
}

#[test]
fn we_can_verify_a_decomposition_with_only_a_varying_sign_and_magnitude_greater_than_one() {
    let data: Vec<TestScalar> = vec![TestScalar::from(-100), TestScalar::from(100)];
    let eval_vec = rand_eval_vec(data.len());
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let data_eval = inner_product(&data, &eval_vec);
    let one_eval = eval_vec.iter().sum();
    let sign_eval = inner_product(&slice_cast(&[1, 0]), &eval_vec);
    assert!(verify_constant_abs_decomposition(&dist, data_eval, one_eval, sign_eval).is_ok());
}
