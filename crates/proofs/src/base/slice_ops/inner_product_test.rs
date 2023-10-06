use super::*;
use crate::base::scalar::ArkScalar;

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
    let a = vec![ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let b = vec![ArkScalar::from(2u64), ArkScalar::from(3u64)];
    assert_eq!(ArkScalar::from(8u64), inner_product(&a, &b));
}

/// test uneven inner product with scalars
#[test]
fn test_inner_product_scalar_uneven() {
    let a = vec![ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let b = vec![
        ArkScalar::from(2u64),
        ArkScalar::from(3u64),
        ArkScalar::from(4u64),
    ];
    assert_eq!(ArkScalar::from(8u64), inner_product(&a, &b));
}

/// test inner product with arkscalar
#[test]
fn test_inner_product_arkscalar() {
    let a = vec![ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let b = vec![ArkScalar::from(2u64), ArkScalar::from(3u64)];
    assert_eq!(ArkScalar::from(8u64), inner_product(&a, &b));
}

/// test uneven inner product with arkscalars
#[test]
fn test_inner_product_arkscalar_uneven() {
    let a = vec![ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let b = vec![
        ArkScalar::from(2u64),
        ArkScalar::from(3u64),
        ArkScalar::from(4u64),
    ];
    assert_eq!(ArkScalar::from(8u64), inner_product(&a, &b));
}
