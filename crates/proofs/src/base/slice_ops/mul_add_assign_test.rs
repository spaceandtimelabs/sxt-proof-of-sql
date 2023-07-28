use crate::base::scalar::ArkScalar;

use super::*;

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

/// test mul_add_assign with arkscalar
#[test]
fn test_mul_add_assign_arkscalar() {
    let mut a = vec![ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let b = vec![ArkScalar::from(2u64), ArkScalar::from(3u64)];
    mul_add_assign(&mut a, ArkScalar::from(10u64), &b);
    let c = vec![
        ArkScalar::from(1u64) + ArkScalar::from(10u64) * ArkScalar::from(2u64),
        ArkScalar::from(2u64) + ArkScalar::from(10u64) * ArkScalar::from(3u64),
    ];
    assert_eq!(a, c);
}

/// test mul_add_assign with uneven arkscalars
#[test]
fn test_mul_add_assign_arkscalar_uneven() {
    let mut a = vec![
        ArkScalar::from(1u64),
        ArkScalar::from(2u64),
        ArkScalar::from(3u64),
    ];
    let b = vec![ArkScalar::from(2u64), ArkScalar::from(3u64)];
    mul_add_assign(&mut a, ArkScalar::from(10u64), &b);
    let c = vec![
        ArkScalar::from(1u64) + ArkScalar::from(10u64) * ArkScalar::from(2u64),
        ArkScalar::from(2u64) + ArkScalar::from(10u64) * ArkScalar::from(3u64),
        ArkScalar::from(3u64),
    ];
    assert_eq!(a, c);
}
