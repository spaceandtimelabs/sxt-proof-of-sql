use crate::base::polynomial::ark_scalar::*;
use ark_ff::BigInt;

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
