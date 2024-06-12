use super::*;
use crate::base::scalar::Curve25519Scalar;

#[test]
fn test_inner_product() {
    let a = vec![1, 2, 3, 4];
    let b = vec![2, 3, 4, 5];
    assert_eq!(40, inner_product(&a, &b));
}

/// test inner products of different lengths
#[test]
fn test_inner_product_different_lengths() {
    let a = vec![1, 2, 3, 4];
    let b = vec![2, 3, 4, 5, 6];
    assert_eq!(40, inner_product(&a, &b));
}

/// test inner producr with scalar
#[test]
fn test_inner_product_scalar() {
    let a = vec![Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let b = vec![Curve25519Scalar::from(2u64), Curve25519Scalar::from(3u64)];
    assert_eq!(Curve25519Scalar::from(8u64), inner_product(&a, &b));
}

/// test uneven inner product with scalars
#[test]
fn test_inner_product_scalar_uneven() {
    let a = vec![Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let b = vec![
        Curve25519Scalar::from(2u64),
        Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(4u64),
    ];
    assert_eq!(Curve25519Scalar::from(8u64), inner_product(&a, &b));
}

/// test inner product with curve25519scalar
#[test]
fn test_inner_product_curve25519scalar() {
    let a = vec![Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let b = vec![Curve25519Scalar::from(2u64), Curve25519Scalar::from(3u64)];
    assert_eq!(Curve25519Scalar::from(8u64), inner_product(&a, &b));
}

/// test uneven inner product with curve25519scalars
#[test]
fn test_inner_product_curve25519scalar_uneven() {
    let a = vec![Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let b = vec![
        Curve25519Scalar::from(2u64),
        Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(4u64),
    ];
    assert_eq!(Curve25519Scalar::from(8u64), inner_product(&a, &b));
}
