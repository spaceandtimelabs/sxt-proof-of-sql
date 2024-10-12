use super::*;
use crate::base::{bit::BitDistribution, scalar::Curve25519Scalar};
use bumpalo::Bump;
use num_traits::{One, Zero};

#[test]
fn we_can_compute_the_bit_matrix_of_empty_data() {
    let data: Vec<Curve25519Scalar> = vec![];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert!(matrix.is_empty());
}

#[test]
fn we_can_compute_the_bit_matrix_for_a_single_element() {
    let data: Vec<Curve25519Scalar> = vec![Curve25519Scalar::one()];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert!(matrix.is_empty());
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_a_single_varying_bit() {
    let data: Vec<Curve25519Scalar> = vec![Curve25519Scalar::one(), Curve25519Scalar::zero()];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 1);
    let slice1 = vec![true, false];
    assert_eq!(matrix[0], slice1);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_a_varying_sign_bit() {
    let data: Vec<Curve25519Scalar> = vec![Curve25519Scalar::one(), -Curve25519Scalar::one()];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 1);
    let slice1 = vec![false, true];
    assert_eq!(matrix[0], slice1);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_varying_bits_in_different_positions() {
    let data: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(2), Curve25519Scalar::one()];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
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
    let data: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(3), Curve25519Scalar::from(-1)];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
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
    let data: Vec<Curve25519Scalar> =
        vec![Curve25519Scalar::from_bigint(val), Curve25519Scalar::one()];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![false, true];
    let slice2 = vec![true, false];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}
