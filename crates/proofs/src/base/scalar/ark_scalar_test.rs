use crate::base::scalar::{ark_scalar::*, Scalar};
use ark_ff::BigInt;
use num_traits::{Inv, One};

#[test]
fn test_dalek_interop_1() {
    let x = curve25519_dalek::scalar::Scalar::from(1u64);
    let xp = ArkScalar::from(1u64);
    assert_eq!(curve25519_dalek::scalar::Scalar::from(xp), x);
}

#[test]
fn test_dalek_interop_m1() {
    let x = curve25519_dalek::scalar::Scalar::from(123u64);
    let mx = -x;
    let xp = ArkScalar::from(123u64);
    let mxp = -xp;
    assert_eq!(mxp, ArkScalar::from(-123i64));
    assert_eq!(curve25519_dalek::scalar::Scalar::from(mxp), mx);
}

#[test]
fn test_add() {
    let one = ArkScalar::from(1u64);
    let two = ArkScalar::from(2u64);
    let sum = one + two;
    let expected_sum = ArkScalar::from(3u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_mod() {
    let pm1: BigInt<4> =
        BigInt!("7237005577332262213973186563042994240857116359379907606001950938285454250988");
    let x = ArkScalar::from(pm1);
    let one = ArkScalar::from(1u64);
    let zero = ArkScalar::from(0u64);
    let xp1 = x + one;
    assert_eq!(xp1, zero);
}

#[test]
fn test_ark_scalar_serialization() {
    let s = [
        ArkScalar::from(1u8),
        -ArkScalar::from(1u8),
        ArkScalar::from(123),
        ArkScalar::from(0),
        ArkScalar::from(255),
        ArkScalar::from(1234),
        ArkScalar::from(12345),
        ArkScalar::from(2357),
        ArkScalar::from(999),
        ArkScalar::from(123456789),
    ];
    let serialized = serde_json::to_string(&s).unwrap();
    let deserialized: [ArkScalar; 10] = serde_json::from_str(&serialized).unwrap();
    assert_eq!(s, deserialized);
}

#[test]
fn test_ark_scalar_display() {
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{}", ArkScalar::from(0xABC123))
    );
    assert_eq!(
        "1000000000000000000000000000000014DEF9DEA2F79CD65812631A5C4A12CA",
        format!("{}", ArkScalar::from(-0xABC123))
    );
    assert_eq!("0x0000...C123", format!("{:#}", ArkScalar::from(0xABC123)));
    assert_eq!("0x1000...12CA", format!("{:#}", ArkScalar::from(-0xABC123)));
    assert_eq!(
        "+0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", ArkScalar::from(0xABC123))
    );
    assert_eq!(
        "-0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", ArkScalar::from(-0xABC123))
    );
    assert_eq!(
        "+0x0000...C123",
        format!("{:+#}", ArkScalar::from(0xABC123))
    );
    assert_eq!(
        "-0x0000...C123",
        format!("{:+#}", ArkScalar::from(-0xABC123))
    );
}

#[test]
fn test_ark_scalar_mid() {
    assert_eq!(
        ArkScalar::MAX_SIGNED,
        -ArkScalar::one() * ArkScalar::from(2).inv()
    );
}
