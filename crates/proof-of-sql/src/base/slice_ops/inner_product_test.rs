use super::*;
use crate::base::scalar::{test_scalar::TestScalar, ScalarExt};

#[test]
fn test_inner_product_with_bytes_basic() {
    // Suppose we have 3 rows of varbinary data
    let lhs = vec![b"abc".to_vec(), b"xyz".to_vec(), b"foo".to_vec()];
    // We'll multiply by [10, 20, 30]
    let rhs = vec![
        TestScalar::from(10),
        TestScalar::from(20),
        TestScalar::from(30),
    ];
    // The “actual” product
    let product = inner_product_with_bytes(&lhs, &rhs);

    // Build the “manual” approach: for each lhs row, do the keccak-based hash → scalar, sum
    let expected = lhs
        .iter()
        .zip(rhs.iter())
        .map(|(bytes, &sc)| TestScalar::from_byte_slice_via_hash(bytes) * sc)
        .sum::<TestScalar>();

    assert_eq!(product, expected);
}

#[test]
fn test_inner_product_with_bytes_uneven() {
    // LHS has 2 entries, RHS has 3
    let lhs = vec![b"foo".to_vec(), b"bar".to_vec()];
    let rhs = vec![
        TestScalar::from(5),
        TestScalar::from(6),
        TestScalar::from(7),
    ];
    // Actual
    let product = inner_product_with_bytes(&lhs, &rhs);
    // Manual
    let expected = lhs
        .iter()
        .zip(rhs.iter()) // stops at the shorter length (2)
        .map(|(bytes, &sc)| TestScalar::from_byte_slice_via_hash(bytes) * sc)
        .sum::<TestScalar>();
    assert_eq!(product, expected);
}

#[test]
fn test_inner_product_with_bytes_empty_lhs() {
    // Both empty
    let lhs: Vec<Vec<u8>> = vec![];
    let rhs: Vec<TestScalar> = vec![];
    assert_eq!(TestScalar::from(0), inner_product_with_bytes(&lhs, &rhs));
}

#[test]
fn test_inner_product_with_bytes_partial_fits() {
    // LHS has 2, RHS has 1
    // Only 1 pair used
    let lhs = vec![b"abc".to_vec(), b"xyz".to_vec()];
    let rhs = vec![TestScalar::from(100)];
    let product = inner_product_with_bytes(&lhs, &rhs);
    // Manual
    let expected = TestScalar::from_byte_slice_via_hash(b"abc") * TestScalar::from(100);
    assert_eq!(product, expected);
}

#[test]
fn test_inner_product_with_bytes_longest_rhs() {
    // LHS has 3, RHS has 5
    let lhs = vec![b"abc".to_vec(), b"xyz".to_vec(), b"foo".to_vec()];
    let rhs = vec![
        TestScalar::from(10),
        TestScalar::from(20),
        TestScalar::from(30),
        TestScalar::from(40),
        TestScalar::from(50),
    ];
    // Only first 3 pairs are used
    let product = inner_product_with_bytes(&lhs, &rhs);
    let expected = [(b"abc", 10), (b"xyz", 20), (b"foo", 30)]
        .iter()
        .map(|(bytes, sc)| {
            TestScalar::from_byte_slice_via_hash(bytes.as_ref()) * TestScalar::from(*sc)
        })
        .sum::<TestScalar>();
    assert_eq!(product, expected);
}

#[test]
fn test_inner_product_with_bytes_some_edge_cases() {
    // test small + large + empty strings
    let lhs = vec![
        b"".to_vec(),
        b"\x00".to_vec(),
        b"some big data in here ...".repeat(4).clone(), // repeated -> bigger
    ];
    let rhs = vec![
        TestScalar::from(5),
        TestScalar::from(10),
        TestScalar::from(15),
    ];
    let product = inner_product_with_bytes(&lhs, &rhs);
    let expected = lhs
        .iter()
        .zip(rhs.iter())
        .map(|(bts, &sc)| TestScalar::from_byte_slice_via_hash(bts) * sc)
        .sum::<TestScalar>();
    assert_eq!(product, expected);
}

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
