use super::*;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn test_mul_add_assign() {
    let mut a = vec![1, 2, 3, 4];
    let b = vec![2, 3, 4, 5];
    mul_add_assign(&mut a, 10, &b);
    let c = vec![1 + 10 * 2, 2 + 10 * 3, 3 + 10 * 4, 4 + 10 * 5];
    assert_eq!(a, c);
}

/// test mul_add_assign with uneven vectors
#[test]
fn test_mul_add_assign_uneven() {
    let mut a = vec![1, 2, 3, 4, 5];
    let b = vec![2, 3, 4, 5];
    mul_add_assign(&mut a, 10, &b);
    let c = vec![1 + 10 * 2, 2 + 10 * 3, 3 + 10 * 4, 4 + 10 * 5, 5];
    assert_eq!(a, c);
}

/// test mul_add_assign with with uneven panics when len(a) < len(b)
#[test]
#[should_panic]
fn test_mul_add_assign_uneven_panic() {
    let mut a = vec![1, 2, 3, 4];
    let b = vec![2, 3, 4, 5, 6];
    mul_add_assign(&mut a, 10, &b);
}

/// test mul_add_assign with TestScalar
#[test]
fn test_mul_add_assign_testscalar() {
    let mut a = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b = vec![TestScalar::from(2u64), TestScalar::from(3u64)];
    mul_add_assign(&mut a, TestScalar::from(10u64), &b);
    let c = vec![
        TestScalar::from(1u64) + TestScalar::from(10u64) * TestScalar::from(2u64),
        TestScalar::from(2u64) + TestScalar::from(10u64) * TestScalar::from(3u64),
    ];
    assert_eq!(a, c);
}

/// test mul_add_assign with uneven TestScalars
#[test]
fn test_mul_add_assign_testscalar_uneven() {
    let mut a = vec![
        TestScalar::from(1u64),
        TestScalar::from(2u64),
        TestScalar::from(3u64),
    ];
    let b = vec![TestScalar::from(2u64), TestScalar::from(3u64)];
    mul_add_assign(&mut a, TestScalar::from(10u64), &b);
    let c = vec![
        TestScalar::from(1u64) + TestScalar::from(10u64) * TestScalar::from(2u64),
        TestScalar::from(2u64) + TestScalar::from(10u64) * TestScalar::from(3u64),
        TestScalar::from(3u64),
    ];
    assert_eq!(a, c);
}
