use crate::base::scalar::{Curve25519Scalar, Scalar, ScalarConversionError};
use alloc::format;
use num_bigint::BigInt;
use num_traits::{Inv, One};

#[test]
fn test_dalek_interop_1() {
    let x = curve25519_dalek::scalar::Scalar::from(1u64);
    let xp = Curve25519Scalar::from(1u64);
    assert_eq!(curve25519_dalek::scalar::Scalar::from(xp), x);
}

#[test]
fn test_dalek_interop_m1() {
    let x = curve25519_dalek::scalar::Scalar::from(123u64);
    let mx = -x;
    let xp = Curve25519Scalar::from(123u64);
    let mxp = -xp;
    assert_eq!(mxp, Curve25519Scalar::from(-123i64));
    assert_eq!(curve25519_dalek::scalar::Scalar::from(mxp), mx);
}

#[test]
fn test_add() {
    let one = Curve25519Scalar::from(1u64);
    let two = Curve25519Scalar::from(2u64);
    let sum = one + two;
    let expected_sum = Curve25519Scalar::from(3u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_mod() {
    let pm1: ark_ff::BigInt<4> = ark_ff::BigInt!(
        "7237005577332262213973186563042994240857116359379907606001950938285454250988"
    );
    let x = Curve25519Scalar::from(pm1.0);
    let one = Curve25519Scalar::from(1u64);
    let zero = Curve25519Scalar::from(0u64);
    let xp1 = x + one;
    assert_eq!(xp1, zero);
}

#[test]
fn test_curve25519_scalar_serialization() {
    let s = [
        Curve25519Scalar::from(1u8),
        -Curve25519Scalar::from(1u8),
        Curve25519Scalar::from(123),
        Curve25519Scalar::from(0),
        Curve25519Scalar::from(255),
        Curve25519Scalar::from(1234),
        Curve25519Scalar::from(12345),
        Curve25519Scalar::from(2357),
        Curve25519Scalar::from(999),
        Curve25519Scalar::from(123_456_789),
    ];
    let serialized = serde_json::to_string(&s).unwrap();
    let deserialized: [Curve25519Scalar; 10] = serde_json::from_str(&serialized).unwrap();
    assert_eq!(s, deserialized);
}

#[test]
fn test_curve25519_scalar_display() {
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "1000000000000000000000000000000014DEF9DEA2F79CD65812631A5C4A12CA",
        format!("{}", Curve25519Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "0x0000...C123",
        format!("{:#}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "0x1000...12CA",
        format!("{:#}", Curve25519Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", Curve25519Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0x0000...C123",
        format!("{:+#}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0x0000...C123",
        format!("{:+#}", Curve25519Scalar::from(-0x00AB_C123))
    );
}

#[test]
fn test_curve25519_scalar_mid() {
    assert_eq!(
        Curve25519Scalar::MAX_SIGNED,
        -Curve25519Scalar::one() * Curve25519Scalar::from(2).inv().unwrap()
    );
}

#[test]
fn test_curve25519_scalar_to_bool() {
    assert!(!bool::try_from(Curve25519Scalar::ZERO).unwrap());
    assert!(bool::try_from(Curve25519Scalar::ONE).unwrap());
}

#[test]
fn test_curve25519_scalar_to_bool_overflow() {
    matches!(
        bool::try_from(Curve25519Scalar::from(2)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(Curve25519Scalar::from(-1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(Curve25519Scalar::from(-2)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i8() {
    assert_eq!(i8::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i8::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i8::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i8::try_from(Curve25519Scalar::from(i8::MAX)).unwrap(),
        i8::MAX
    );
    assert_eq!(
        i8::try_from(Curve25519Scalar::from(i8::MIN)).unwrap(),
        i8::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i8_overflow() {
    matches!(
        i8::try_from(Curve25519Scalar::from(i8::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i8::try_from(Curve25519Scalar::from(i8::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i16() {
    assert_eq!(i16::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i16::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i16::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i16::try_from(Curve25519Scalar::from(i16::MAX)).unwrap(),
        i16::MAX
    );
    assert_eq!(
        i16::try_from(Curve25519Scalar::from(i16::MIN)).unwrap(),
        i16::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i16_overflow() {
    matches!(
        i16::try_from(Curve25519Scalar::from(i16::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i16::try_from(Curve25519Scalar::from(i16::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i32() {
    assert_eq!(i32::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i32::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i32::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i32::try_from(Curve25519Scalar::from(i32::MAX)).unwrap(),
        i32::MAX
    );
    assert_eq!(
        i32::try_from(Curve25519Scalar::from(i32::MIN)).unwrap(),
        i32::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i32_overflow() {
    matches!(
        i32::try_from(Curve25519Scalar::from(i32::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i32::try_from(Curve25519Scalar::from(i32::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i64() {
    assert_eq!(i64::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i64::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i64::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i64::try_from(Curve25519Scalar::from(i64::MAX)).unwrap(),
        i64::MAX
    );
    assert_eq!(
        i64::try_from(Curve25519Scalar::from(i64::MIN)).unwrap(),
        i64::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i64_overflow() {
    matches!(
        i64::try_from(Curve25519Scalar::from(i64::MAX as i128 + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i64::try_from(Curve25519Scalar::from(i64::MIN as i128 - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i128() {
    assert_eq!(i128::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i128::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i128::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i128::try_from(Curve25519Scalar::from(i128::MAX)).unwrap(),
        i128::MAX
    );
    assert_eq!(
        i128::try_from(Curve25519Scalar::from(i128::MIN)).unwrap(),
        i128::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i128_overflow() {
    matches!(
        i128::try_from(Curve25519Scalar::from(i128::MAX) + Curve25519Scalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i128::try_from(Curve25519Scalar::from(i128::MIN) - Curve25519Scalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_bigint() {
    assert_eq!(BigInt::from(Curve25519Scalar::ZERO), BigInt::from(0_i8));
    assert_eq!(BigInt::from(Curve25519Scalar::ONE), BigInt::from(1_i8));
    assert_eq!(BigInt::from(-Curve25519Scalar::ONE), BigInt::from(-1_i8));
    assert_eq!(
        BigInt::from(Curve25519Scalar::from(i128::MAX)),
        BigInt::from(i128::MAX)
    );
    assert_eq!(
        BigInt::from(Curve25519Scalar::from(i128::MIN)),
        BigInt::from(i128::MIN)
    );
}

#[test]
fn test_curve25519_scalar_from_bigint() {
    assert_eq!(
        Curve25519Scalar::try_from(BigInt::from(0_i8)).unwrap(),
        Curve25519Scalar::ZERO
    );
    assert_eq!(
        Curve25519Scalar::try_from(BigInt::from(1_i8)).unwrap(),
        Curve25519Scalar::ONE
    );
    assert_eq!(
        Curve25519Scalar::try_from(BigInt::from(-1_i8)).unwrap(),
        -Curve25519Scalar::ONE
    );
}
