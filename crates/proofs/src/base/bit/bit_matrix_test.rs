use super::*;

use crate::base::bit::BitDistribution;
use crate::base::scalar::ArkScalar;
use bumpalo::Bump;
use num_traits::{One, Zero};

#[test]
fn we_can_compute_the_bit_matrix_of_empty_data() {
    let data: Vec<ArkScalar> = vec![];
    let dist = BitDistribution::new(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert!(matrix.is_empty());
}

#[test]
fn we_can_compute_the_bit_matrix_for_a_single_element() {
    let data: Vec<ArkScalar> = vec![ArkScalar::one()];
    let dist = BitDistribution::new(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert!(matrix.is_empty());
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_a_single_varying_bit() {
    let data: Vec<ArkScalar> = vec![ArkScalar::one(), ArkScalar::zero()];
    let dist = BitDistribution::new(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 1);
    let slice1 = vec![true, false];
    assert_eq!(matrix[0], slice1)
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_a_varying_sign_bit() {
    let data: Vec<ArkScalar> = vec![ArkScalar::one(), -ArkScalar::one()];
    let dist = BitDistribution::new(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 1);
    let slice1 = vec![false, true];
    assert_eq!(matrix[0], slice1)
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_varying_bits_in_different_positions() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(2), ArkScalar::one()];
    let dist = BitDistribution::new(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![false, true];
    let slice2 = vec![true, false];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_varying_bits_and_constant_bits() {
    let data: Vec<ArkScalar> = vec![ArkScalar::from(3), ArkScalar::from(-1)];
    let dist = BitDistribution::new(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![true, false];
    let slice2 = vec![false, true];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_entries_bigger_than_64_bit_integers() {
    let mut val = [0; 4];
    val[3] = 1 << 2;
    let data: Vec<ArkScalar> = vec![ArkScalar::from_bigint(val), ArkScalar::one()];
    let dist = BitDistribution::new(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![false, true];
    let slice2 = vec![true, false];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}
