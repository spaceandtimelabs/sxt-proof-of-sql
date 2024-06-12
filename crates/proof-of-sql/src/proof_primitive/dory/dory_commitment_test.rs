use super::DoryScalar;
use crate::base::scalar::{Scalar, ScalarConversionError};
#[test]
fn test_dory_scalar_to_i8() {
    assert_eq!(TryInto::<i8>::try_into(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(TryInto::<i8>::try_into(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(TryInto::<i8>::try_into(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(
        TryInto::<i8>::try_into(DoryScalar::from(i8::MAX)).unwrap(),
        i8::MAX
    );
    assert_eq!(
        TryInto::<i8>::try_into(DoryScalar::from(i8::MIN)).unwrap(),
        i8::MIN
    );
}

#[test]
fn test_dory_scalar_to_i8_overflow() {
    matches!(
        TryInto::<i8>::try_into(DoryScalar::from(i8::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow(_))
    );
    matches!(
        TryInto::<i8>::try_into(DoryScalar::from(i8::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow(_))
    );
}

#[test]
fn test_dory_scalar_to_i16() {
    assert_eq!(TryInto::<i16>::try_into(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(TryInto::<i16>::try_into(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(TryInto::<i16>::try_into(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(
        TryInto::<i16>::try_into(DoryScalar::from(i16::MAX)).unwrap(),
        i16::MAX
    );
    assert_eq!(
        TryInto::<i16>::try_into(DoryScalar::from(i16::MIN)).unwrap(),
        i16::MIN
    );
}

#[test]
fn test_dory_scalar_to_i16_overflow() {
    matches!(
        TryInto::<i16>::try_into(DoryScalar::from(i16::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow(_))
    );
    matches!(
        TryInto::<i16>::try_into(DoryScalar::from(i16::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow(_))
    );
}

#[test]
fn test_dory_scalar_to_i32() {
    assert_eq!(TryInto::<i32>::try_into(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(TryInto::<i32>::try_into(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(TryInto::<i32>::try_into(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(
        TryInto::<i32>::try_into(DoryScalar::from(i32::MAX)).unwrap(),
        i32::MAX
    );
    assert_eq!(
        TryInto::<i32>::try_into(DoryScalar::from(i32::MIN)).unwrap(),
        i32::MIN
    );
}

#[test]
fn test_dory_scalar_to_i32_overflow() {
    matches!(
        TryInto::<i32>::try_into(DoryScalar::from(i32::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow(_))
    );
    matches!(
        TryInto::<i32>::try_into(DoryScalar::from(i32::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow(_))
    );
}

#[test]
fn test_dory_scalar_to_i64() {
    assert_eq!(TryInto::<i64>::try_into(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(TryInto::<i64>::try_into(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(TryInto::<i64>::try_into(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(
        TryInto::<i64>::try_into(DoryScalar::from(i64::MAX)).unwrap(),
        i64::MAX
    );
    assert_eq!(
        TryInto::<i64>::try_into(DoryScalar::from(i64::MIN)).unwrap(),
        i64::MIN
    );
}

#[test]
fn test_dory_scalar_to_i64_overflow() {
    matches!(
        TryInto::<i64>::try_into(DoryScalar::from(i64::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow(_))
    );
    matches!(
        TryInto::<i64>::try_into(DoryScalar::from(i64::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow(_))
    );
}

#[test]
fn test_dory_scalar_to_i128() {
    assert_eq!(TryInto::<i128>::try_into(DoryScalar::from(0)).unwrap(), 0);
    assert_eq!(TryInto::<i128>::try_into(DoryScalar::ONE).unwrap(), 1);
    assert_eq!(TryInto::<i128>::try_into(DoryScalar::from(-1)).unwrap(), -1);
    assert_eq!(
        TryInto::<i128>::try_into(DoryScalar::from(i128::MAX)).unwrap(),
        i128::MAX
    );
    assert_eq!(
        TryInto::<i128>::try_into(DoryScalar::from(i128::MIN)).unwrap(),
        i128::MIN
    );
}

#[test]
fn test_dory_scalar_to_i128_overflow() {
    matches!(
        TryInto::<i128>::try_into(DoryScalar::from(i128::MAX) + DoryScalar::ONE),
        Err(ScalarConversionError::Overflow(_))
    );
    matches!(
        TryInto::<i128>::try_into(DoryScalar::from(i128::MIN) - DoryScalar::ONE),
        Err(ScalarConversionError::Overflow(_))
    );
}
