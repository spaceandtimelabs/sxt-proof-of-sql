use super::{
    is_within_acceptable_range, verify_constant_abs_decomposition,
    verify_constant_sign_decomposition,
};
use crate::base::{
    bit::BitDistribution,
    scalar::{compute_commitment_for_testing, ArkScalar},
};
use blitzar::compute::get_one_commit;

#[test]
fn zero_is_within_range() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(0)];
    let dist = BitDistribution::new(&data);
    assert!(is_within_acceptable_range(&dist));
}

#[test]
fn the_sum_of_two_signed_128_bit_numbers_is_within_range() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(i128::MIN) + ArkScalar::from(i128::MIN)];
    let dist = BitDistribution::new(&data);
    assert!(is_within_acceptable_range(&dist));
}

#[test]
fn we_reject_distributions_that_are_outside_of_maximum_range() {
    let data: Vec<ArkScalar> =
        vec![ArkScalar::from(u128::MAX) + ArkScalar::from(u128::MAX) + ArkScalar::from(u128::MAX)];
    let dist = BitDistribution::new(&data);
    assert!(!is_within_acceptable_range(&dist));
}

#[test]
fn we_can_verify_the_decomposition_of_a_constant_column() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(1234), ArkScalar::from(1234)];
    let dist = BitDistribution::new(&data);
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    assert!(verify_constant_sign_decomposition(&dist, &commit, &one_commit, &[]).is_ok());
}

#[test]
fn we_can_verify_the_decomposition_of_a_column_with_constant_sign() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(123), ArkScalar::from(122)];
    let dist = BitDistribution::new(&data);
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    let bits = [compute_commitment_for_testing(&[1, 0], 0)];
    assert!(verify_constant_sign_decomposition(&dist, &commit, &one_commit, &bits).is_ok());
}

#[test]
fn we_can_verify_the_decomposition_of_a_constant_column_with_negative_values() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(-1234), ArkScalar::from(-1234)];
    let dist = BitDistribution::new(&data);
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    assert!(verify_constant_sign_decomposition(&dist, &commit, &one_commit, &[]).is_ok());
}

#[test]
fn constant_verification_fails_if_the_commitment_doesnt_match() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(1234), ArkScalar::from(1234)];
    let dist = BitDistribution::new(&data);
    let data: Vec<ArkScalar> = vec![ArkScalar::from(1235), ArkScalar::from(1234)];
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    assert!(verify_constant_sign_decomposition(&dist, &commit, &one_commit, &[]).is_err());
}

#[test]
fn constant_verification_fails_if_the_sign_bit_doesnt_match() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(1234), ArkScalar::from(1234)];
    let dist = BitDistribution::new(&data);
    let data: Vec<ArkScalar> = vec![ArkScalar::from(-1234), ArkScalar::from(-1234)];
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    assert!(verify_constant_sign_decomposition(&dist, &commit, &one_commit, &[]).is_err());
}

#[test]
fn constant_verification_fails_if_a_varying_bit_doesnt_match() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(1234), ArkScalar::from(1234)];
    let dist = BitDistribution::new(&data);
    let data: Vec<ArkScalar> = vec![ArkScalar::from(234), ArkScalar::from(1234)];
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    assert!(verify_constant_sign_decomposition(&dist, &commit, &one_commit, &[]).is_err());
}

#[test]
fn we_can_verify_a_decomposition_with_only_a_varying_sign() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(-1), ArkScalar::from(1)];
    let dist = BitDistribution::new(&data);
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    let sign_commit = compute_commitment_for_testing(&[1, 0], 0);
    assert!(verify_constant_abs_decomposition(&dist, &commit, &one_commit, &sign_commit).is_ok());
}

#[test]
fn constant_abs_verification_fails_if_the_sign_and_data_dont_match() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(-1), ArkScalar::from(1)];
    let dist = BitDistribution::new(&data);
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    let sign_commit = compute_commitment_for_testing(&[0, 1], 0);
    assert!(verify_constant_abs_decomposition(&dist, &commit, &one_commit, &sign_commit).is_err());
}

#[test]
fn we_can_verify_a_decomposition_with_only_a_varying_sign_and_magnitude_greater_than_one() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(-100), ArkScalar::from(100)];
    let dist = BitDistribution::new(&data);
    let commit = compute_commitment_for_testing(&data, 0);
    let one_commit = get_one_commit(data.len() as u64);
    let sign_commit = compute_commitment_for_testing(&[1, 0], 0);
    assert!(verify_constant_abs_decomposition(&dist, &commit, &one_commit, &sign_commit).is_ok());
}
