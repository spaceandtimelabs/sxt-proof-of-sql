use super::DoryScalar;
use crate::base::scalar::{Scalar, ScalarConversionError};
use core::cmp::Ordering;
use num_bigint::BigInt;

#[test]
fn test_dory_scalar_to_bool() {
    assert!(!bool::try_from(DoryScalar::ZERO).unwrap());
    assert!(bool::try_from(DoryScalar::ONE).unwrap());
}

#[test]
fn test_dory_scalar_to_bool_overflow() {
    matches!(
        bool::try_from(DoryScalar::from(2)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(DoryScalar::from(-1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(DoryScalar::from(-2)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_dory_scalar_to_i8() {
    assert_eq!(i8::try_from(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(i8::try_from(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(i8::try_from(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(i8::try_from(DoryScalar::from(i8::MAX)).unwrap(), i8::MAX);
    assert_eq!(i8::try_from(DoryScalar::from(i8::MIN)).unwrap(), i8::MIN);
}

#[test]
fn test_dory_scalar_to_i8_overflow() {
    matches!(
        i8::try_from(DoryScalar::from(i128::from(i8::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i8::try_from(DoryScalar::from(i128::from(i8::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_dory_scalar_to_i16() {
    assert_eq!(i16::try_from(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(i16::try_from(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(i16::try_from(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(i16::try_from(DoryScalar::from(i16::MAX)).unwrap(), i16::MAX);
    assert_eq!(i16::try_from(DoryScalar::from(i16::MIN)).unwrap(), i16::MIN);
}

#[test]
fn test_dory_scalar_to_i16_overflow() {
    matches!(
        i16::try_from(DoryScalar::from(i128::from(i16::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i16::try_from(DoryScalar::from(i128::from(i16::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_dory_scalar_to_i32() {
    assert_eq!(i32::try_from(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(i32::try_from(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(i32::try_from(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(i32::try_from(DoryScalar::from(i32::MAX)).unwrap(), i32::MAX);
    assert_eq!(i32::try_from(DoryScalar::from(i32::MIN)).unwrap(), i32::MIN);
}

#[test]
fn test_dory_scalar_to_i32_overflow() {
    matches!(
        i32::try_from(DoryScalar::from(i128::from(i32::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i32::try_from(DoryScalar::from(i128::from(i32::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_dory_scalar_to_i64() {
    assert_eq!(i64::try_from(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(i64::try_from(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(i64::try_from(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(i64::try_from(DoryScalar::from(i64::MAX)).unwrap(), i64::MAX);
    assert_eq!(i64::try_from(DoryScalar::from(i64::MIN)).unwrap(), i64::MIN);
}

#[test]
fn test_dory_scalar_to_i64_overflow() {
    matches!(
        i64::try_from(DoryScalar::from(i128::from(i64::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i64::try_from(DoryScalar::from(i128::from(i64::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_dory_scalar_to_i128() {
    assert_eq!(i128::try_from(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(i128::try_from(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(i128::try_from(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i128::try_from(DoryScalar::from(i128::MAX)).unwrap(),
        i128::MAX
    );
    assert_eq!(
        i128::try_from(DoryScalar::from(i128::MIN)).unwrap(),
        i128::MIN
    );
}

#[test]
fn test_dory_scalar_to_i128_overflow() {
    matches!(
        i128::try_from(DoryScalar::from(i128::MAX) + DoryScalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i128::try_from(DoryScalar::from(i128::MIN) - DoryScalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_dory_scalar_to_bigint() {
    assert_eq!(BigInt::from(DoryScalar::ZERO), BigInt::from(0_i8));
    assert_eq!(BigInt::from(DoryScalar::ONE), BigInt::from(1_i8));
    assert_eq!(BigInt::from(-DoryScalar::ONE), BigInt::from(-1_i8));
    assert_eq!(
        BigInt::from(DoryScalar::from(i128::MAX)),
        BigInt::from(i128::MAX)
    );
    assert_eq!(
        BigInt::from(DoryScalar::from(i128::MIN)),
        BigInt::from(i128::MIN)
    );
}

#[test]
fn scalar_comparison_works() {
    let zero = DoryScalar::ZERO;
    let one = DoryScalar::ONE;
    let two = DoryScalar::TWO;
    let max = DoryScalar::MAX_SIGNED;
    let min = max + one;
    assert_eq!(max.signed_cmp(&one), Ordering::Greater);
    assert_eq!(one.signed_cmp(&zero), Ordering::Greater);
    assert_eq!(min.signed_cmp(&zero), Ordering::Less);
    assert_eq!((two * max).signed_cmp(&zero), Ordering::Less);
    assert_eq!(two * max + one, zero);
}
