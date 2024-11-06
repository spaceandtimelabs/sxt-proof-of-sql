use super::*;
use crate::base::scalar::test_scalar::TestScalar;

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
    let a = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b = vec![TestScalar::from(2u64), TestScalar::from(3u64)];
    assert_eq!(TestScalar::from(8u64), inner_product(&a, &b));
}

/// test uneven inner product with scalars
#[test]
fn test_inner_product_scalar_uneven() {
    let a = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b = vec![
        TestScalar::from(2u64),
        TestScalar::from(3u64),
        TestScalar::from(4u64),
    ];
    assert_eq!(TestScalar::from(8u64), inner_product(&a, &b));
}

/// test inner product with `TestScalar`
#[test]
fn test_inner_product_testscalar() {
    let a = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b = vec![TestScalar::from(2u64), TestScalar::from(3u64)];
    assert_eq!(TestScalar::from(8u64), inner_product(&a, &b));
}

/// test uneven inner product with `TestScalar`
#[test]
fn test_inner_product_testscalar_uneven() {
    let a = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b = vec![
        TestScalar::from(2u64),
        TestScalar::from(3u64),
        TestScalar::from(4u64),
    ];
    assert_eq!(TestScalar::from(8u64), inner_product(&a, &b));
}
